use crate::{
    Compiler, CompilerId, CompilerView, DateTime, Device, DeviceView, Error, Firmware,
    Organization, Result, TargetPrototype, TargetPrototypeId, Transaction, User,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[id]
pub struct CollectionId;

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    #[copy]
    id: CollectionId,
    name: String,
    description: Option<String>,
    compiler: Option<CompilerView>,
    devices: Vec<DeviceView>,
    target_prototype: TargetPrototype,
    #[copy]
    created_at: DateTime,
    #[copy]
    updated_at: DateTime,
}

impl CollectionView {
    pub async fn new(txn: &mut Transaction<'_>, collection: Collection) -> Result<Self> {
        let devices = collection.devices(txn).await?;
        let compiler = collection.compiler(txn).await?;
        let compiler = match compiler {
            Some(c) => Some(CompilerView::new(txn, c).await?),
            None => None,
        };
        Ok(Self {
            id: collection.id,
            target_prototype: collection.target_prototype(txn).await?,
            name: collection.name,
            description: collection.description,
            compiler,
            devices,
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    #[copy]
    id: CollectionId,
    name: String,
    description: Option<String>,
    #[copy]
    target_prototype_id: TargetPrototypeId,
    #[copy]
    compiler_id: Option<CompilerId>,
    #[copy]
    created_at: DateTime,
    #[copy]
    updated_at: DateTime,
}

impl Collection {
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        target_prototype_id: TargetPrototypeId,
        organization: &Organization,
    ) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::InvalidName);
        }

        let (id, now) = sqlx::query_as::<_, (CollectionId, DateTime)>(
            "INSERT INTO collections (name, target_prototype_id) VALUES ($1, $2) RETURNING id, created_at",
        )
        .bind(&name)
        .bind(&target_prototype_id)
        .fetch_one(&mut *txn)
        .await?;

        let mut col = Self {
            id,
            name,
            target_prototype_id,
            description: None,
            compiler_id: None,
            created_at: now,
            updated_at: now,
        };
        col.associate_to_organization(txn, organization).await?;

        Ok(col)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        collection_id: CollectionId,
        user: &User,
    ) -> Result<Self> {
        let collection: Self = sqlx::query_as(
            "SELECT col.id, col.target_prototype_id, col.name, col.description, col.compiler_id, col.created_at, col.updated_at
             FROM collections as col
             INNER JOIN collection_belongs_to_organization cbt ON cbt.collection_id = col.id
             INNER JOIN user_belongs_to_organization ubt ON ubt.organization_id = cbt.organization_id
             WHERE col.id = $1 AND ubt.user_id = $2",
        )
            .bind(collection_id)
            .bind(user.id())
            .fetch_one(&mut *txn)
            .await?;
        Ok(collection)
    }

    pub async fn from_organization(
        txn: &mut Transaction<'_>,
        organization: &Organization,
    ) -> Result<Vec<Self>> {
        let collections: Vec<Self> = sqlx::query_as(
            "SELECT col.id, col.target_prototype_id, col.name, col.description, col.compiler_id, col.created_at, col.updated_at
             FROM collections as col
             INNER JOIN collection_belongs_to_organization as cbt ON cbt.collection_id = col.id
             WHERE cbt.organization_id = $1",
        )
        .bind(organization.id())
        .fetch_all(&mut *txn)
        .await?;
        Ok(collections)
    }

    pub async fn associate_to_organization(
        &mut self,
        txn: &mut Transaction<'_>,
        organization: &Organization,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO collection_belongs_to_organization (collection_id, organization_id) VALUES ($1, $2)",
        )
        .bind(self.id)
        .bind(organization.id())
        .execute(&mut *txn)
        .await?;

        let (updated_at,): (DateTime,) = sqlx::query_as(
            "UPDATE collections SET updated_at = NOW() WHERE id = $1 RETURNING updated_at",
        )
        .bind(self.id())
        .fetch_one(txn)
        .await?;
        self.updated_at = updated_at;
        Ok(())
    }

    pub async fn devices(&self, txn: &mut Transaction<'_>) -> Result<Vec<DeviceView>> {
        let devices = Device::from_collection(txn, self).await?;
        let mut device_views = Vec::with_capacity(devices.len());
        for device in devices {
            device_views.push(DeviceView::new(txn, device).await?);
        }
        Ok(device_views)
    }

    pub async fn find_by_device(txn: &mut Transaction<'_>, device: &Device) -> Result<Self> {
        let collection = sqlx::query_as(
            "SELECT col.id, col.target_prototype_id, col.name, col.description, col.compiler_id, col.created_at, col.updated_at
             FROM collections as col
             INNER JOIN devices ON devices.collection_id = col.id
             WHERE devices.id = $1",
        )
        .bind(device.id())
        .fetch_one(txn)
        .await?;
        Ok(collection)
    }

    pub async fn find_by_compiler(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
    ) -> Result<Option<Self>> {
        let collection = sqlx::query_as(
            "SELECT col.id, col.target_prototype_id, col.name, col.description, col.compiler_id,  col.created_at, col.updated_at
            FROM collections as col
            WHERE col.compiler_id = $1",
        )
        .bind(compiler.id())
        .fetch_optional(txn)
        .await?;
        Ok(collection)
    }

    pub async fn organization(&self, txn: &mut Transaction<'_>) -> Result<Organization> {
        Organization::find_by_collection(txn, self).await
    }

    pub async fn set_compiler(
        &mut self,
        txn: &mut Transaction<'_>,
        compiler: Option<&Compiler>,
    ) -> Result<()> {
        if self.compiler_id == compiler.map(|c| c.id()) {
            return Ok(());
        }

        let (updated_at,): (DateTime,) = sqlx::query_as("UPDATE collections SET compiler_id = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at")
            .bind(compiler.map(|c| c.id()))
            .bind(self.id)
            .fetch_one(txn)
            .await?;
        self.updated_at = updated_at;
        self.compiler_id = compiler.map(|c| c.id());
        Ok(())
    }

    pub async fn compiler(&self, txn: &mut Transaction<'_>) -> Result<Option<Compiler>> {
        if let Some(compiler_id) = self.compiler_id {
            let organization = self.organization(txn).await?;
            Ok(Some(
                Compiler::find_by_id(txn, &organization, compiler_id).await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn update(&self, txn: &mut Transaction<'_>) -> Result<Option<Firmware>> {
        if let Some(compiler) = self.compiler(txn).await? {
            let compilation = compiler.latest_compilation(txn).await?;
            Ok(Some(compilation.firmware(txn).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_name(&mut self, txn: &mut Transaction<'_>, name: String) -> Result<()> {
        let (updated_at,): (DateTime,) = sqlx::query_as(
            "UPDATE collections SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at",
        )
        .bind(&name)
        .bind(self.id)
        .fetch_one(txn)
        .await?;
        self.updated_at = updated_at;
        self.name = name;
        Ok(())
    }

    pub async fn delete(self, txn: &mut Transaction<'_>) -> Result<()> {
        sqlx::query("DELETE FROM collection_belongs_to_organization where collection_id = $1")
            .bind(self.id)
            .execute(&mut *txn)
            .await?;
        sqlx::query("DELETE FROM collections where id = $1")
            .bind(self.id)
            .execute(txn)
            .await?;
        Ok(())
    }

    pub async fn target_prototype(&self, txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
        TargetPrototype::find_by_id(txn, self.target_prototype_id).await
    }
}

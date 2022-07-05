use crate::db::timestamp::{now, DateTime};
use crate::{prelude::*, DeviceView, Organization, User, Device};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct CollectionId(pub i64);

impl CollectionId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub devices: Vec<DeviceView>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl CollectionView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        collection: Collection,
        user: &User,
    ) -> Result<Self> {
        let devices = collection.devices(txn, user).await?;
        Ok(Self {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            devices,
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    id: CollectionId,
    name: String,
    description: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl Collection {
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        organization: &Organization,
    ) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::BadData);
        }

        let (id,) = sqlx::query_as::<_, (CollectionId,)>(
            "INSERT INTO collections (name) VALUES ($1) RETURNING id",
        )
        .bind(&name)
        .fetch_one(&mut *txn)
        .await?;

        let col = Self {
            id,
            name,
            description: None,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
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
            "SELECT col.id, col.name, col.description, col.created_at, col.updated_at
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
            "SELECT col.id, col.name, col.description, col.created_at, col.updated_at
             FROM collections as col
             INNER JOIN collection_belongs_to_organization as cbt ON cbt.collection_id = col.id
             WHERE cbt.organization_id = $1",
        )
        .bind(organization.id())
        .fetch_all(&mut *txn)
        .await?;
        Ok(collections)
    }

    pub fn id(&self) -> CollectionId {
        self.id
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
            "UPDATE users SET updated_at = NOW() WHERE id = $1 RETURNING updated_at",
        )
        .bind(self.id())
        .fetch_one(txn)
        .await?;
        self.updated_at = updated_at;
        Ok(())
    }

    pub async fn devices(&self, txn: &mut Transaction<'_>, user: &User) -> Result<Vec<DeviceView>> {
        let devices = Device::from_collection(txn, self.id, user).await?;
        let mut device_views = Vec::with_capacity(devices.len());
        for device in devices {
            device_views.push(DeviceView::new(txn, device).await?);
        }
        Ok(device_views)
    }
}

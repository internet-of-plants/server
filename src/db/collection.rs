use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::{Device, OrganizationId};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct CollectionId(i64);

impl CollectionId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CollectionView {
    id: CollectionId,
    name: String,
    description: Option<String>,
    devices: Vec<Device>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl CollectionView {
    pub fn new_from_collection_and_devices(
        collection: Collection,
        devices: Vec<Device>,
    ) -> Result<Self> {
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
    pub fn id(&self) -> &CollectionId {
        &self.id
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        organization_id: &OrganizationId,
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
        Self::associate_to_organization(&mut *txn, &id, organization_id).await?;
        Ok(Self {
            id,
            name,
            description: None,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
        })
    }

    pub async fn associate_to_organization(
        txn: &mut Transaction<'_>,
        collection_id: &CollectionId,
        organization_id: &OrganizationId,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO collection_belongs_to_organization (collection_id, organization_id) VALUES ($1, $2)",
        )
        .bind(collection_id)
        .bind(organization_id)
        .execute(&mut *txn)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        collection_id: &CollectionId,
    ) -> Result<Self> {
        let collection: Self = sqlx::query_as(
            "SELECT col.id, col.name, col.description, col.created_at, col.updated_at
             FROM collections as col
             WHERE col.id = $1",
        )
        .bind(collection_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(collection)
    }

    pub async fn from_organization(
        txn: &mut Transaction<'_>,
        organization_id: &OrganizationId,
    ) -> Result<Vec<Self>> {
        let collections: Vec<Self> = sqlx::query_as(
            "SELECT col.id, col.name, col.description, col.created_at, col.updated_at
                FROM collections as col
                INNER JOIN collection_belongs_to_organization as bt ON bt.collection_id = col.id
                WHERE bt.organization_id = $1",
        )
        .bind(organization_id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(collections)
    }
}

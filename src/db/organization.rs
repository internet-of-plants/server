use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::{Collection, User, UserId, Username};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct OrganizationId(pub i64);

impl OrganizationId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OrganizationView {
    pub id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub collections: Vec<Collection>,
    pub members: Vec<Username>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl OrganizationView {
    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        organization_id: &OrganizationId,
    ) -> Result<Self> {
        // TODO: this is dumb, we are making too many roundtrips to the db, but it's less complex,
        // let's optimize later
        let organization = Organization::find_by_id(&mut *txn, organization_id).await?;
        let collections = Collection::from_organization(&mut *txn, organization_id).await?;
        let members = User::from_organization(&mut *txn, organization_id).await?;
        Ok(Self {
            id: organization.id,
            name: organization.name,
            description: organization.description,
            collections,
            members,
            created_at: organization.created_at,
            updated_at: organization.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Organization {
    id: OrganizationId,
    name: String,
    description: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl Organization {
    pub fn id(&self) -> &OrganizationId {
        &self.id
    }

    pub async fn new(txn: &mut Transaction<'_>, name: String) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::BadData);
        }

        let (id,) = sqlx::query_as::<_, (OrganizationId,)>(
            "INSERT INTO organizations (name) VALUES ($1) RETURNING id",
        )
        .bind(&name)
        .fetch_one(&mut *txn)
        .await?;
        Ok(Self {
            id,
            name,
            description: None,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
        })
    }

    pub async fn default_for_user(txn: &mut Transaction<'_>, user_id: &UserId) -> Result<Self> {
        let organization: Self = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM organizations as w 
             INNER JOIN users as u ON u.default_organization_id = w.id
             WHERE w.id = $1",
        )
        .bind(user_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(organization)
    }

    pub async fn from_user(txn: &mut Transaction<'_>, user_id: &UserId) -> Result<Vec<Self>> {
        let organizations: Vec<Organization> = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM organizations as w
             INNER JOIN user_belongs_to_organization as bt ON bt.organization_id = w.id
             WHERE bt.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(organizations)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        organization_id: &OrganizationId,
    ) -> Result<Self> {
        let organization: Self = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM organizations as w
             WHERE w.id = $1",
        )
        .bind(organization_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(organization)
    }
}
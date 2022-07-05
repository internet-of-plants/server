use crate::db::timestamp::{now, DateTime};
use crate::{prelude::*, CollectionView};
use crate::{Collection, User, Username};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationView {
    pub id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub collections: Vec<CollectionView>,
    pub members: Vec<Username>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl OrganizationView {
pub async fn new(txn: &mut Transaction<'_>, organization: &Organization, user: &User) -> Result<Self> {
        // TODO: this is dumb, we are making too many roundtrips to the db, but it's less complex,
        // let's optimize later
        let cols = Collection::from_organization(txn, organization).await?;
        let mut collections = Vec::with_capacity(cols.len());
        for col in cols {
            collections.push(CollectionView::new(txn, col, user).await?);
        }

        let members = User::from_organization(txn, organization).await?;
        Ok(Self {
            id: organization.id,
            name: organization.name.clone(),
            description: organization.description.clone(),
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
    pub async fn new(txn: &mut Transaction<'_>, name: String) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::BadData);
        }

        let (id,) = sqlx::query_as::<_, (OrganizationId,)>(
            "INSERT INTO organizations (name) VALUES ($1) RETURNING id",
        )
        .bind(&name)
        .fetch_one(txn)
        .await?;
        Ok(Self {
            id,
            name,
            description: None,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
        })
    }

    pub async fn default_for_user(txn: &mut Transaction<'_>, user: &User) -> Result<Self> {
        let organization: Self = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM organizations as w 
             INNER JOIN users as u ON u.default_organization_id = w.id
             WHERE w.id = $1",
        )
        .bind(user.id())
        .fetch_one(txn)
        .await?;
        Ok(organization)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        organization_id: OrganizationId,
        user: &User,
    ) -> Result<Self> {
        let organization: Self = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM organizations as w
             INNER JOIN user_belongs_to_organization as bt ON bt.organization_id = w.id
             WHERE w.id = $1 AND bt.user_id = $2",
        )
        .bind(organization_id)
        .bind(user.id())
        .fetch_one(&mut *txn)
        .await?;
        Ok(organization)
    }

    pub async fn from_user(txn: &mut Transaction<'_>, user: &User) -> Result<Vec<Self>> {
        let organizations: Vec<Organization> = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM organizations as w
             INNER JOIN user_belongs_to_organization as bt ON bt.organization_id = w.id
             WHERE bt.user_id = $1",
        )
        .bind(user.id())
        .fetch_all(&mut *txn)
        .await?;
        Ok(organizations)
    }

    pub fn id(&self) -> &OrganizationId {
        &self.id
    }
}

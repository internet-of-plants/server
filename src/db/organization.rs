use crate::{
    Collection, CollectionView, Compiler, DateTime, Error, Result, Transaction, User, Username,
};
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
    pub async fn new(txn: &mut Transaction<'_>, organization: &Organization) -> Result<Self> {
        let cols = Collection::from_organization(txn, organization).await?;
        let mut collections = Vec::with_capacity(cols.len());
        for col in cols {
            collections.push(CollectionView::new(txn, col).await?);
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

    pub fn id(&self) -> OrganizationId {
        self.id
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
            return Err(Error::InvalidName);
        }

        let (id, now) = sqlx::query_as::<_, (OrganizationId, DateTime)>(
            "INSERT INTO organizations (name) VALUES ($1) RETURNING id, created_at",
        )
        .bind(&name)
        .fetch_one(txn)
        .await?;
        Ok(Self {
            id,
            name,
            description: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn default_for_user(txn: &mut Transaction<'_>, user: &User) -> Result<Self> {
        let organization: Self = sqlx::query_as(
            "SELECT o.id, o.name, o.description, o.created_at, o.updated_at
             FROM organizations as o 
             INNER JOIN users as u ON u.default_organization_id = o.id
             WHERE u.id = $1",
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

    pub async fn find_by_collection(
        txn: &mut Transaction<'_>,
        collection: &Collection,
    ) -> Result<Self> {
        let organization = sqlx::query_as(
            "SELECT o.id, o.name, o.description, o.created_at, o.updated_at
             FROM organizations as o
             INNER JOIN collection_belongs_to_organization as bt ON bt.organization_id = o.id
             WHERE bt.collection_id = $1",
        )
        .bind(collection.id())
        .fetch_one(&mut *txn)
        .await?;
        Ok(organization)
    }

    pub async fn find_by_compiler(txn: &mut Transaction<'_>, compiler: &Compiler) -> Result<Self> {
        let organization = sqlx::query_as(
            "SELECT o.id, o.name, o.description, o.created_at, o.updated_at
             FROM organizations as o
             INNER JOIN compilers ON compilers.organization_id = o.id
             WHERE compilers.id = $1",
        )
        .bind(compiler.id())
        .fetch_one(&mut *txn)
        .await?;
        Ok(organization)
    }

    pub fn id(&self) -> OrganizationId {
        self.id
    }
}

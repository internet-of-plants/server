use crate::db::collection::{Collection, CollectionView};
use crate::db::device::DeviceView;
use crate::db::timestamp::{now, DateTime};
use crate::db::user::{User, UserId, Username};
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct WorkspaceId(i64);

impl WorkspaceId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceView {
    id: WorkspaceId,
    name: String,
    description: Option<String>,
    collections: Vec<Collection>,
    members: Vec<Username>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl WorkspaceView {
    pub async fn find_by_id(txn: &mut Transaction<'_>, workspace_id: &WorkspaceId) -> Result<Self> {
        // TODO: this is dumb, we are making too many roundtrips to the db, but it's less complex,
        // let's optimize later
        let workspace = Workspace::find_by_id(&mut *txn, workspace_id).await?;
        let collections = Collection::from_workspace(&mut *txn, workspace_id).await?;
        let members = User::from_workspace(&mut *txn, workspace_id).await?;
        Ok(Self {
            id: workspace.id,
            name: workspace.name,
            description: workspace.description,
            collections,
            members,
            created_at: workspace.created_at,
            updated_at: workspace.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    description: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl Workspace {
    pub fn id(&self) -> &WorkspaceId {
        &self.id
    }

    pub async fn new(txn: &mut Transaction<'_>, name: String) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::BadData);
        }

        let (id,) = sqlx::query_as::<_, (WorkspaceId,)>(
            "INSERT INTO workspaces (name) VALUES ($1) RETURNING id",
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
        let workspace: Self = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM workspaces as w 
             INNER JOIN users as u ON u.default_workspace_id = w.id
             WHERE w.id = $1",
        )
        .bind(user_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(workspace)
    }

    pub async fn from_user(txn: &mut Transaction<'_>, user_id: &UserId) -> Result<Vec<Self>> {
        let workspaces: Vec<Workspace> = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM workspaces as w
             INNER JOIN user_belongs_to_workspace as bt ON bt.workspace_id = w.id
             WHERE bt.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(workspaces)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, workspace_id: &WorkspaceId) -> Result<Self> {
        let workspace: Self = sqlx::query_as(
            "SELECT w.id, w.name, w.description, w.created_at, w.updated_at
             FROM workspaces as w
             WHERE w.id = $1",
        )
        .bind(workspace_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(workspace)
    }

    pub fn set_name(name: String) {
        todo!();
    }

    pub fn set_description(description: String) {
        todo!();
    }

    pub fn unset_description() {
        todo!();
    }
}

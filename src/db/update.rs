use crate::{Device, DeviceId, UserId};
use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use codegen::exec_time;

#[exec_time]
//#[cache(valid_for = 30)]
pub async fn get(
    txn: &mut Transaction<'_>,
    _user_id: UserId,
    device_id: DeviceId,
) -> Result<Option<Update>> {
    // TODO: we currently don't allow global updates, but we should (at least by groups)
    let last_update: Option<Update> = sqlx::query_as(
        "SELECT id, collection_id, file_hash, file_name, version, created_at
        FROM binary_updates
        WHERE collection_id = $1
        ORDER BY created_at DESC",
    )
    .bind(device_id)
    .fetch_optional(txn)
    .await?;
    Ok(last_update)
}

#[exec_time]
pub async fn new(
    txn: &mut Transaction<'_>,
    _user_id: UserId,
    device_id: DeviceId,
    file_hash: String,
    file_name: String,
    version: String,
) -> Result<()> {
    // TODO: Redundant since we have device_id?
    //db::plant::owns(txn, user_id, device_id).await?;
    sqlx::query(
        "INSERT INTO binary_updates (collection_id, file_hash, file_name, version) VALUES ($1, $2, $3, $4)",
    )
    .bind(Device::find_by_id(txn, &device_id).await?.collection_id())
    .bind(&file_hash)
    .bind(&file_name)
    .bind(&version)
    .execute(txn)
    .await?;
    Ok(())
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct UpdateId(i64);

impl UpdateId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Update {
    id: UpdateId,
    name: String,
    description: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl Update {
    pub fn id(&self) -> &UpdateId {
        &self.id
    }

    pub async fn new(txn: &mut Transaction<'_>, name: String) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::BadData);
        }

        let (id,) = sqlx::query_as::<_, (UpdateId,)>(
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
        let workspaces: Vec<Update> = sqlx::query_as(
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

    pub async fn find_by_id(txn: &mut Transaction<'_>, workspace_id: &UpdateId) -> Result<Self> {
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

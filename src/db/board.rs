use crate::db::target_prototype::*;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct BoardId(i64);

impl BoardId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Board {
    id: BoardId,
    pub board: String,
    target_prototype_id: TargetPrototypeId,
    pub pin_hpp: String,
}

impl Board {
    pub async fn new(
        txn: &mut Transaction<'_>,
        board: String,
        pins: Vec<String>,
        pin_hpp: String,
        target_prototype_id: TargetPrototypeId,
    ) -> Result<Self> {
        let (id,): (BoardId,) = sqlx::query_as(
            "INSERT INTO boards (board, target_prototype_id, pin_hpp) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&board)
        .bind(&target_prototype_id)
        .bind(&pin_hpp)
        .fetch_one(&mut *txn)
        .await?;
        for pin in pins {
            sqlx::query("INSERT INTO board_pins (board_id, name) VALUES ($1, $2)")
                .bind(id)
                .bind(pin)
                .execute(&mut *txn)
                .await?;
        }
        Ok(Self {
            id,
            board,
            pin_hpp,
            target_prototype_id,
        })
    }

    pub fn id(&self) -> BoardId {
        self.id
    }

    pub async fn pins(&self, txn: &mut Transaction<'_>) -> Result<Vec<String>> {
        let pins = sqlx::query_as("SELECT name FROM board_pins WHERE board_id = $1")
            .bind(self.id)
            .fetch_all(&mut *txn)
            .await?
            .into_iter()
            .map(|(name,)| name)
            .collect();
        Ok(pins)
    }

    pub async fn list_by_target_prototype(
        txn: &mut Transaction<'_>,
        target_prototype_id: TargetPrototypeId,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, board, target_prototype_id, pin_hpp FROM boards WHERE target_prototype_id = $1",
        )
        .bind(&target_prototype_id)
        .fetch_all(&mut *txn)
        .await?)
    }

    pub fn pin_hpp(&self) -> &str {
        &self.pin_hpp
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: BoardId) -> Result<Self> {
        let board = sqlx::query_as(
            "SELECT id, board, target_prototype_id, pin_hpp FROM boards WHERE id = $1",
        )
        .bind(&id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(board)
    }
}

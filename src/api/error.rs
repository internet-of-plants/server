use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
#[cache(valid_for = 30)]
pub async fn index(pool: &'static Pool, user_id: i64) -> Result<Vec<ErrorDump>> {
    let errors: Vec<ErrorDump> = sqlx::query_as(
        "SELECT errors.id, errors.plant_id, errors.error, errors.created_at
        FROM errors
        INNER JOIN plants ON plants.id = errors.plant_id
        WHERE errors.is_solved = FALSE AND plants.owner_id = $1
        ORDER BY errors.created_at ASC")
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(errors)
}

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, error: ErrorReport) -> Result<()> {
    api::plant::owns(pool, user_id, error.plant_id).await?;
    sqlx::query("INSERT INTO errors (plant_id, error) VALUES ($1, $2)")
        .bind(&error.plant_id)
        .bind(&error.error)
        .execute(pool)
        .await?;
    Ok(())
}

#[exec_time]
pub async fn solve(pool: &'static Pool, user_id: i64, error_id: i64) -> Result<()> {
    api::error::owns(pool, user_id, error_id).await?;
    sqlx::query("UPDATE errors SET is_solved = TRUE WHERE id = $1")
        .bind(&error_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[exec_time]
pub async fn owns(pool: &'static Pool, user_id: i64, error_id: i64) -> Result<()> {
    let exists: Option<(i32,)> = sqlx::query_as(
        "SELECT 1
        FROM errors
        INNER JOIN plants ON plants.id = errors.plant_id
        WHERE plants.owner_id = $1
              AND errors.id = $2")
        .bind(user_id)
        .bind(error_id)
        .fetch_optional(pool)
        .await?;
    match exists {
        Some(_) => Ok(()),
        None => Err(Error::NothingFound),
    }
}

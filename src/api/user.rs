use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
#[cache(valid_for = 3600)]
pub async fn authenticate(pool: &'static Pool, token: String) -> Result<i64> {
    let id: Option<Id> = sqlx::query_as(
        "SELECT users.id
        FROM users
        INNER JOIN authentications ON authentications.user_id = users.id
        WHERE authentications.token = $1",
    )
    .bind(&token)
    .fetch_optional(pool)
    .await?;
    match id {
        Some(Id { id }) => Ok(id),
        None => Err(Error::Forbidden),
    }
}

#[exec_time]
pub async fn new(pool: &'static Pool, user: &NewUser) -> Result<()> {
    if user.password.is_empty() || user.email.is_empty() || user.username.is_empty() {
        return Err(Error::BadData);
    }

    let exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM users WHERE users.email = $1 OR users.username = $2")
            .bind(&user.email)
            .bind(&user.username)
            .fetch_optional(pool)
            .await?;
    if exists.is_some() {
        warn!("User already exists");
        return Err(Error::Forbidden);
    }

    let hash = utils::hash_password(&user.password)?;
    sqlx::query("INSERT INTO users (email, password_hash, username) VALUES ($1, $2, $3)")
        .bind(&user.email)
        .bind(&hash)
        .bind(&user.username)
        .execute(pool)
        .await?;
    Ok(())
}

#[exec_time]
pub async fn login(pool: &'static Pool, client: Login) -> Result<String> {
    let hash: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, password_hash
        FROM users
        WHERE email = $1",
    )
    .bind(&client.email)
    .fetch_optional(pool)
    .await?;
    let is_auth = match &hash {
        Some((_, hash)) => utils::verify_password(&client.password, hash)?,
        // Avoids timing attacks to detect usernames
        None => {
            // Pwease don't optimize out, m'compiler, TODO: can we ensure that?
            let _fake_hash = utils::hash_password(&client.password)?;
            false
        }
    };

    match (hash, is_auth) {
        (Some((user_id, _)), true) => {
            let plant_id = match client.mac {
                Some(mac) => Some(api::plant::put(pool, user_id, mac).await?),
                None => None,
            };

            let token = utils::random_string(64);
            sqlx::query(
                "INSERT INTO authentications (user_id, plant_id, token) VALUES ($1, $2, $3)",
            )
            .bind(user_id)
            .bind(&plant_id)
            .bind(&token)
            .execute(pool)
            .await?;
            Ok(token)
        }
        _ => Err(Error::NothingFound),
    }
}

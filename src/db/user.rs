use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::Organization;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct Username(String);

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct UserId(i64);

impl UserId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub username: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Debug, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct AuthToken(pub String);

impl AuthToken {
    pub fn random() -> Self {
        Self(utils::random_string(64))
    }
}

impl AuthToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }
}

impl User {
    pub async fn new(txn: &mut Transaction<'_>, user: NewUser) -> Result<Self> {
        // TODO: improve password rules
        if user.password.is_empty() {
            return Err(Error::BadData);
        }

        let organization = Organization::new(&mut *txn, user.username.clone()).await?;

        let hash = utils::hash_password(&user.password)?;
        let (id,) = sqlx::query_as::<_, (UserId,)>(
            "INSERT INTO users (email, password_hash, username, default_organization_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(&user.email)
        .bind(&hash)
        .bind(&user.username)
        .bind(organization.id())
        .fetch_one(&mut *txn)
        .await?;

        let user = Self {
            id,
            email: user.email,
            username: user.username,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
        };
        user.associate_to_organization(&mut *txn, &organization)
            .await?;

        Ok(user)
    }

    pub async fn find_by_auth_token(txn: &mut Transaction<'_>, token: AuthToken) -> Result<Self> {
        let user: Option<Self> = sqlx::query_as(
            "SELECT users.id, users.email, users.username, users.created_at, users.updated_at
             FROM users
             INNER JOIN authentications ON authentications.user_id = users.id
             WHERE authentications.token = $1",
        )
        .bind(&token)
        .fetch_optional(&mut *txn)
        .await?;
        user.ok_or(Error::Forbidden)
    }

    pub async fn login(
        txn: &mut Transaction<'_>,
        client: Login,
    ) -> Result<AuthToken> {
        let hash: Option<(UserId, String, String, DateTime, DateTime, String)> = sqlx::query_as(
            "SELECT id, email, username, created_at, updated_at, password_hash
            FROM users
            WHERE email = $1",
        )
        .bind(&client.email)
        .fetch_optional(&mut *txn)
        .await?;
        let is_auth = match &hash {
            Some((_, _, _, _, _, hash)) => utils::verify_password(&client.password, hash)?,
            // Avoids timing attacks to detect usernames
            None => utils::hash_password(&client.password)? == "abc"
        };

        match (hash, is_auth) {
            (Some((id, email, username, created_at, updated_at, _)), true) => {
                let user = Self {
                    id,
                    email,
                    username,
                    created_at,
                    updated_at,
                };

                let token = AuthToken::random();
                sqlx::query(
                    "INSERT INTO authentications (user_id, token) VALUES ($1, $2)",
                )
                .bind(user.id())
                .bind(&token)
                .execute(&mut *txn)
                .await?;
                Ok(token)
            }
            _ => Err(Error::Forbidden)
        }
    }

    pub async fn from_organization(
        txn: &mut Transaction<'_>,
        organization: &Organization,
    ) -> Result<Vec<Username>> {
        let users: Vec<Username> = sqlx::query_as::<_, (Username,)>(
            "SELECT u.username
             FROM users as u
             INNER JOIN user_belongs_to_organization as bt ON bt.user_id = u.id
             WHERE bt.organization_id = $1",
        )
        .bind(organization.id())
        .fetch_all(&mut *txn)
        .await?
        .into_iter()
        .map(|(username,)| username)
        .collect();
        Ok(users)
    }

    pub async fn associate_to_organization(
        &self,
        txn: &mut Transaction<'_>,
        organization: &Organization,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO user_belongs_to_organization (user_id, organization_id) VALUES ($1, $2)",
        )
        .bind(self.id())
        .bind(organization.id())
        .execute(&mut *txn)
        .await?;
        Ok(())
    }

    pub fn id(&self) -> UserId {
        self.id
    }
}

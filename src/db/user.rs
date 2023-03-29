use crate::{utils, AuthToken, DateTime, Error, Organization, Result, Transaction};
use derive_more::FromStr;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Getters, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    email: String,
    password: String,
    username: String,
    organization_name: String,
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    #[copy]
    pub id: UserId,
    pub email: String,
    pub username: String,
    #[copy]
    pub created_at: DateTime,
    #[copy]
    pub updated_at: DateTime,
}

impl User {
    pub async fn new(txn: &mut Transaction<'_>, user: NewUser) -> Result<Self> {
        // TODO: improve password rules and error reporting
        if user.password.len() < 10 {
            return Err(Error::InsecurePassword);
        }

        let organization = Organization::new(txn, user.organization_name).await?;

        let hash = utils::hash_password(&user.password)?;
        let (id, now) = sqlx::query_as::<_, (UserId, DateTime)>(
            "INSERT INTO users (email, password_hash, username, default_organization_id) VALUES ($1, $2, $3, $4) RETURNING id, created_at",
        )
        .bind(&user.email)
        .bind(&hash)
        .bind(&user.username)
        .bind(organization.id())
        .fetch_one(&mut *txn)
        .await?;

        let mut user = Self {
            id,
            email: user.email,
            username: user.username,
            created_at: now,
            updated_at: now,
        };
        user.associate_to_organization(txn, &organization).await?;

        Ok(user)
    }

    pub async fn find_by_auth_token(txn: &mut Transaction<'_>, token: AuthToken) -> Result<Self> {
        let user: Option<Self> = sqlx::query_as(
            "SELECT users.id, users.email, users.username, users.created_at, users.updated_at
             FROM users
             INNER JOIN authentications ON authentications.user_id = users.id
             WHERE authentications.token = $1 AND authentications.expired = false",
        )
        .bind(&token)
        .fetch_optional(txn)
        .await?;
        user.ok_or(Error::Unauthorized)
    }

    pub async fn login(txn: &mut Transaction<'_>, client: Login) -> Result<AuthToken> {
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
            None => utils::hash_password(&client.password)? == "abc",
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
                sqlx::query("INSERT INTO authentications (user_id, token) VALUES ($1, $2)")
                    .bind(user.id())
                    .bind(&token)
                    .execute(txn)
                    .await?;
                Ok(token)
            }
            _ => Err(Error::Unauthorized),
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
        .fetch_all(txn)
        .await?
        .into_iter()
        .map(|(username,)| username)
        .collect();
        Ok(users)
    }

    pub async fn associate_to_organization(
        &mut self,
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

        let (updated_at,): (DateTime,) = sqlx::query_as(
            "UPDATE users SET updated_at = NOW() WHERE id = $1 RETURNING updated_at",
        )
        .bind(self.id())
        .fetch_one(txn)
        .await?;
        self.updated_at = updated_at;
        Ok(())
    }

    pub async fn default_organization(&self, txn: &mut Transaction<'_>) -> Result<Organization> {
        Organization::default_for_user(txn, self).await
    }
}

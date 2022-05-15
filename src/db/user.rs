use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::{Device, NewDevice, Organization, OrganizationId};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: UserId,
    email: String,
    username: String,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Debug, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct AuthToken(pub String);

impl AuthToken {
    pub fn random() -> Self {
        Self(utils::random_string(64))
    }
}

//impl Reply for AuthToken {
//    fn into_response(self) -> warp::reply::Response {
//        self.0.into_response()
//    }
//}

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
        Self::associate_to_organization(&mut *txn, &id, organization.id()).await?;

        Ok(Self {
            id,
            email: user.email,
            username: user.username,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
        })
    }

    // TODO: device auth should be tied to the device not the user, so the user can delete their account, or move
    // the device between collections/organizations
    // TODO: move this to Auth struct
    pub async fn find_by_auth_token(txn: &mut Transaction<'_>, token: AuthToken) -> Result<Auth> {
        let auth: Option<Auth> = sqlx::query_as(
            "SELECT users.id as user_id, authentications.device_id
             FROM users
             INNER JOIN authentications ON authentications.user_id = users.id
             WHERE authentications.token = $1",
        )
        .bind(&token)
        .fetch_optional(&mut *txn)
        .await?;
        Ok(auth.ok_or(Error::Forbidden)?)
    }

    pub async fn login(
        txn: &mut Transaction<'_>,
        client: Login,
        new_device: Option<NewDevice>,
    ) -> Result<AuthToken> {
        let hash: Option<(UserId, String)> = sqlx::query_as(
            "SELECT id, password_hash
            FROM users
            WHERE email = $1",
        )
        .bind(&client.email)
        .fetch_optional(&mut *txn)
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
                let device = match new_device {
                    Some(new_device) => Some(Device::put(txn, &user_id, new_device).await?),
                    None => None,
                };

                let token = AuthToken::random();
                sqlx::query(
                    "INSERT INTO authentications (user_id, device_id, token) VALUES ($1, $2, $3)",
                )
                .bind(user_id)
                .bind(device.map(|d| *d.id()))
                .bind(&token)
                .execute(&mut *txn)
                .await?;
                Ok(token)
            }
            _ => Err(Error::NothingFound),
        }
    }

    pub fn verify_email() -> Result<()> {
        todo!();
    }

    pub async fn from_organization(
        txn: &mut Transaction<'_>,
        organization_id: &OrganizationId,
    ) -> Result<Vec<Username>> {
        let users: Vec<Username> = sqlx::query_as::<_, (Username,)>(
            "SELECT u.username
             FROM users as u
             INNER JOIN user_belongs_to_organization as bt ON bt.user_id = u.id
             WHERE bt.organization_id = $1",
        )
        .bind(organization_id)
        .fetch_all(&mut *txn)
        .await?
        .into_iter()
        .map(|(username,)| username)
        .collect();
        Ok(users)
    }

    pub async fn associate_to_organization(
        txn: &mut Transaction<'_>,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO user_belongs_to_organization (user_id, organization_id) VALUES ($1, $2)",
        )
        .bind(user_id)
        .bind(organization_id)
        .execute(&mut *txn)
        .await?;
        Ok(())
    }
}

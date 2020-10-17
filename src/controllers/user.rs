use crate::prelude::*;
use controllers::Result;

pub async fn new(pool: &'static Pool, user: NewUser) -> Result<impl Reply> {
    api::user::new(pool, &user).await?;
    login(pool, Login { email: user.email, password: user.password }).await
}

pub async fn login(pool: &'static Pool, user: Login) -> Result<impl Reply> {
    Ok(api::user::login(pool, user).await?)
}

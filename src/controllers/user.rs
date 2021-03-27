use crate::prelude::*;
use controllers::Result;

pub async fn new(pool: &'static Pool, user: NewUser, mac: String) -> Result<impl Reply> {
    // We should fix al the avenues for user abuse before allowing signups
    //return Result::<&'static str, _>::Err(Error::Forbidden.into());
 
    api::user::new(pool, &user).await?;
    login(
        pool,
        Login {
            email: user.email,
            password: user.password,
        },
        mac
    )
    .await
}

pub async fn login(pool: &'static Pool, user: Login, mac: String) -> Result<impl Reply> {
    Ok(api::user::login(pool, user, Some(mac)).await?)
}

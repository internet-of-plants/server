use crate::prelude::*;
use controllers::Result;

pub async fn new(_pool: &'static Pool, _user: NewUser) -> Result<impl Reply> {
    // We should fix al the avenues for user abuse before allowing signups
    return Result::<&'static str, _>::Err(Error::Forbidden.into());
 
    //api::user::new(pool, &user).await?;
    //login(
    //    pool,
    //    Login {
    //        email: user.email,
    //        password: user.password,
    //    },
    //    None,
    //)
    //.await
}

pub async fn login(pool: &'static Pool, user: Login, mac: Option<String>) -> Result<impl Reply> {
    info!("Login: {:?}, {:?}", user, mac);
    Ok(api::user::login(pool, user, mac).await?)
}

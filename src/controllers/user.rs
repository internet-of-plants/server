use crate::{NewDevice, User, NewUser};
use crate::prelude::*;
use controllers::Result;

pub async fn new(pool: &'static Pool, user: NewUser) -> Result<impl Reply> {
    // We should fix al the avenues for user abuse before allowing signups
    //return Result::<&'static str, _>::Err(Error::Forbidden.into());

    let mut txn = pool.begin().await.map_err(Error::from)?;
    User::new(&mut txn, user.clone()).await?;
    let token = User::login(
        &mut txn,
        Login {
            email: user.email,
            password: user.password,
        },
        None,
    )
    .await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(token)
}

pub async fn login(
    pool: &'static Pool,
    user: Login,
    mac: Option<String>,
    file_hash: Option<String>,
) -> Result<impl Reply> {
    info!("Login: {:?}, {:?}, {:?}", user, mac, file_hash);
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let token = User::login(
        &mut txn,
        user,
        mac.and_then(|mac| file_hash.map(|file_hash| NewDevice { mac, file_hash })),
    )
    .await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(token)
}

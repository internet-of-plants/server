use crate::db::user::Login;
use crate::extractor::{MacAddress, Version};
use crate::{prelude::*, Device};
use crate::{NewDevice, NewUser, User};
use axum::extract::{Extension, Json, TypedHeader};
use controllers::Result;

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Json(user): Json<NewUser>,
) -> Result<impl IntoResponse> {
    // We should fix al the avenues for user abuse before allowing signups
    //return Result::<&'static str, _>::Err(Error::Forbidden.into());

    let mut txn = pool.begin().await?;
    User::new(&mut txn, user.clone()).await?;
    let token = User::login(
        &mut txn,
        Login {
            email: user.email,
            password: user.password,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(token.0)
}

pub async fn login(
    Extension(pool): Extension<&'static Pool>,
    Json(user): Json<Login>,
    mac: Option<TypedHeader<MacAddress>>,
    file_hash: Option<TypedHeader<Version>>,
) -> Result<impl IntoResponse> {
    info!("Login: {:?}, {:?}, {:?}", user.email, mac, file_hash);
    let mut txn = pool.begin().await?;
    let token = if let (Some(mac), Some(file_hash)) = (mac, file_hash) {
        let (mac, file_hash) = (mac.0 .0, file_hash.0 .0);
        Device::login(&mut txn, user, NewDevice { mac, file_hash }).await?
    } else {
        User::login(&mut txn, user).await?
    };
    txn.commit().await?;
    Ok(token.0)
}

use crate::{
    extractor::MacAddress, extractor::Version, logger::*, Device, Login, NewDevice, NewUser, Pool,
    Result, User,
};
use axum::{extract::Extension, extract::Json, extract::TypedHeader, response::IntoResponse};

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Json(user): Json<NewUser>,
) -> Result<impl IntoResponse> {
    // We should fix al the avenues for user abuse before allowing signups
    //return Result::<&'static str, _>::Err(Error::Unauthorized.into());

    let mut txn = pool.begin().await?;
    User::new(&mut txn, user.clone()).await?;
    let token = User::login(
        &mut txn,
        Login {
            email: user.email().to_owned(),
            password: user.password().to_owned(),
        },
    )
    .await?;
    txn.commit().await?;
    Ok(token)
}

pub async fn login(
    Extension(pool): Extension<&'static Pool>,
    Json(user): Json<Login>,
    mac: Option<TypedHeader<MacAddress>>,
    file_hash: Option<TypedHeader<Version>>,
) -> Result<impl IntoResponse> {
    info!("Login: {:?}, {:?}, {:?}", user.email(), mac, file_hash);
    let mut txn = pool.begin().await?;
    let token = if let (Some(mac), Some(file_hash)) = (mac, file_hash) {
        let (mac, file_hash) = (mac.0 .0, file_hash.0 .0);
        Device::login(&mut txn, user, NewDevice::new(mac, file_hash)).await?
    } else {
        User::login(&mut txn, user).await?
    };
    txn.commit().await?;
    Ok(token)
}

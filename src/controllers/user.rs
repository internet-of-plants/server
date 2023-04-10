use crate::{
    extractor::MacAddress, extractor::MaybeTargetPrototype, extractor::User, extractor::Version,
    logger::*, Device, Error, Login, NewDevice, NewUser, Pool, Result, UserView,
};
use axum::{extract::Extension, extract::Json, extract::TypedHeader, response::IntoResponse};

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Json(user): Json<NewUser>,
) -> Result<impl IntoResponse> {
    // We should fix al the avenues for user abuse before allowing signups
    //return Result::<&'static str, _>::Err(Error::Unauthorized.into());

    let mut txn = pool.begin().await?;
    crate::User::new(&mut txn, user.clone()).await?;
    let token = crate::User::login(
        &mut txn,
        Login {
            organization: Some(user.organization_name().to_owned()),
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
    MaybeTargetPrototype(maybe_target_prototype): MaybeTargetPrototype,
) -> Result<impl IntoResponse> {
    info!(
        "Login: {:?} - {}, {:?}, {:?}",
        user.organization(),
        user.email(),
        mac,
        file_hash
    );
    let mut txn = pool.begin().await?;
    let token = match (mac, file_hash, maybe_target_prototype) {
        (Some(mac), Some(file_hash), Some(target_prototype)) => {
            let (mac, file_hash) = (mac.0 .0, file_hash.0 .0);
            Device::login(
                &mut txn,
                user,
                NewDevice::new(mac, file_hash, target_prototype),
            )
            .await?
        }
        (None, None, None) => crate::User::login(&mut txn, user).await?,
        (_, _, None) => return Err(Error::MissingHeader("DRIVER")),
        (_, None, _) => return Err(Error::MissingHeader("VERSION")),
        (None, _, _) => return Err(Error::MissingHeader("MAC_ADDRESS")),
    };
    txn.commit().await?;
    Ok(token)
}

pub async fn find(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
) -> Result<Json<UserView>> {
    let mut txn = pool.begin().await?;
    let user = UserView::new(&mut txn, &user).await?;
    txn.commit().await?;

    Ok(Json(user))
}

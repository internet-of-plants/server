use crate::prelude::*;
use crate::{Organization, OrganizationId, OrganizationView};
use controllers::Result;

pub async fn find(
    organization_id: OrganizationId,
    pool: &'static Pool,
    _auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let organization = OrganizationView::find_by_id(&mut txn, &organization_id).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&organization))
}

pub async fn from_user(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let organizations = Organization::from_user(&mut txn, &auth.user_id).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&organizations))
}

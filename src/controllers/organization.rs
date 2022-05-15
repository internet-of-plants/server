use crate::prelude::*;
use crate::{Organization, OrganizationId, OrganizationView};
use controllers::Result;
use crate::extractor::Authorization;
use axum::extract::{Extension, Path, Json};

pub async fn find(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
    Path(organization_id): Path<OrganizationId>,
) -> Result<Json<OrganizationView>> {
    let mut txn = pool.begin().await?;
    let organization = OrganizationView::find_by_id(&mut txn, &organization_id).await?;
    txn.commit().await?;
    Ok(Json(organization))
}

pub async fn from_user(Extension(pool): Extension<&'static Pool>, Authorization(auth): Authorization) -> Result<Json<Vec<Organization>>> {
    let mut txn = pool.begin().await?;
    let organizations = Organization::from_user(&mut txn, &auth.user_id).await?;
    txn.commit().await?;
    Ok(Json(organizations))
}

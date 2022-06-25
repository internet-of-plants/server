use crate::extractor::User;
use crate::prelude::*;
use crate::{Organization, OrganizationId, OrganizationView};
use axum::extract::{Extension, Json, Query};
use controllers::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindRequest {
    organization_id: OrganizationId,
}

pub async fn find(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<FindRequest>,
) -> Result<Json<OrganizationView>> {
    let mut txn = pool.begin().await?;
    let organization = Organization::find_by_id(&mut txn, request.organization_id, &user).await?;
    let organization = OrganizationView::new(&mut txn, &organization).await?;
    txn.commit().await?;
    Ok(Json(organization))
}

pub async fn from_user(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
) -> Result<Json<Vec<OrganizationView>>> {
    let mut txn = pool.begin().await?;
    let organizations = Organization::from_user(&mut txn, &user).await?;
    let mut views = Vec::with_capacity(organizations.len());
    for organization in organizations {
        views.push(OrganizationView::new(&mut txn, &organization).await?);
    }
    txn.commit().await?;
    Ok(Json(views))
}

use crate::{extractor::User, Organization, OrganizationView, Pool, Result};
use axum::extract::{Extension, Json};

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

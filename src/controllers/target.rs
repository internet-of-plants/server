use crate::db::target::{Target, TargetView};
use crate::extractor::User;
use crate::prelude::*;
use axum::extract::{Extension, Json};
use controllers::Result;

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    User(_user): User,
) -> Result<Json<Vec<TargetView>>> {
    let mut txn = pool.begin().await?;
    let targets = Target::list(&mut txn).await?;
    let mut views = Vec::with_capacity(targets.len());
    for target in targets {
        views.push(TargetView::new(&mut txn, target).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

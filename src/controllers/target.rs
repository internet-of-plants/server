use crate::{extractor::User, Pool, Result, Target, TargetView};
use axum::extract::{Extension, Json};

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

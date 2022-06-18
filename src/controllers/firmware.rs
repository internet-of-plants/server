use crate::extractor::Authorization;
use crate::{
    db::firmware::{Firmware, FirmwareId},
    prelude::*,
};
use axum::extract::{Extension, Json};
use controllers::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareView {
    id: FirmwareId,
    //compilation: CompilationView,
    hash: String,
}

impl FirmwareView {
    pub async fn new(_txn: &mut Transaction<'_>, firmware: Firmware) -> Result<Self> {
        //if let Some(compilation) = firmware.compilation(txn).await? {
        Ok(Self {
            id: firmware.id(),
            //compilation: CompilationView::new(txn, compilation).await?,
            hash: firmware.hash().to_owned(),
        })
        //} else {
        //    Ok(None)
        //}
    }
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<impl IntoResponse> {
    let mut txn = pool.begin().await?;
    let firmwares = Firmware::list(&mut txn).await?;
    let mut firmwares_view = Vec::with_capacity(firmwares.len());
    for firmware in firmwares {
        if firmware.binary().is_none() {
            continue;
        }

        firmwares_view.push(FirmwareView::new(&mut txn, firmware).await?);
    }

    txn.commit().await?;
    Ok(Json(firmwares_view))
}

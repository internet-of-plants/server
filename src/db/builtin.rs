use crate::{
    logger::*, NewSensorPrototype, NewTarget, NewTargetPrototype, Result, SensorPrototype, Target,
    TargetPrototype, Transaction,
};
use tokio::fs;

// TODO: add propper pinnage to esp32
// TODO: we should configure the analog pin in soil resistivity
pub async fn create_builtins(txn: &mut Transaction<'_>) -> Result<()> {
    target_prototypes(txn).await?;
    sensor_prototypes(txn).await?;

    Ok(())
}

pub async fn target_prototypes(txn: &mut Transaction<'_>) -> Result<()> {
    let mut reader = match fs::read_dir("packages/target_prototypes").await {
        Ok(reader) => reader,
        Err(err) => {
            warn!("Unable to open packages/target_prototypes: {err}");
            return Ok(());
        }
    };
    while let Some(entry) = reader.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            error!("Invalid package, must be a folder");
            continue;
        }

        let mut target_prototypes = Vec::new();

        let mut reader = fs::read_dir(entry.path()).await?;
        while let Some(entry) = reader.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if entry.file_name() != "targets" {
                    error!(
                        "Found folder that is not `targets`: {}",
                        entry.file_name().to_string_lossy()
                    );
                    continue;
                }
            } else if entry.file_type().await?.is_file() {
                let json = fs::read_to_string(entry.path()).await?;
                let target_prototype: NewTargetPrototype = match serde_json::from_str(&json) {
                    Ok(prototype) => prototype,
                    Err(err) => {
                        error!("Unable to deserialize {}: {err}", entry.path().display());
                        continue;
                    }
                };
                target_prototypes.push(TargetPrototype::new(txn, target_prototype).await?);
            } else if entry.file_type().await?.is_symlink() {
                error!("Symlinks are not supported as packages");
                break;
            }
        }

        let mut reader = fs::read_dir(entry.path().join("targets")).await?;
        while let Some(entry) = reader.next_entry().await? {
            if !entry.file_type().await?.is_file() {
                error!("Invalid target, must be a file");
                continue;
            }
            let json = fs::read_to_string(entry.path()).await?;
            let target: NewTarget = match serde_json::from_str(&json) {
                Ok(target) => target,
                Err(err) => {
                    error!("Unable to deserialize {}: {err}", entry.path().display());
                    continue;
                }
            };
            if !target_prototypes
                .iter()
                .any(|p| target.target_prototype_arch() == p.arch())
            {
                error!(
                    "Target prototype for {} doesn't exist",
                    entry.path().display()
                );
                continue;
            }
            Target::new(txn, target).await?;
        }
    }
    Ok(())
}

pub async fn sensor_prototypes(txn: &mut Transaction<'_>) -> Result<()> {
    let mut reader = match fs::read_dir("packages/sensor_prototypes").await {
        Ok(reader) => reader,
        Err(err) => {
            warn!("Unable to open packages/sensor_prototypes: {err}");
            return Ok(());
        }
    };
    while let Some(entry) = reader.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            error!("Invalid sensor prototype, must be a file");
            continue;
        }
        let json = fs::read_to_string(entry.path()).await?;
        let sensor_prototype: NewSensorPrototype = match serde_json::from_str(&json) {
            Ok(target) => target,
            Err(err) => {
                error!("Unable to deserialize {}: {err}", entry.path().display());
                continue;
            }
        };
        SensorPrototype::new(txn, sensor_prototype).await?;
    }
    Ok(())
}

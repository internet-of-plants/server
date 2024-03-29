use crate::{
    utils, AuthToken, Collection, CollectionId, CompilerView, DateTime, Error, Event, EventView,
    Firmware, FirmwareId, FirmwareView, Login, Organization, Result, TargetPrototype,
    TargetPrototypeId, Transaction, User, UserId,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[id]
pub struct DeviceId;

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    #[copy]
    id: DeviceId,
    name: String,
    description: Option<String>,
    mac: String,
    target_prototype: TargetPrototype,
    firmware: FirmwareView,
    compiler: Option<CompilerView>,
    last_event: Option<EventView>,
    #[copy]
    created_at: DateTime,
    #[copy]
    updated_at: DateTime,
}

impl DeviceView {
    pub async fn new(txn: &mut Transaction<'_>, device: Device) -> Result<Self> {
        let firmware = device.current_firmware(txn).await?;
        let firmware = FirmwareView::new(firmware);
        let collection = device.collection(txn).await?;
        let compiler = collection.compiler(txn).await?;
        let compiler = match compiler {
            Some(c) => Some(CompilerView::new(txn, c).await?),
            None => None,
        };

        let last_event = device.last_event(txn).await?;
        let last_event = if let Some(last_event) = last_event {
            Some(EventView::new(last_event)?)
        } else {
            None
        };
        Ok(DeviceView {
            id: device.id,
            target_prototype: device.target_prototype(txn).await?,
            name: device.name,
            description: device.description,
            firmware,
            mac: device.mac,
            compiler,
            last_event,
            created_at: device.created_at,
            updated_at: device.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Device {
    #[copy]
    id: DeviceId,
    #[copy]
    collection_id: CollectionId,
    #[copy]
    firmware_id: FirmwareId,
    name: String,
    description: Option<String>,
    mac: String,
    #[copy]
    target_prototype_id: TargetPrototypeId,
    #[copy]
    created_at: DateTime,
    #[copy]
    updated_at: DateTime,
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewDevice {
    mac: String,
    file_hash: String,
    target_prototype: TargetPrototype,
}

impl NewDevice {
    pub fn new(mac: String, file_hash: String, target_prototype: TargetPrototype) -> Self {
        Self {
            mac,
            file_hash,
            target_prototype,
        }
    }
}

impl Device {
    pub async fn put(
        txn: &mut Transaction<'_>,
        organization: &Organization,
        new_device: NewDevice,
    ) -> Result<Self> {
        let device = Self::try_find_by_mac(txn, organization, &new_device.mac).await?;

        // TODO: add target prototype check here
        let firmware = if let Some(firmware) =
            Firmware::try_find_by_hash(txn, organization, &new_device.file_hash).await?
        {
            firmware
        } else {
            Firmware::new_unknown(txn, new_device.file_hash.to_lowercase(), organization).await?
        };

        let compiler = if let Some(compilation) = firmware.compilation(txn).await? {
            let compiler = compilation.compiler(txn).await?;
            let target = compiler.target(txn).await?;

            if target.target_prototype_id() != new_device.target_prototype.id() {
                return Err(Error::WrongTargetPrototype(
                    target.target_prototype_id(),
                    new_device.target_prototype.id(),
                ));
            }

            Some(compiler)
        } else {
            None
        };

        if let Some(mut device) = device {
            if device.target_prototype_id != new_device.target_prototype.id() {
                return Err(Error::WrongTargetPrototype(
                    device.target_prototype_id,
                    new_device.target_prototype.id(),
                ));
            }

            let mut collection = device.collection(txn).await?;

            if collection.compiler(txn).await?.is_none() {
                if let Some(compiler) = compiler {
                    if let Some(col) = Collection::find_by_compiler(txn, &compiler).await? {
                        if col.target_prototype_id() != device.target_prototype_id() {
                            return Err(Error::WrongTargetPrototype(
                                col.target_prototype_id(),
                                device.target_prototype_id(),
                            ));
                        }

                        device.set_collection(txn, &col).await?;
                    } else {
                        let target = compiler.target(txn).await?;
                        let prototype = target.prototype(txn).await?;
                        if collection.target_prototype_id() != prototype.id() {
                            return Err(Error::WrongTargetPrototype(
                                collection.target_prototype_id(),
                                prototype.id(),
                            ));
                        }
                        collection.set_compiler(txn, Some(&compiler)).await?;
                    }
                } else {
                    // Assume all devices with the same firmware are of the same collection, a race might make this not true, but let's pick one
                    if let Some(dev) = crate::Device::list_by_firmware(txn, &firmware, organization)
                        .await?
                        .pop()
                    {
                        let col = dev.collection(txn).await?;

                        if col.target_prototype_id() != device.target_prototype_id() {
                            return Err(Error::WrongTargetPrototype(
                                col.target_prototype_id(),
                                device.target_prototype_id(),
                            ));
                        }
                        device.set_collection(txn, &col).await?;
                    }
                }
            }

            if device.firmware_id != firmware.id() {
                device.set_firmware(txn, &firmware).await?;
            }
            return Ok(device);
        }

        let name = format!("device-{}", utils::random_string(5));
        let mut collection = Collection::new(
            txn,
            name.clone(),
            new_device.target_prototype.id(),
            organization,
        )
        .await?;

        if device.is_none() {
            if let Some(compiler) = &compiler {
                let target = compiler.target(txn).await?;
                let prototype = target.prototype(txn).await?;
                if collection.target_prototype_id() != prototype.id() {
                    return Err(Error::WrongTargetPrototype(
                        collection.target_prototype_id(),
                        prototype.id(),
                    ));
                }
            }
            collection.set_compiler(txn, compiler.as_ref()).await?;
        }

        let (id, now) =
            sqlx::query_as::<_, (DeviceId, DateTime)>("INSERT INTO devices (mac, collection_id, name, firmware_id, target_prototype_id) VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at")
                .bind(new_device.mac())
                .bind(collection.id())
                .bind(&name)
                .bind(firmware.id())
                .bind(new_device.target_prototype.id())
                .fetch_one(&mut *txn)
                .await?;
        Ok(Self {
            id,
            collection_id: collection.id(),
            firmware_id: firmware.id(),
            target_prototype_id: new_device.target_prototype.id(),
            name,
            description: None,
            mac: new_device.mac,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn try_find_by_mac(
        txn: &mut Transaction<'_>,
        organization: &Organization,
        mac: &str,
    ) -> Result<Option<Self>> {
        let device: Option<Self> = sqlx::query_as(
            "SELECT dev.id, dev.target_prototype_id, dev.collection_id, dev.firmware_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             INNER JOIN collection_belongs_to_organization as cbt ON cbt.collection_id = dev.collection_id
             INNER JOIN user_belongs_to_organization ubt ON ubt.organization_id = cbt.organization_id
             WHERE dev.mac = $1 AND cbt.organization_id = $2",
        )
            .bind(mac)
            .bind(organization.id())
            .fetch_optional(txn)
            .await?;
        Ok(device)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        device_id: DeviceId,
        user: &User,
    ) -> Result<Self> {
        let device: Self = sqlx::query_as(
            "SELECT dev.id, dev.target_prototype_id, dev.collection_id, dev.firmware_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             INNER JOIN collection_belongs_to_organization as cbt ON cbt.collection_id = dev.collection_id
             INNER JOIN user_belongs_to_organization ubt ON ubt.organization_id = cbt.organization_id
             WHERE dev.id = $1 AND ubt.user_id = $2",
        )
            .bind(device_id)
            .bind(user.id())
            .fetch_one(txn)
            .await?;
        Ok(device)
    }

    pub async fn from_collection(
        txn: &mut Transaction<'_>,
        collection: &Collection,
    ) -> Result<Vec<Self>> {
        let devices: Vec<Self> = sqlx::query_as(
            "SELECT dev.id, dev.target_prototype_id, dev.collection_id, dev.firmware_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             INNER JOIN collection_belongs_to_organization as cbt ON cbt.collection_id = dev.collection_id
             WHERE dev.collection_id = $1",
        )
            .bind(collection.id())
            .fetch_all(txn)
            .await?;
        Ok(devices)
    }

    pub async fn find_by_auth_token(
        txn: &mut Transaction<'_>,
        token: AuthToken,
        mac: String,
    ) -> Result<Self> {
        let device: Option<Self> = sqlx::query_as(
            "SELECT dev.id, dev.target_prototype_id, dev.collection_id, dev.firmware_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             INNER JOIN authentications ON authentications.device_id = dev.id AND authentications.mac = dev.mac AND authentications.expired = false
             WHERE authentications.token = $1 AND authentications.mac = $2",
        )
        .bind(&token)
        .bind(&mac)
        .fetch_optional(&mut *txn)
        .await?;
        device.ok_or(Error::Unauthorized)
    }

    pub async fn list_by_firmware(
        txn: &mut Transaction<'_>,
        firmware: &Firmware,
        organization: &Organization,
    ) -> Result<Vec<Self>> {
        let device = sqlx::query_as(
            "SELECT dev.id, dev.target_prototype_id, dev.collection_id, dev.firmware_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             INNER JOIN collection_belongs_to_organization as cbt ON cbt.collection_id = dev.collection_id
             WHERE cbt.organization_id = $1 AND dev.firmware_id = $2"
        )
        .bind(organization.id())
        .bind(firmware.id())
        .fetch_all(&mut *txn)
        .await?;
        Ok(device)
    }

    pub async fn login(
        txn: &mut Transaction<'_>,
        client: Login,
        new_device: NewDevice,
    ) -> Result<AuthToken> {
        let hash: Option<(UserId, String, String, DateTime, DateTime, String)> = sqlx::query_as(
            "SELECT users.id, users.email, users.username, users.created_at, users.updated_at, users.password_hash
            FROM users
            INNER JOIN user_belongs_to_organization bt ON bt.user_id = users.id
            INNER JOIN organizations on organizations.id = bt.organization_id
            WHERE users.email = $1 AND organizations.name = $2",
        )
        .bind(client.email())
        .bind(client.organization())
        .fetch_optional(&mut *txn)
        .await?;
        let is_auth = match &hash {
            Some((_, _, _, _, _, hash)) => utils::verify_password(client.password(), hash)?,
            // Avoids timing attacks to detect usernames
            None => utils::hash_password(client.password())? == "abc",
        };

        match (hash, is_auth) {
            (Some((id, email, username, created_at, updated_at, _)), true) => {
                let organization_name = client
                    .organization()
                    .as_ref()
                    .expect("organization should be available");
                let user = User {
                    id,
                    email,
                    username,
                    created_at,
                    updated_at,
                };

                let organization =
                    Organization::find_by_name(txn, organization_name, &user).await?;
                let device = Device::put(txn, &organization, new_device).await?;
                let token = AuthToken::random();

                sqlx::query("UPDATE authentications SET expired = true WHERE mac = $1")
                    .bind(&device.mac)
                    .execute(&mut *txn)
                    .await?;

                sqlx::query(
                    "INSERT INTO authentications (mac, device_id, token) VALUES ($1, $2, $3)",
                )
                .bind(&device.mac)
                .bind(device.id())
                .bind(&token)
                .execute(&mut *txn)
                .await?;
                Ok(token)
            }
            _ => Err(Error::Unauthorized),
        }
    }

    pub async fn update_collection(
        txn: &mut Transaction<'_>,
        id: DeviceId,
        collection: &Collection,
    ) -> Result<DateTime> {
        let (updated_at,): (DateTime,) = sqlx::query_as(
            "UPDATE devices SET collection_id = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at",
        )
        .bind(collection.id())
        .bind(id)
        .fetch_one(&mut *txn)
        .await?;

        Ok(updated_at)
    }

    pub async fn set_collection(
        &mut self,
        txn: &mut Transaction<'_>,
        collection: &Collection,
    ) -> Result<()> {
        let old_collection = self.collection(txn).await?;

        let updated_at = Self::update_collection(txn, self.id, collection).await?;

        if old_collection.devices(txn).await?.is_empty() {
            // TODO FIXME: race
            old_collection.delete(txn).await?;
        }

        self.updated_at = updated_at;
        self.collection_id = collection.id();
        Ok(())
    }

    pub async fn set_name(&mut self, txn: &mut Transaction<'_>, name: String) -> Result<()> {
        let (updated_at,): (DateTime,) = sqlx::query_as(
            "UPDATE devices SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at",
        )
        .bind(&name)
        .bind(self.id)
        .fetch_one(txn)
        .await?;
        self.updated_at = updated_at;
        self.name = name;
        Ok(())
    }

    pub async fn set_firmware(
        &mut self,
        txn: &mut Transaction<'_>,
        firmware: &Firmware,
    ) -> Result<()> {
        let (updated_at,): (DateTime,) = sqlx::query_as("UPDATE devices SET firmware_id = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at")
            .bind(firmware.id())
            .bind(self.id)
            .fetch_one(txn)
            .await?;
        self.updated_at = updated_at;
        self.firmware_id = firmware.id();
        Ok(())
    }

    pub async fn current_firmware(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        Firmware::find_by_device(txn, self).await
    }

    pub async fn last_event(&self, txn: &mut Transaction<'_>) -> Result<Option<Event>> {
        Event::last_from_device(txn, self).await
    }

    pub async fn collection(&self, txn: &mut Transaction<'_>) -> Result<Collection> {
        Collection::find_by_device(txn, self).await
    }

    pub async fn target_prototype(&self, txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
        TargetPrototype::find_by_id(txn, self.target_prototype_id).await
    }
}

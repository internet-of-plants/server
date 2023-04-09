use crate::{AuthToken, Pool};
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    http::StatusCode,
};

pub struct Device(pub super::Device);

#[async_trait]
impl<B> FromRequest<B> for Device
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<&'static Pool>::from_request(req)
            .await
            .expect("`Pool` extension missing");

        let mut token = TypedHeader::<AuthorizationHeader>::from_request(req)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "No auth token"))?;
        let mac = match TypedHeader::<MacAddress>::from_request(req).await {
            Ok(mac) => mac.0 .0,
            Err(_) => {
                TypedHeader::<Esp8266StaMac>::from_request(req)
                    .await
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "No mac address"))?
                    .0
                     .0
            }
        };
        if token.0 .0.starts_with("Basic ") {
            token.0 .0.drain(.."Basic ".len());
            let mut txn = pool
                .begin()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            let device =
                crate::Device::find_by_auth_token(&mut txn, AuthToken::new(token.0 .0), mac)
                    .await
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "Device not found"))?;
            txn.commit()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            Ok(Self(device))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid authorization header"))
        }
    }
}

pub struct User(pub super::User);

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<&'static Pool>::from_request(req)
            .await
            .expect("`Pool` extension missing");

        let mut token = TypedHeader::<AuthorizationHeader>::from_request(req)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "No authorization"))?;
        if token.0 .0.starts_with("Basic ") {
            token.0 .0.drain(.."Basic ".len());
            let mut txn = pool
                .begin()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            let user = crate::User::find_by_auth_token(&mut txn, AuthToken::new(token.0 .0))
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Internal Server Error"))?;
            txn.commit()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            Ok(Self(user))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid authorization"))
        }
    }
}

pub struct MaybeTargetPrototype(pub Option<super::TargetPrototype>);

#[async_trait]
impl<B> FromRequest<B> for MaybeTargetPrototype
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<&'static Pool>::from_request(req)
            .await
            .expect("`Pool` extension missing");

        let driver = TypedHeader::<Driver>::from_request(req).await.ok();

        if let Some(driver) = driver {
            let mut txn = pool
                .begin()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;

            let maybe_target_prototype =
                crate::TargetPrototype::try_find_by_arch(&mut txn, &driver.0 .0.to_lowercase())
                    .await
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "Device not found"))?;
            Ok(Self(maybe_target_prototype))
        } else {
            Ok(Self(None))
        }
    }
}

#[derive(Debug)]
struct AuthorizationHeader(String);

const AUTHORIZATION_NAME: &str = "authorization";
impl headers_core::Header for AuthorizationHeader {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
        static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(AUTHORIZATION_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct Esp8266StaMac(pub String);

impl From<&Esp8266StaMac> for headers_core::HeaderValue {
    fn from(mac: &Esp8266StaMac) -> Self {
        Self::from_str(&mac.0).expect("unable to convert esp8266 sta mac address to header value")
    }
}

#[derive(Debug)]
pub struct MacAddress(pub String);

impl From<&MacAddress> for headers_core::HeaderValue {
    fn from(mac: &MacAddress) -> Self {
        Self::from_str(&mac.0).expect("unable to convert mac address to header value")
    }
}

const MAC_ADDRESS_NAME: &str = "mac_address";
impl headers_core::Header for MacAddress {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(MAC_ADDRESS_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct Version(pub String);

impl From<&Version> for headers_core::HeaderValue {
    fn from(version: &Version) -> Self {
        Self::from_str(&version.0).expect("unable to convert version to header value")
    }
}

const VERSION_NAME: &str = "version";
impl headers_core::Header for Version {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(VERSION_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct Driver(pub String);

impl From<&Driver> for headers_core::HeaderValue {
    fn from(driver: &Driver) -> Self {
        Self::from_str(&driver.0).expect("unable to convert driver to header value")
    }
}

const DRIVER_NAME: &str = "driver";
impl headers_core::Header for Driver {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(DRIVER_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct TimeRunning(pub String);

const TIME_RUNNING_NAME: &str = "time_running";
impl headers_core::Header for TimeRunning {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(TIME_RUNNING_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct Vcc(pub String);

const VCC_NAME: &str = "vcc";
impl headers_core::Header for Vcc {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(VCC_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct FreeDram(pub String);

const FREE_DRAM_NAME: &str = "free_dram";
impl headers_core::Header for FreeDram {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(FREE_DRAM_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct BiggestDramBlock(pub String);

const BIGGEST_DRAM_BLOCK_NAME: &str = "biggest_block_dram";
impl headers_core::Header for BiggestDramBlock {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(BIGGEST_DRAM_BLOCK_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct FreeStack(pub String);

const FREE_STACK_NAME: &str = "free_stack";
impl headers_core::Header for FreeStack {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(FREE_STACK_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct Esp8266Md5(pub String);

const ESP8266_MD5_NAME: &str = "x-esp8266-sketch-md5";
impl headers_core::Header for Esp8266Md5 {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(ESP8266_MD5_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

const ESP8266_STA_MAC_NAME: &str = "x-esp8266-sta-mac";
impl headers_core::Header for Esp8266StaMac {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(ESP8266_STA_MAC_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct FreeIram(pub String);

const FREE_IRAM_NAME: &str = "free_iram";
impl headers_core::Header for FreeIram {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(FREE_IRAM_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

#[derive(Debug)]
pub struct BiggestIramBlock(pub String);

const BIGGEST_IRAM_BLOCK_NAME: &str = "biggest_block_iram";
impl headers_core::Header for BiggestIramBlock {
    fn name() -> &'static headers_core::HeaderName {
        thread_local! {
            static NAME: &'static headers_core::HeaderName = Box::leak(headers_core::HeaderName::from_static(BIGGEST_IRAM_BLOCK_NAME).into());
        }
        NAME.with(|n| *n)
    }

    fn decode<'i, I: Iterator<Item = &'i headers_core::HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers_core::Error> {
        values
            .next()
            .and_then(|val| val.to_str().ok().map(|v| Self(v.to_owned())))
            .ok_or_else(headers_core::Error::invalid)
    }

    fn encode<E: Extend<headers_core::HeaderValue>>(&self, values: &mut E) {
        if let Ok(name) = headers_core::HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(name));
        }
    }
}

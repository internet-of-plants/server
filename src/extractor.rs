use crate::db::user::{AuthToken, User};
use crate::prelude::*;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    http::StatusCode,
};

pub struct Authorization(pub Auth);

#[async_trait]
impl<B> FromRequest<B> for Authorization
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
            // TODO: we should check if the MAC_ADDRESS header is the same as in the db
            // TODO: we could check for updates here, but we don't want to lose the
            // payload, think about a middleware (although it's unclear what to do with
            // failures)
            let mut txn = pool
                .begin()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            let auth = User::find_by_auth_token(&mut txn, AuthToken::new(token.0 .0))
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            txn.commit()
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
            Ok(Authorization(auth))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid authorization"))
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
pub struct MacAddress(pub String);

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

const BIGGEST_DRAM_BLOCK_NAME: &str = "biggest_dram_block";
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

const BIGGEST_IRAM_BLOCK_NAME: &str = "biggest_iram_block";
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
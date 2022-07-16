use crate::{
    controllers::{device::SetNameRequest, sensor::{SetAliasRequest, SetColorRequest}},
    extractor::MacAddress,
    extractor::Version,
    AuthToken, CollectionId, CollectionView, CompilationView, DeviceId, DeviceLogView,
    DevicePanicView, DeviceView, Login, NewCompiler, NewDevicePanic, NewUser, OrganizationId,
    OrganizationView, SensorPrototypeView, TargetId, TargetView,
};
use axum::{body::Body, http, http::Method, http::Request, http::StatusCode, Router};
use tower::ServiceExt;

pub async fn signup(app: Router, new_user: NewUser) -> AuthToken {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/user")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(serde_json::to_vec(&new_user).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let token = String::from_utf8(
        hyper::body::to_bytes(response.into_body())
            .await
            .unwrap()
            .as_ref()
            .to_owned(),
    )
    .unwrap();
    AuthToken(token)
}

pub async fn login(
    app: Router,
    login: Login,
    mac_address: Option<String>,
    version: Option<String>,
) -> AuthToken {
    let request = Request::builder()
        .uri("/v1/user/login")
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref());

    let request = if let Some(version) = version {
        request.header("VERSION", version)
    } else {
        request
    };
    let request = if let Some(mac) = mac_address {
        request.header("MAC_ADDRESS", mac)
    } else {
        request
    };
    let request = request
        .method(Method::POST)
        .body(Body::from(serde_json::to_vec(&login).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let token = String::from_utf8(
        hyper::body::to_bytes(response.into_body())
            .await
            .unwrap()
            .as_ref()
            .to_owned(),
    )
    .unwrap();
    AuthToken(token)
}

pub async fn list_organizations(app: Router, token: &AuthToken) -> Vec<OrganizationView> {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/organizations")
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn find_organization(
    app: Router,
    token: &AuthToken,
    id: OrganizationId,
) -> OrganizationView {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/organization?organizationId={}", id.0))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn find_collection(
    app: Router,
    token: &AuthToken,
    collection_id: CollectionId,
) -> CollectionView {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/collection?collectionId={}", collection_id.0))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn send_device_log(
    app: Router,
    token: &AuthToken,
    mac_address: &str,
    version: &str,
    log: &str,
) {
    let body = Body::from(log.to_owned());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/log")
                .header("Authorization", format!("Basic {}", token.0))
                .header("MAC_ADDRESS", mac_address)
                .header("VERSION", version)
                .method(Method::POST)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

pub async fn set_device_name(app: Router, token: &AuthToken, device_id: DeviceId, name: String) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/device/name")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&SetNameRequest { device_id, name }).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

pub async fn find_device(app: Router, token: &AuthToken, device_id: DeviceId) -> DeviceView {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/device?deviceId={}", device_id.0,))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn list_device_logs(
    app: Router,
    token: &AuthToken,
    device_id: DeviceId,
) -> Vec<DeviceLogView> {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/device/logs?deviceId={}&limit={}",
                    device_id.0, 10
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn send_device_panic(
    app: Router,
    token: &AuthToken,
    mac_address: &str,
    version: &str,
    panic: &NewDevicePanic,
) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/panic")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .header("MAC_ADDRESS", mac_address)
                .header("VERSION", version)
                .method(Method::POST)
                .body(Body::from(serde_json::to_vec(&panic).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

pub async fn list_device_panics(
    app: Router,
    token: &AuthToken,
    device_id: DeviceId,
) -> Vec<DevicePanicView> {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/device/panics?deviceId={}&limit={}",
                    device_id.0, 10
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn send_event(
    app: Router,
    token: &AuthToken,
    version: &Version,
    mac_address: &MacAddress,
    new_event: &serde_json::Value,
) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/event")
                .header("Authorization", format!("Basic {}", token.0))
                .header("MAC_ADDRESS", &mac_address.0)
                .header("VERSION", &version.0)
                .header("TIME_RUNNING", "1")
                .header("VCC", "1")
                .header("FREE_STACK", "10000")
                .header("FREE_DRAM", "10000")
                .header("BIGGEST_BLOCK_DRAM", "10000")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(serde_json::to_vec(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/*
pub async fn send_update(
    app: Router,
    token: &AuthToken,
    organization_id: OrganizationId,
    collection_id: CollectionId,
    device_id: DeviceId,
    update: NewUpdate,
) {
    let NewUpdate { file, version } = update;
    let mut body = format!(
        "-----------------------------152619935231652215881740279177\r
Content-Disposition: form-data; name=\"version\"\r
\r
{}\r
-----------------------------152619935231652215881740279177\r
Content-Disposition: form-data; name=\"binary\"; filename=\"firmware.bin\"\r
Content-Type: application/octet-stream\r
\r
",
        version
    )
    .into_bytes();
    body.extend(file);
    body.extend("\r\n-----------------------------152619935231652215881740279177--".as_bytes());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/organization/{}/collection/{}/device/{}/update",
                    organization_id.0, collection_id.0, device_id.0
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .header(
                    http::header::CONTENT_LENGTH,
                    body.len().to_string()
                )
                .header(
                    http::header::CONTENT_TYPE,
                    "multipart/form-data; boundary=---------------------------152619935231652215881740279177"
                )
                .method(Method::POST)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

pub async fn find_update(app: Router, token: &AuthToken, file_hash: &Version) -> Vec<u8> {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/update")
                .header("Authorization", format!("Basic {}", token.0))
                .header("x-ESP8266-sketch-md5", &file_hash.0)
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    body.as_ref().to_owned()
}
*/

pub async fn list_targets(app: Router, token: &AuthToken) -> Vec<TargetView> {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/targets")
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn list_sensor_prototypes(
    app: Router,
    token: &AuthToken,
    target_id: TargetId,
) -> Vec<SensorPrototypeView> {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/target/sensor/prototypes?targetId={}",
                    target_id.0
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn create_compiler(
    app: Router,
    token: &AuthToken,
    compiler: NewCompiler,
) -> CompilationView {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/compiler")
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::POST)
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&compiler).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn set_sensor_alias(app: Router, token: &AuthToken, request: SetAliasRequest) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/sensor/alias")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(serde_json::to_vec(dbg!(&request)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

pub async fn set_sensor_color(app: Router, token: &AuthToken, request: SetColorRequest) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/sensor/color")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(serde_json::to_vec(dbg!(&request)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

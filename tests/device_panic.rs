use axum::{body::Body, http::Method, http::Request, http::StatusCode};
use server::test_helpers::{
    list_device_panics, list_organizations, login, send_device_panic, signup,
};
use server::NewDevicePanic;
use server::{db::user::Login, db::user::NewUser, test_router};
use tower::ServiceExt;

#[tokio::test]
async fn device_panic() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão4@example.com".to_owned(),
            username: "bobão4".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão4".to_owned(),
        },
    )
    .await;

    let token_device1 = login(
        app.clone(),
        Login {
            email: "bobão4@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("aaaa".to_owned()),
        Some("bbba".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let device_id1 = orgs[0].collections[0].devices[0].id();

    let panics = list_device_panics(app.clone(), &token, device_id1).await;
    assert_eq!(panics.len(), 0);

    let panic = "my panicgy panic panicger";
    send_device_panic(
        app.clone(),
        &token_device1,
        "aaaa",
        "bbbb",
        &NewDevicePanic {
            file: "myfile.cpp".to_owned(),
            line: 32,
            func: "myfunc()".to_owned(),
            msg: panic.to_owned(),
        },
    )
    .await;

    let panics = list_device_panics(app.clone(), &token, device_id1).await;
    assert_eq!(panics.len(), 1);
    assert_eq!(panics[0].msg(), panic);

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão5@example.com".to_owned(),
            username: "bobão5".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão5".to_owned(),
        },
    )
    .await;

    let token_device = login(
        app.clone(),
        Login {
            email: "bobão5@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("ddd".to_owned()),
        Some("ccc".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let device_id = orgs[0].collections[0].devices[0].id();

    let panics = list_device_panics(app.clone(), &token, device_id).await;
    assert_eq!(panics.len(), 0);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/device/panics?deviceId={}&limit={}",
                    device_id1.0, 10
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/device/panics?deviceId={}&limit={}",
                    device_id.0, 10001
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/device/panics?deviceId={}&limit={}",
                    device_id.0, 10
                ))
                .header("Authorization", format!("Basic {}", token_device.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = Body::from(panic.to_owned());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/panic")
                .header("Authorization", format!("Basic {}", token.0))
                .header("MAC_ADDRESS", "ddd")
                .header("VERSION", "ccc")
                .method(Method::POST)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

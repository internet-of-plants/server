// TODO: test compiler/firmware/last_event field
use axum::{body::Body, http, http::Method, http::Request, http::StatusCode};
use server::controllers::device::SetNameRequest;
use server::test_helpers::{find_device, list_organizations, login, set_device_name, signup};
use server::{db::user::Login, db::user::NewUser, test_router};
use tower::ServiceExt;

#[tokio::test]
async fn device() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão6@example.com".to_owned(),
            username: "bobão6".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão6".to_owned(),
        },
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão6@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("aaaaaa".to_owned()),
        Some("bbbbbb".to_owned()),
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão6@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("gggggg".to_owned()),
        Some("hhhhhh".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;

    assert_eq!(orgs[0].collections.len(), 2);
    let collection = orgs[0].collections.iter().next().unwrap();

    let dev1 = find_device(app.clone(), &token, collection.devices[0].id()).await;
    assert_eq!(dev1.id, collection.devices[0].id());

    set_device_name(app.clone(), &token, dev1.id(), "new-name-1".to_owned()).await;

    let dev1_again = find_device(app.clone(), &token, collection.devices[0].id()).await;
    assert_eq!(dev1_again.name(), "new-name-1");
    assert_ne!(dev1.name(), dev1_again.name());
    assert_ne!(dev1, dev1_again);

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão7@example.com".to_owned(),
            username: "bobão7".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão7".to_owned(),
        },
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão7@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("eeeeee".to_owned()),
        Some("ffffff".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;

    assert_eq!(orgs[0].collections.len(), 1);
    let collection = orgs[0].collections.iter().next().unwrap();

    let dev = find_device(app.clone(), &token, collection.devices[0].id()).await;
    assert_eq!(dev.id, collection.devices[0].id());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/device?deviceId={}", dev1.id().0))
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/device/name")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&SetNameRequest {
                        device_id: dev1.id(),
                        name: "other-name1".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

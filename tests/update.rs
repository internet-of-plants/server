use server::test_helpers::{find_collection, find_organization, list_organizations, login, signup, find_device, send_update, find_update};
use server::{db::user::Login, db::user::NewUser, test_router, controllers::update::NewUpdate};
use server::extractor::{MacAddress, Version};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use std::fmt::Write;
use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
async fn update() {
    let app = test_router().await;

    signup(
        app.clone(),
        NewUser {
            email: "bobão8@example.com".to_owned(),
            username: "bobão8".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let mac_address = MacAddress("aaaaaaaa".to_owned());
    let version = Version("bbbbbbbb".to_owned());
    let token = login(
        app.clone(),
        Login {
            email: "bobão8@example.com".to_owned(),
            password: "bobão".to_owned(),
        },
        Some(mac_address.0.clone()),
        Some(version.0.clone()),
    )
    .await;
    
    let orgs = list_organizations(app.clone(), &token).await;
    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;

    assert_eq!(org.collections.len(), 1);
    let collection = org.collections.into_iter().next().unwrap();

    let col = find_collection(app.clone(), &token, org.id, *collection.id()).await;
    assert_eq!(col.id, *collection.id());
    assert_eq!(col.devices.len(), 1);
    
    let dev = find_device(app.clone(), &token, org.id, *collection.id(), *col.devices[0].id()).await;
    assert_eq!(dev.id, *col.devices[0].id());

    // No update was created
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(
                    "/v1/update",
                )
                .header("Authorization", format!("Basic {}", token.0))
                .header("x-ESP8266-sketch-md5", "abc")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

    let file = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let md5 = md5::compute(&file);
    let md5 = &*md5;
    let mut file_hash = String::with_capacity(md5.len() * 2);
    for byte in md5 {
        write!(file_hash, "{:02X}", byte).unwrap();
    }

    let update = NewUpdate {
        version: "my-version".to_owned(),
        file: file.clone(),
    };
    send_update(app.clone(), &token, org.id, col.id, dev.id, update).await;

    // Same hash, no update
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/update")
                .header("Authorization", format!("Basic {}", token.0))
                .header("x-ESP8266-sketch-md5", &file_hash)
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

    // Different version means there is an update to download
    let update = NewUpdate {
        version: "".to_owned(),
        file: file.clone(),
    };
    send_update(app.clone(), &token, org.id, col.id, dev.id, update).await;

    let up = find_update(app.clone(), &token, &version).await;
    assert_eq!(up, file);
}

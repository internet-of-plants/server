use axum::{body::Body, http, http::Method, http::Request, http::StatusCode};
use server::test_helpers::{login, signup};
use server::{test_router, Login, NewUser};
use tower::ServiceExt;

#[tokio::test]
async fn user() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão@example.com".to_owned(),
            organization_name: "bobão".to_owned(),
            username: "bobão".to_owned(),
            password: "bobão1234".to_owned(),
        },
    )
    .await;
    assert_eq!(token.0.len(), 64);

    let token = login(
        app.clone(),
        Login {
            email: "bobão@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        None,
        None,
    )
    .await;
    assert_eq!(token.0.len(), 64);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/user")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&NewUser {
                        email: "bobão@example.com".to_owned(),
                        organization_name: "bobão".to_owned(),
                        password: "bobão1234".to_owned(),
                        username: "bobão".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão@example.com".to_owned(),
            organization_name: "bobão".to_owned(),
            username: "bobão".to_owned(),
            password: "bobão1234".to_owned(),
        },
    )
    .await;
    assert_eq!(token.0.len(), 64);

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão2@example.com".to_owned(),
            organization_name: "bobão".to_owned(),
            username: "bobão2".to_owned(),
            password: "bobão1234".to_owned(),
        },
    )
    .await;
    assert_eq!(token.0.len(), 64);

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

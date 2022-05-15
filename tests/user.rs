use axum::{
    body::Body,
    http::{self, Method, Request, StatusCode},
};
use server::test_helpers::{login, signup};
use server::{db::user::Login, db::user::NewUser, test_router};
use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
async fn user() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão@example.com".to_owned(),
            username: "bobão".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;
    assert_eq!(token.0.len(), 64);

    let token = login(
        app.clone(),
        Login {
            email: "bobão@example.com".to_owned(),
            password: "bobão".to_owned(),
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
                        password: "bobão".to_owned(),
                        username: "bobão".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
/*
#[tokio::test]
async fn json() {
    let app = router();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/json")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body, json!({ "data": [1, 2, 3, 4] }));
}

#[tokio::test]
async fn not_found() {
    let app = router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert!(body.is_empty());
}

// You can also spawn a server and talk to it like any other HTTP server:
#[tokio::test]
async fn the_real_deal() {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(router().into_make_service())
            .await
            .unwrap();
    });

    let client = hyper::Client::new();

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}", addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}
*/

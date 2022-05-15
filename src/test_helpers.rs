use crate::db::user::{AuthToken, Login, NewUser};
use crate::{CollectionId, CollectionView, Organization, OrganizationId, OrganizationView};
use axum::{
    body::Body,
    http::{self, Method, Request, StatusCode},
    Router,
};
use tower::ServiceExt; // for `app.oneshot()`

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

pub async fn list_organizations(app: Router, token: &AuthToken) -> Vec<Organization> {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/organizations")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
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
                .uri(format!("/v1/organization/{}", id.0))
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
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
    organization_id: OrganizationId,
    collection_id: CollectionId,
) -> CollectionView {
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/organization/{}/collection/{}",
                    organization_id.0, collection_id.0
                ))
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
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

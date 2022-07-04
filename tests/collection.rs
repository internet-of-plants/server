use server::test_helpers::{find_collection, find_organization, list_organizations, login, signup};
use server::{db::user::Login, db::user::NewUser, test_router};

#[tokio::test]
async fn collection() {
    let app = test_router().await;

    signup(
        app.clone(),
        NewUser {
            email: "bobão3@example.com".to_owned(),
            username: "bobão3".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let token = login(
        app.clone(),
        Login {
            email: "bobão3@example.com".to_owned(),
            password: "bobão".to_owned(),
        },
        Some("aa".to_owned()),
        Some("bb".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;

    assert_eq!(org.collections.len(), 1);
    let collection = org.collections.into_iter().next().unwrap();

    let col = find_collection(app.clone(), &token, org.id, collection.id()).await;
    assert_eq!(col.id, collection.id());
}

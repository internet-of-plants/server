use server::test_helpers::{find_collection, list_organizations, login, signup};
use server::{Login, NewUser, test_router};

#[tokio::test]
async fn collection() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão3@example.com".to_owned(),
            organization_name: "bobão3".to_owned(),
            username: "bobão3".to_owned(),
            password: "bobão1234".to_owned(),
        },
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão3@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("aa".to_owned()),
        Some("bb".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;

    assert_eq!(orgs[0].collections.len(), 1);
    let collection = orgs[0].collections.iter().next().unwrap();

    let col = find_collection(app.clone(), &token, collection.id).await;
    assert_eq!(col.id, collection.id);
}

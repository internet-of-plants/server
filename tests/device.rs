use server::test_helpers::{
    find_collection, find_device, find_organization, list_organizations, login, signup,
};
use server::{db::user::Login, db::user::NewUser, test_router};

#[tokio::test]
async fn device() {
    let app = test_router().await;

    signup(
        app.clone(),
        NewUser {
            email: "bobão6@example.com".to_owned(),
            username: "bobão6".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let token = login(
        app.clone(),
        Login {
            email: "bobão6@example.com".to_owned(),
            password: "bobão".to_owned(),
        },
        Some("aaaaaa".to_owned()),
        Some("bbbbbb".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;

    assert_eq!(org.collections.len(), 1);
    let collection = org.collections.into_iter().next().unwrap();

    let col = find_collection(app.clone(), &token, org.id, *collection.id()).await;
    assert_eq!(col.id, *collection.id());
    assert_eq!(col.devices.len(), 1);

    let dev = find_device(
        app.clone(),
        &token,
        org.id,
        *collection.id(),
        *col.devices[0].id(),
    )
    .await;
    assert_eq!(dev.id, *col.devices[0].id());
}

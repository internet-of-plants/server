use server::{db::user::NewUser, test_router};
use server::test_helpers::{find_organization, list_organizations, signup};

#[tokio::test]
async fn organization() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão2@example.com".to_owned(),
            username: "bobão2".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    assert_eq!(orgs.len(), 1);

    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;
    assert_eq!(org.id, *orgs[0].id());
}

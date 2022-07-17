use server::test_helpers::{list_targets, signup};
use server::{test_router, NewUser};

#[tokio::test]
async fn target() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão11@example.com".to_owned(),
            organization_name: "bobão11".to_owned(),
            username: "bobão11".to_owned(),
            password: "bobão1234".to_owned(),
        },
    )
    .await;

    let targets = list_targets(app.clone(), &token).await;
    assert_eq!(targets.len(), 4);
}

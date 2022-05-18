use server::test_helpers::{signup, list_target_prototypes, find_target_prototype};
use server::{db::user::NewUser, test_router};

#[tokio::test]
async fn target_prototype() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão9@example.com".to_owned(),
            username: "bobão9".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let target_prototypes = list_target_prototypes(app.clone(), &token).await;
    assert_eq!(target_prototypes[0], find_target_prototype(app.clone(), &token, target_prototypes[0].id).await);
}

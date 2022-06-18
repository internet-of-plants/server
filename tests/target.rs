use server::test_helpers::{
    list_target_prototypes, list_targets, list_targets_for_prototype, signup,
};
use server::{db::target_prototype::TargetPrototypeId, db::user::NewUser, test_router};

// TODO: test for access of other users/orgs/collections

#[tokio::test]
async fn target() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão11@example.com".to_owned(),
            username: "bobão11".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let target_prototypes = list_target_prototypes(app.clone(), &token).await;

    let targets = list_targets(app.clone(), &token).await;
    assert_eq!(targets.len(), 1);

    let targets = list_targets_for_prototype(app.clone(), &token, target_prototypes[0].id).await;
    assert_eq!(targets.len(), 1);

    let targets = list_targets_for_prototype(app.clone(), &token, TargetPrototypeId(-1)).await;
    assert_eq!(targets.len(), 0);
}

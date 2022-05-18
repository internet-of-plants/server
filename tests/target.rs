use server::test_helpers::{signup, list_targets, list_target_prototypes, create_target, list_targets_for_prototype};
use server::{db::user::NewUser, test_router, db::target_prototype::TargetPrototypeId};

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
    let target = create_target(
        app.clone(),
        &token,
        target_prototypes[0].id,
        target_prototypes[0].boards[0].id,
    )
    .await;

    let targets = list_targets(
        app.clone(),
        &token,
    )
    .await;
    assert_eq!(targets.len(), 1);
    assert_eq!(target, targets[0]);

    let targets = list_targets_for_prototype(
        app.clone(),
        &token,
        target_prototypes[0].id,
    )
    .await;
    assert_eq!(targets.len(), 1);
    assert_eq!(target, targets[0]);

    let targets = list_targets_for_prototype(
        app.clone(),
        &token,
        TargetPrototypeId(-1)
    )
    .await;
    assert_eq!(targets.len(), 0);
}

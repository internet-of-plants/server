use server::test_helpers::{list_sensor_prototypes, list_targets, signup};
use server::{test_router, NewUser};

#[tokio::test]
async fn sensor_prototype() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão10@example.com".to_owned(),
            username: "bobão10".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão10".to_owned(),
        },
    )
    .await;

    let targets = list_targets(app.clone(), &token).await;

    let sensor_prototypes = list_sensor_prototypes(app.clone(), &token, targets[0].id()).await;
    assert_eq!(sensor_prototypes.len(), 4);
}

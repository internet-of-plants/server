use server::test_helpers::{
    create_target, find_sensor_prototype, list_sensor_prototypes, list_target_prototypes, signup,
};
use server::{
    db::board::Board, db::sensor::config_type::WidgetKind, db::user::NewUser, test_router,
};
use sqlx::Connection;

#[tokio::test]
async fn sensor_prototype() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão10@example.com".to_owned(),
            username: "bobão10".to_owned(),
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

    let sensor_prototypes = list_sensor_prototypes(app.clone(), &token).await;
    assert_eq!(
        sensor_prototypes[0],
        find_sensor_prototype(app.clone(), &token, sensor_prototypes[0].id, None).await
    );

    let mut view = sensor_prototypes[0].clone();

    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop_test";
    let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
    let mut txn = connection.begin().await.unwrap();
    let board = Board::find_by_id(&mut txn, target_prototypes[0].boards[0].id)
        .await
        .unwrap();
    let pins = board.pins(&mut txn).await.unwrap();
    txn.commit().await.unwrap();

    for config_request in &mut view.configuration_requests {
        if config_request.ty.name == "Pin" {
            config_request.ty.widget = WidgetKind::Selection(pins);
            break;
        }
    }
    assert_eq!(
        view,
        find_sensor_prototype(app.clone(), &token, sensor_prototypes[0].id, Some(target)).await
    );
}

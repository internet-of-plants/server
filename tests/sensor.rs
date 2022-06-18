use rand::{random, seq::SliceRandom};
use server::test_helpers::{
    create_sensor, list_sensor_prototypes, list_sensors, list_sensors_for_prototype,
    signup, list_targets,
};
use server::{
    db::sensor::config_type::WidgetKind, db::sensor::NewConfig, db::sensor::NewSensor,
    db::sensor_prototype::SensorPrototype, db::sensor_prototype::SensorPrototypeId,
    db::user::NewUser, test_router,
};
use sqlx::Connection;

#[tokio::test]
async fn sensor() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão12@example.com".to_owned(),
            username: "bobão12".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let targets = list_targets(app.clone(), &token).await;
    let sensor_prototypes = list_sensor_prototypes(app.clone(), &token).await;

    let mut configs = vec![];
    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop_test";
    let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
    let mut txn = connection.begin().await.unwrap();
    let prototype = SensorPrototype::find_by_id(&mut txn, sensor_prototypes[0].id)
        .await
        .unwrap();
    for config_request in prototype.configuration_requests(&mut txn).await.unwrap() {
        let ty = config_request.ty(&mut txn).await.unwrap();
        configs.push(NewConfig {
            request_id: config_request.id,
            value: match ty.widget(&mut txn, &[targets[0].id]).await.unwrap() {
                WidgetKind::U8 => format!("{}", random::<u8>()),
                WidgetKind::U16 => format!("{}", random::<u16>()),
                WidgetKind::U32 => format!("{}", random::<u32>()),
                WidgetKind::U64 => format!("{}", random::<u64>()),
                WidgetKind::F32 => format!("{}", random::<f32>()),
                WidgetKind::F64 => format!("{}", random::<f64>()),
                WidgetKind::String => format!("{}", random::<u32>()),
                WidgetKind::Selection(opts) => {
                    opts.choose(&mut rand::thread_rng()).unwrap().clone()
                }
                WidgetKind::PinSelection => todo!(),
            },
        });
    }
    txn.commit().await.unwrap();

    let sensor = create_sensor(
        app.clone(),
        &token,
        NewSensor {
            prototype_id: sensor_prototypes[0].id,
            configs,
        },
    )
    .await;

    let sensors = list_sensors(app.clone(), &token).await;
    assert_eq!(sensors.len(), 1);
    assert_eq!(sensors[0], sensor);

    let sensors = list_sensors_for_prototype(app.clone(), &token, sensor_prototypes[0].id).await;
    assert_eq!(sensors.len(), 1);
    assert_eq!(sensors[0], sensor);

    let sensors = list_sensors_for_prototype(app.clone(), &token, SensorPrototypeId(-1)).await;
    assert_eq!(sensors.len(), 0);
}

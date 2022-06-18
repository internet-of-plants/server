use rand::{random, seq::SliceRandom};
use server::DeviceId;
use server::test_helpers::{
    compile_firmware, create_compiler, create_sensor, list_compilations, list_sensor_prototypes,
    list_targets, signup,
};
use server::{
    controllers::compiler::NewCompiler, db::sensor::config_type::WidgetKind, db::sensor::NewConfig,
    db::sensor::NewSensor, db::sensor_prototype::SensorPrototype, db::user::NewUser, test_router,
};
use sqlx::Connection;

#[tokio::test]
async fn compiler() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão13@example.com".to_owned(),
            username: "bobão13".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let target = list_targets(app.clone(), &token)
        .await
        .into_iter()
        .next()
        .unwrap();
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
            value: match ty.widget(&mut txn, &[target.id]).await.unwrap() {
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

    let new_sensor1 = NewSensor {
        prototype_id: sensor_prototypes[0].id,
        configs,
    };
    let _sensor1 = create_sensor(app.clone(), &token, new_sensor1.clone()).await;

    let prototype = SensorPrototype::find_by_id(&mut txn, sensor_prototypes[1].id)
        .await
        .unwrap();
    let mut configs = vec![];
    for config_request in prototype.configuration_requests(&mut txn).await.unwrap() {
        let ty = config_request.ty(&mut txn).await.unwrap();
        configs.push(NewConfig {
            request_id: config_request.id,
            value: match ty.widget(&mut txn, &[target.id]).await.unwrap() {
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

    let new_sensor2 = NewSensor {
        prototype_id: sensor_prototypes[1].id,
        configs,
    };
    let _sensor2 = create_sensor(app.clone(), &token, new_sensor2.clone()).await;

    let compilation = create_compiler(
        app.clone(),
        &token,
        NewCompiler {
            device_id: DeviceId::new(-1),
            target_id: target.id,
            sensors: vec![new_sensor1, new_sensor2],
        },
    )
    .await;
    println!("{}", compilation.platformio_ini);
    println!("{}", compilation.main_cpp);
    println!("{}", compilation.pin_hpp);

    let compilations = list_compilations(app.clone(), &token).await;
    assert_eq!(compilations.len(), 1);
    assert_eq!(compilation, compilations[0]);

    // TODO: check that the binary makes sense?
    compile_firmware(app.clone(), &token, compilations[0].id).await;
}

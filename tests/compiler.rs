use axum::{body::Body, http, http::Method, http::Request, http::StatusCode};
use server::test_helpers::{
    create_compiler, list_organizations, list_sensor_prototypes, list_targets, login, signup,
};
use server::{
    test_router, DeviceWidgetKind, Login, NewCompiler, NewDeviceConfig, NewSensor, NewSensorConfig,
    NewUser, SensorWidgetKind,
};
use tower::ServiceExt;

// TODO: randomize data sent
// TODO: abstract data generation in test_helpers
// TODO: ensure other organization's compiler/sensor_configs/device_configs can't be shared
// TODO: test compiler reusage inside an organization

#[tokio::test]
async fn compiler() {
    let app = test_router().await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão13@example.com".to_owned(),
            username: "bobão13".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão13".to_owned(),
        },
    )
    .await;

    let token1 = signup(
        app.clone(),
        NewUser {
            email: "bobão14@example.com".to_owned(),
            username: "bobão14".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão14".to_owned(),
        },
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão14@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("fasfas".to_owned()),
        Some("hseeeh".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token1).await;
    let device_id1 = orgs[0].collections[0].devices[0].id();

    login(
        app.clone(),
        Login {
            email: "bobão13@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("gggggg".to_owned()),
        Some("hhhhhh".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let device_id = orgs[0].collections[0].devices[0].id();

    let targets = list_targets(app.clone(), &token).await;
    let sensor_prototypes = list_sensor_prototypes(app.clone(), &token, targets[0].id()).await;
    let mut new_sensor1 = NewSensor {
        prototype_id: sensor_prototypes[0].id,
        alias: "Sensor 1".to_owned(),
        configs: Vec::new(),
    };
    for request in &sensor_prototypes[0].configuration_requests {
        let value = match &request.ty.widget {
            SensorWidgetKind::Selection(options) => options[0].clone(),
            _ => todo!(),
        };
        new_sensor1.configs.push(NewSensorConfig {
            request_id: request.id,
            value,
        });
    }

    let mut new_sensor2 = NewSensor {
        prototype_id: sensor_prototypes[1].id,
        alias: "Sensor 2".to_owned(),
        configs: Vec::new(),
    };
    for request in &sensor_prototypes[1].configuration_requests {
        let value = match &request.ty.widget {
            SensorWidgetKind::Selection(options) => options[0].clone(),
            _ => todo!(),
        };
        new_sensor2.configs.push(NewSensorConfig {
            request_id: request.id,
            value,
        });
    }

    let mut new_sensor3 = NewSensor {
        prototype_id: sensor_prototypes[2].id,
        alias: "Sensor 3".to_owned(),
        configs: Vec::new(),
    };
    for request in &sensor_prototypes[2].configuration_requests {
        let value = match &request.ty.widget {
            SensorWidgetKind::Selection(options) => options[0].clone(),
            _ => todo!(),
        };
        new_sensor3.configs.push(NewSensorConfig {
            request_id: request.id,
            value,
        });
    }

    let mut device_configs = Vec::new();
    for request in &targets[0].configuration_requests {
        let value = match &request.ty.widget {
            DeviceWidgetKind::SSID => "my-ssid".to_owned(),
            DeviceWidgetKind::PSK => "my-psk".to_owned(),
        };
        device_configs.push(NewDeviceConfig {
            request_id: request.id,
            value,
        });
    }

    let compilation = create_compiler(
        app.clone(),
        &token,
        NewCompiler {
            device_id,
            target_id: targets[0].id(),
            device_configs: device_configs.clone(),
            sensors: vec![
                new_sensor1.clone(),
                new_sensor2.clone(),
                new_sensor3.clone(),
            ],
        },
    )
    .await;

    let compilation2 = create_compiler(
        app.clone(),
        &token,
        NewCompiler {
            device_id,
            target_id: targets[0].id(),
            device_configs: device_configs.clone(),
            sensors: vec![
                new_sensor1.clone(),
                new_sensor2.clone(),
                new_sensor3.clone(),
            ],
        },
    )
    .await;
    assert_eq!(compilation, compilation2);

    println!("{}", compilation.platformio_ini);
    println!("{}", compilation.main_cpp);
    println!("{}", compilation.pin_hpp);
    println!("\n\n\n\n\n\n\n\n\n\n\n\n");

    device_configs[0].value = "my-new-ssid".to_owned();
    let compilation3 = create_compiler(
        app.clone(),
        &token,
        NewCompiler {
            device_id,
            target_id: targets[0].id(),
            device_configs: device_configs.clone(),
            sensors: vec![
                new_sensor1.clone(),
                new_sensor2.clone(),
                new_sensor3.clone(),
            ],
        },
    )
    .await;
    assert_ne!(compilation, compilation3);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/compiler")
                .header("Authorization", format!("Basic {}", token.0))
                .method(Method::POST)
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&NewCompiler {
                        device_id: device_id1,
                        target_id: targets[0].id(),
                        device_configs,
                        sensors: vec![new_sensor1, new_sensor2, new_sensor3],
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // TODO: check that the binary makes sense?
    // TODO: check that all configs/sensors are there

    /*
    for config_request in prototype.configuration_requests(&mut txn).await.unwrap() {
        let ty = config_request.ty(&mut txn).await.unwrap();
        configs.push(NewConfig {
            request_id: config_request.id,
            value: match ty.widget(&mut txn, &[&target]).await.unwrap() {
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
    */
}

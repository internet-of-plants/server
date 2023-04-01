use axum::{body::Body, http, http::Method, http::Request, http::StatusCode};
use server::controllers::sensor::{SetAliasRequest, SetColorRequest};
use server::test_helpers::{
    create_compiler, list_organizations, list_sensor_prototypes, list_targets, login,
    set_sensor_alias, set_sensor_color, signup,
};
use server::{
    test_router, DeviceWidgetKind, Login, NewCompiler, NewDeviceConfig, NewSensor, NewSensorConfig,
    NewSensorWidgetKind, NewUser,
};
use tower::ServiceExt;

#[tokio::test]
async fn sensors_alias() {
    let app = test_router().await;

    let other_token = signup(
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
        Some("fasfg".to_owned()),
        Some("hseeh".to_owned()),
    )
    .await;

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

    login(
        app.clone(),
        Login {
            email: "bobão13@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("fasfas".to_owned()),
        Some("hseeeh".to_owned()),
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão13@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("faeafs".to_owned()),
        Some("efefeh".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let device_id = orgs[0].collections[0].devices[0].id();
    let device_id1 = orgs[0].collections[1].devices[0].id();

    let targets = list_targets(app.clone(), &token).await;
    let sensor_prototypes = list_sensor_prototypes(app.clone(), &token, targets[0].id()).await;
    let mut new_sensor1 = NewSensor {
        prototype_id: sensor_prototypes[0].id,
        alias: "Sensor 1".to_owned(),
        configs: Vec::new(),
    };
    for request in &sensor_prototypes[0].configuration_requests {
        let value = match &request.ty.widget {
            NewSensorWidgetKind::Selection(options) => options[0].clone(),
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
            NewSensorWidgetKind::Selection(options) => options[0].clone(),
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
            NewSensorWidgetKind::Selection(options) => options[0].clone(),
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

    set_sensor_alias(
        app.clone(),
        &token,
        SetAliasRequest {
            device_id,
            sensor_id: compilation.compiler.sensors[0].id,
            alias: "Sensor 1 Alias".to_owned(),
        },
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let alias = orgs[0].collections[0].devices[0]
        .compiler
        .as_ref()
        .unwrap()
        .sensors[0]
        .alias
        .clone();
    assert_eq!("Sensor 1 Alias", alias);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/sensor/alias")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&SetAliasRequest {
                        device_id: device_id1,
                        sensor_id: compilation.compiler.sensors[0].id,
                        alias: "Sensor 1 Hijack".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/sensor/alias")
                .header("Authorization", format!("Basic {}", other_token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&SetAliasRequest {
                        device_id,
                        sensor_id: compilation.compiler.sensors[0].id,
                        alias: "Sensor 1 Hijack".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn sensors_color() {
    let app = test_router().await;

    let other_token = signup(
        app.clone(),
        NewUser {
            email: "bobão16@example.com".to_owned(),
            username: "bobão16".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão16".to_owned(),
        },
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão16@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("fasfg".to_owned()),
        Some("hseeh".to_owned()),
    )
    .await;

    let token = signup(
        app.clone(),
        NewUser {
            email: "bobão15@example.com".to_owned(),
            username: "bobão15".to_owned(),
            password: "bobão1234".to_owned(),
            organization_name: "bobão15".to_owned(),
        },
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão15@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("fasfas".to_owned()),
        Some("hseeeh".to_owned()),
    )
    .await;

    login(
        app.clone(),
        Login {
            email: "bobão15@example.com".to_owned(),
            password: "bobão1234".to_owned(),
        },
        Some("faeafs".to_owned()),
        Some("efefeh".to_owned()),
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let device_id = orgs[0].collections[0].devices[0].id();
    let device_id1 = orgs[0].collections[1].devices[0].id();

    let targets = list_targets(app.clone(), &token).await;
    let sensor_prototypes = list_sensor_prototypes(app.clone(), &token, targets[0].id()).await;
    let mut new_sensor1 = NewSensor {
        prototype_id: sensor_prototypes[0].id,
        alias: "Sensor 1".to_owned(),
        configs: Vec::new(),
    };
    for request in &sensor_prototypes[0].configuration_requests {
        let value = match &request.ty.widget {
            NewSensorWidgetKind::Selection(options) => options[0].clone(),
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
            NewSensorWidgetKind::Selection(options) => options[0].clone(),
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
            NewSensorWidgetKind::Selection(options) => options[0].clone(),
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

    set_sensor_color(
        app.clone(),
        &token,
        SetColorRequest {
            device_id,
            sensor_id: compilation.compiler.sensors[0].id,
            color: "Sensor 1 Color".to_owned(),
        },
    )
    .await;

    let orgs = list_organizations(app.clone(), &token).await;
    let color = orgs[0].collections[0].devices[0]
        .compiler
        .as_ref()
        .unwrap()
        .sensors[0]
        .color
        .clone();
    assert_eq!("Sensor 1 Color", color);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/sensor/color")
                .header("Authorization", format!("Basic {}", token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&SetColorRequest {
                        device_id: device_id1,
                        sensor_id: compilation.compiler.sensors[0].id,
                        color: "Sensor 1 Hijack".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/sensor/color")
                .header("Authorization", format!("Basic {}", other_token.0))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(Body::from(
                    serde_json::to_vec(&SetColorRequest {
                        device_id,
                        sensor_id: compilation.compiler.sensors[0].id,
                        color: "Sensor 1 Hijack".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

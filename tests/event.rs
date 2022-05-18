use server::test_helpers::{find_collection, find_organization, list_organizations, login, signup, find_device, send_event};
use server::{db::user::Login, db::user::NewUser, test_router, NewEvent};
use rand::random;
use server::extractor::{MacAddress, Version};

#[tokio::test]
async fn event() {
    let app = test_router().await;

    signup(
        app.clone(),
        NewUser {
            email: "bobão7@example.com".to_owned(),
            username: "bobão7".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let mac_address = MacAddress("aaaaaaa".to_owned());
    let version = Version("bbbbbbb".to_owned());
    let token = login(
        app.clone(),
        Login {
            email: "bobão7@example.com".to_owned(),
            password: "bobão".to_owned(),
        },
        Some(mac_address.0.clone()),
        Some(version.0.clone()),
    )
    .await;
    
    let event = NewEvent {
        air_temperature_celsius: random(),
        air_humidity_percentage: 2.,
        air_heat_index_celsius: 3.,
        soil_resistivity_raw: 4,
        soil_temperature_celsius: 5.,
    };
    send_event(app.clone(), &token, &version, &mac_address, &event).await;

    let orgs = list_organizations(app.clone(), &token).await;
    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;

    assert_eq!(org.collections.len(), 1);
    let collection = org.collections.into_iter().next().unwrap();

    let col = find_collection(app.clone(), &token, org.id, *collection.id()).await;
    assert_eq!(col.id, *collection.id());
    assert_eq!(col.devices.len(), 1);
    
    let dev = find_device(app.clone(), &token, org.id, *collection.id(), *col.devices[0].id()).await;
    assert_eq!(dev.id, *col.devices[0].id());

    assert_eq!(format!("{:0.4}", dev.last_event.unwrap().air_temperature_celsius), format!("{:0.4}", event.air_temperature_celsius));
}

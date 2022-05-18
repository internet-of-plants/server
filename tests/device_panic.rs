use server::test_helpers::{
    find_collection, find_organization, list_device_panics, list_organizations, login,
    send_device_panic, signup,
};
use server::{db::device_panic::NewDevicePanic, db::user::Login, db::user::NewUser, test_router};

#[tokio::test]
async fn device_panic() {
    let app = test_router().await;

    signup(
        app.clone(),
        NewUser {
            email: "bobão5@example.com".to_owned(),
            username: "bobão5".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let token = login(
        app.clone(),
        Login {
            email: "bobão5@example.com".to_owned(),
            password: "bobão".to_owned(),
        },
        Some("aaaaa".to_owned()),
        Some("bbbbb".to_owned()),
    )
    .await;

    let panic = NewDevicePanic {
        file: "panic.cpp".to_owned(),
        line: 1,
        func: "function".to_owned(),
        msg: "my panichy panic panicher".to_owned(),
    };
    send_device_panic(app.clone(), &token, &panic).await;

    let orgs = list_organizations(app.clone(), &token).await;
    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;

    assert_eq!(org.collections.len(), 1);
    let collection = org.collections.into_iter().next().unwrap();

    let col = find_collection(app.clone(), &token, org.id, *collection.id()).await;
    assert_eq!(col.id, *collection.id());
    assert_eq!(col.devices.len(), 1);

    let panics = list_device_panics(
        app.clone(),
        &token,
        org.id,
        *collection.id(),
        *col.devices[0].id(),
    )
    .await;
    assert_eq!(panics.len(), 1);
    assert_eq!(panics[0].msg(), panic.msg);
}

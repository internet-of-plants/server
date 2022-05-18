use server::test_helpers::{
    find_collection, find_organization, list_device_logs, list_organizations, login,
    send_device_log, signup,
};
use server::{db::user::Login, db::user::NewUser, test_router};

#[tokio::test]
async fn device_log() {
    let app = test_router().await;

    signup(
        app.clone(),
        NewUser {
            email: "bobão4@example.com".to_owned(),
            username: "bobão4".to_owned(),
            password: "bobão".to_owned(),
        },
    )
    .await;

    let token = login(
        app.clone(),
        Login {
            email: "bobão4@example.com".to_owned(),
            password: "bobão".to_owned(),
        },
        Some("aaaa".to_owned()),
        Some("bbba".to_owned()),
    )
    .await;

    let log = "my loggy log logger";
    send_device_log(app.clone(), &token, &log).await;

    let orgs = list_organizations(app.clone(), &token).await;
    let org = find_organization(app.clone(), &token, *orgs[0].id()).await;

    assert_eq!(org.collections.len(), 1);
    let collection = org.collections.into_iter().next().unwrap();

    let col = find_collection(app.clone(), &token, org.id, *collection.id()).await;
    assert_eq!(col.id, *collection.id());
    assert_eq!(col.devices.len(), 1);

    let logs = list_device_logs(
        app.clone(),
        &token,
        org.id,
        *collection.id(),
        *col.devices[0].id(),
    )
    .await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].log(), log);
}

use lib::utils::{Timestamp, UID};
use schema::users;

#[macro_export]
macro_rules! UserViewSql {
    () => ((users::id, users::username, users::email, users::timestamp));
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct UserView {
    pub id: UID,
    pub username: String,
    pub email: String,
    pub timestamp: Timestamp,
}

#[derive(Queryable, Serialize, Debug)]
pub struct User {
    pub id: UID,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub timestamp: Timestamp,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

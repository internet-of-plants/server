use lib::utils::UID;
use schema::users;

#[derive(Queryable, Serialize, Debug)]
pub struct User {
    pub id: UID,
    pub username: String,
    pub email: String,
    pub password_hash: String
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String
}

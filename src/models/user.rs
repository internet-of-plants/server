use lib::{schema::users, utils::Timestamp, utils::UID};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignupForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SigninForm {
    pub login: String,
    pub password: String,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: UID,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub timestamp: Timestamp,
}

pub type UserView = User;

macro_rules! UserViewSql {
    () => {
        users::all_columns
    };
}

use schema::users::dsl::*;

//#[derive(Insertable, Debug, Deserialize)]
//#[table_name = "users"]
#[derive(Deserialize, Debug, Clone)]
pub struct SignupForm {
    pub name: String,
    pub email: String,
    pub password: String
}

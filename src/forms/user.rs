#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize)]
pub struct SigninForm {
    pub login: String,
    pub password: String
}

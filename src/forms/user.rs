#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize)]
pub struct SigninForm {
    pub email: String,
    pub password: String
}

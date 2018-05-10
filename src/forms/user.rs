use lib::utils::{parse_multipart, MultipartDeserialize};

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl MultipartDeserialize for SignupForm {
    fn from_multipart(content: &[u8], boundary: &[u8]) -> Option<Self> {
        let values = parse_multipart(content, boundary);
        let username = match values.get("username") {
            Some(username) => username.to_owned(),
            None => return None,
        };

        let email = match values.get("email") {
            Some(email) => email.to_owned(),
            None => return None,
        };

        match values.get("password") {
            Some(password) => Some(SignupForm {
                username: username,
                email: email,
                password: password.to_owned(),
            }),
            None => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SigninForm {
    pub login: String,
    pub password: String,
}

impl MultipartDeserialize for SigninForm {
    fn from_multipart(content: &[u8], boundary: &[u8]) -> Option<Self> {
        let values = parse_multipart(content, boundary);
        let login = match values.get("login") {
            Some(email) => email.to_owned(),
            None => return None,
        };

        match values.get("password") {
            Some(password) => Some(SigninForm {
                login: login,
                password: password.to_owned(),
            }),
            None => None,
        }
    }
}

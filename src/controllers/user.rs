use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::User;
use forms::SignupForm;
use lib::http::{render_template, redirect};
use lib::auth::is_auth;

/// Returns the HTML form to create a new account
pub fn signup(state: State) -> (State, Response) {
    if is_auth(&state) {
        return redirect(state, url_for!("home"));
    }

    render_template(state, "signup.html", &mut Context::new())
}

/// Get name, email, password, creates actor, authenticates
pub fn signup_post(mut state: State) -> (State, Response) {
    if is_auth(&state) {
        return redirect(state, url_for!("home"));
    }

    let signup = from_body!(state, "signup", SignupForm);
    println!("{:?}", signup);

    /*
    let mut actor = ActorDb::new(None, name, email, password);
    actor.save();

    // actor.save() stores the id so actor.get_id() will return Some(i64)
    let actor_id = actor.get_id().unwrap();

    // Saves signed cookie to authenticate actor
    authenticate_actor(actor_id, req);
    */

    redirect(state, url_for!("home"))
}
/*

/// Returns the HTML form to authenticate as an actor
fn signin(mut state: State) -> (State, Response) {
    if is_auth(req) {
        return redirect(url_for!(req, "home"));
    }

    let mut ctx = Context::new();
    // `is_auth` sets csrf token in GET requests if non existent
    ctx.add("csrf_token", &get_csrf_token(req).unwrap());
    render_template("signin.html", &mut ctx)
}

/// Receives email, password, authenticates actor
fn signin_post(mut state: State) -> (State, Response) {
    if is_auth(req) {
        return redirect(url_for!(req, "home"));
    }

    let email = match get_param(req, "email") {
        Some(Value::String(value)) => value,
        Some(_) => return invalid_value("e-mail"),
        None => return missing_field("e-mail")
    };

    let password = match get_param(req, "password") {
        Some(Value::String(value)) => value,
        Some(_) => return invalid_value("password"),
        None => return missing_field("password")
    };

    match ActorDb::authenticate(&email, &password) {
        Some(id) => authenticate_actor(id, req),
        None => return not_found("Wrong e-mail or password")
    }

    redirect(url_for!(req, "home"))
}

// Logout user
fn logout(mut state: State) -> (State, Response) {
    if !is_auth(req){
        return redirect(url_for!(req, "signin"));
    }

    deauth(req);
    redirect(url_for!(req, "signin"))
}

#[cfg(test)]
use iron::Headers;

#[cfg(test)]
impl ActorClient {
    /// Authenticate user for further requests, for testing purposes
    pub fn auth_test(mut state: State) -> (State, Response){
        use client::adapter::html;

        let mut actor = ActorDb::new(None,
                                    "Actor".to_owned(),
                                    "email@example.com".to_owned(),
                                    "Password!".to_owned());
        actor.save();
        let actor_id = actor.get_id().unwrap();

        authenticate_actor(actor_id, req);

        html::response(Ok(&actor_id.to_string()))
    }

    /// Makes request to obtain signed authentication cookie and actor id
    pub fn auth_headers() -> (i64, Headers) {
        use lib::handler::set_handler;
        use iron_test::{request, response};
        use iron::headers::Cookie;
        use lib::utils::{format_url, format_cookies};

        let resp = request::get(&format_url("auth_test"),
                                Headers::new(),
                                &set_handler()).unwrap();

        let mut headers = Headers::new();
        headers.set(Cookie(format_cookies(resp.headers.clone())));

        let actor_id = response::extract_body_to_string(resp).parse::<i64>().unwrap();

        (actor_id, headers)
    }
}

#[cfg(test)]
mod tests {
    use iron::headers::ContentType;
    use lib::handler::set_handler;
    use iron::headers::Cookie;
    use iron::{Headers, status};
    use iron_test::request;
    use lib::utils::{format_body, format_url, format_cookies};
    use config;

    // Assure user is authenticated
    fn authenticated_test(headers: Headers) {
        let resp = request::get(&format_url("signin"),
                                headers,
                                &set_handler()).unwrap();
        assert_eq!(resp.status.unwrap(), status::Found);
    }

    // Assure user is not authenticated
    fn non_authenticated_test(headers: Headers) {
        let resp = request::get(&format_url("signup"),
                                headers,
                                &set_handler()).unwrap();
        assert_eq!(resp.status.unwrap(), status::Ok);
    }

    // signup unit test
    fn signup_test() {
        // Set headers for post request
        let mut headers = Headers::new();
        headers.set(ContentType::form_url_encoded());

        let body = format_body("name=Name&email=actor@example.com&password=123");
        let resp = request::post(&format_url("signup"),
                                 headers,
                                 &body,
                                 &set_handler()).unwrap();
        assert_eq!(resp.status.unwrap(), status::Found);

        // Get cookie and assure user is authenticated
        headers = Headers::new();
        headers.set(Cookie(format_cookies(resp.headers)));
        authenticated_test(headers);
    }

    // signin unit test
    fn signin_test() -> Headers {
        // Set headers for post request
        let mut headers = Headers::new();
        headers.set(ContentType::form_url_encoded());

        let body = format_body("email=actor@example.com&password=123");
        let resp = request::post(&format_url("signin"),
                                 headers,
                                 &body,
                                 &set_handler()).unwrap();
        assert_eq!(resp.status.unwrap(), status::Found);

        // Get cookie and assure user is authenticated
        headers = Headers::new();
        headers.set(Cookie(format_cookies(resp.headers)));
        authenticated_test(headers.clone());

        // Return headers to test logout
        headers
    }

    // logout unit test
    fn logout_test(headers: Headers) {
        // Assure user is authenticated before logout
        authenticated_test(headers.clone());

        // Set headers for post request while still auth
        let mut post_headers = headers.clone();
        post_headers.set(ContentType::form_url_encoded());

        let resp = request::post(&format_url("logout"),
                                 post_headers,
                                 &format_body(""),
                                 &set_handler()).unwrap();
        assert_eq!(resp.status.unwrap(), status::Found);

        // Get cookies and assure user is not authenticated anymore
        let mut headers = Headers::new();
        headers.set(Cookie(format_cookies(resp.headers)));
        non_authenticated_test(headers);
    }

    #[test]
    /// Unit and integration tests on all methods from ActorClientPort
    fn actor_handler() {
        config::load();

        // Assure that users that didn't logout are not authenticated
        non_authenticated_test(Headers::new());

        // Unit tests
        signup_test();
        let headers = signin_test();
        logout_test(headers);
    }
}
*/

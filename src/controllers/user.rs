use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::{NewUser, User, UserView};
use forms::{SigninForm, SignupForm};
use lib::http::{bad_request, redirect, render_template};
use lib::auth::{authenticate, check_password, deauth, hash_password, is_auth};
use lib::db::connection;
use router::SlugPath;

use diesel::prelude::*;
use diesel::insert_into;
use schema::users;

use slugify::slugify;

lazy_static! {
    static ref ANTI_TIMING_HASH: String = hash_password(&"");
}

pub fn user(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let username = from_path!(state, SlugPath).slug;

    let user = try_db!(
        state,
        users::table
            .filter(users::username.eq(username))
            .select(UserViewSql!())
            .first::<UserView>(&*connection())
    );

    let mut ctx = Context::new();
    ctx.add("user", &user);
    render_template(state, "user.html", &mut ctx)
}

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

    let form = from_body!(state, url_for!("signup"), SignupForm);
    let user = NewUser {
        username: slugify!(&form.username),
        email: form.email,
        password_hash: hash_password(&form.password),
    };

    let user = try_db!(
        state,
        insert_into(users::table)
            .values(&user)
            .get_result::<User>(&*connection())
    );
    authenticate(
        &mut state,
        UserView {
            id: user.id,
            username: user.username,
            email: user.email,
            timestamp: user.timestamp,
        },
    );

    redirect(state, url_for!("home"))
}

/// Returns the HTML form to authenticate as an actor
pub fn signin(state: State) -> (State, Response) {
    if is_auth(&state) {
        return redirect(state, url_for!("home"));
    }

    render_template(state, "signin.html", &mut Context::new())
}

/// Receives email, password, authenticates actor
pub fn signin_post(mut state: State) -> (State, Response) {
    if is_auth(&state) {
        return redirect(state, url_for!("home"));
    }

    let form = from_body!(state, url_for!("signin"), SigninForm);

    let user = try_db_option!(
        state,
        users::table
            .filter(
                users::email
                    .eq(form.login.clone())
                    .or(users::username.eq(form.login))
            )
            .first::<User>(&*connection())
    );

    // Hardens against timing attacks
    match user {
        Some(user) => {
            if check_password(&form.password, &user.password_hash) {
                authenticate(
                    &mut state,
                    UserView {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        timestamp: user.timestamp,
                    },
                );
                redirect(state, url_for!("home"))
            } else {
                redirect(state, url_for!("signin"))
            }
        }
        None => {
            check_password(&form.password, &ANTI_TIMING_HASH);
            bad_request(state)
        }
    }
}

// Logout user
pub fn logout(mut state: State) -> (State, Response) {
    assert_csrf!(state, url_for!("login"));
    deauth(&mut state);
    redirect(state, url_for!("signin"))
}

/*
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

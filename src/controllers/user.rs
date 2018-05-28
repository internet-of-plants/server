use actix_web::{HttpRequest, HttpResponse, Json, Path, middleware::session::RequestSession};
use diesel::{insert_into, NotFound, prelude::*};
use slugify::slugify;

use State;
use lib::auth::{authenticate, check_password, hash_password, is_auth, user_id};
use lib::{error::Error, schema::users};
use models::{NewUser, SigninForm, SignupForm, User, UserView};

pub fn user((path, req): (Path<String>, HttpRequest<State>)) -> Result<HttpResponse, Error> {
    is_auth(&req)?;
    let username = path.into_inner();
    trace!(
        req.state().log,
        "User (user_id: {:?}): {}",
        user_id(&req),
        username.clone()
    );

    let user = users::table
        .filter(users::username.eq(username))
        .select(UserViewSql!())
        .first::<UserView>(conn!(req.state().pool))?;
    info!(req.state().log, "User: {:?}", user);

    Ok(HttpResponse::Ok().json(user))
}

pub fn signup((form, req): (Json<SignupForm>, HttpRequest<State>)) -> Result<HttpResponse, Error> {
    let form = form.into_inner();
    trace!(req.state().log, "Signup: {}", form.username);

    if is_auth(&req).is_ok() {
        debug!(req.state().log, "Already authenticated");
        return Ok(HttpResponse::Ok().finish());
    }

    let user = NewUser {
        username: filled!(slugify!(&form.username)),
        email: filled!(form.email),
        password_hash: hash_password(&filled!(form.password))?,
    };

    let user = insert_into(users::table)
        .values(&user)
        .get_result::<User>(conn!(req.state().pool))?;
    info!(req.state().log, "Created User: {:?}", user);

    authenticate(&req, user.id)?;
    info!(req.state().log, "Authenticated: {}", user.id);

    Ok(HttpResponse::Ok().finish())
}

pub fn signin((form, req): (Json<SigninForm>, HttpRequest<State>)) -> Result<HttpResponse, Error> {
    let form = form.into_inner();
    trace!(req.state().log, "Signin: {}", form.login);

    if is_auth(&req).is_ok() {
        debug!(req.state().log, "Already authenticated");
        return Ok(HttpResponse::Ok().finish());
    }

    let user = match users::table
        .filter(
            users::email
                .eq(form.login.clone())
                .or(users::username.eq(form.login)),
        )
        .first::<UserView>(conn!(req.state().pool))
    {
        Ok(u) => u,
        Err(NotFound) => {
            // Hardern against timing attacks
            hash_password("")?;
            return Err(Error::InvalidCredentials);
        }
        Err(err) => Err(Error::Diesel(err))?,
    };
    info!(req.state().log, "Signin User: {:?}", user);

    if check_password(&form.password, &user.password_hash)? {
        authenticate(&req, user.id)?;
        info!(req.state().log, "Authenticated: {}", user.id);

        Ok(HttpResponse::Ok().finish())
    } else {
        Err(Error::InvalidCredentials)
    }
}

pub fn logout(req: HttpRequest<State>) -> Result<HttpResponse, Error> {
    trace!(req.state().log, "Logout: {:?}", user_id(&req));
    req.session().clear();
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::{http::Method, http::StatusCode, test::TestServer};
    use build_app;
    use lib::{db::test_pool, utils::extract_cookie};
    use models::{SigninForm, SignupForm};

    fn authenticated(srv: &mut TestServer, cookie: &str) {
        show(srv, cookie, "test000", StatusCode::NOT_FOUND);
    }

    fn not_authenticated(srv: &mut TestServer, cookie: &str) {
        show(srv, cookie, "test000", StatusCode::UNAUTHORIZED);
    }

    fn show(srv: &mut TestServer, cookie: &str, username: &str, expected: StatusCode) {
        let mut req = srv.client(Method::GET, &format!("/user/{}", username));
        opt_cookie!(req, cookie);

        let r = srv.execute(req.finish().unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);
    }

    fn signup(
        srv: &mut TestServer,
        username: &str,
        email: &str,
        password: &str,
        expected: StatusCode,
    ) -> String {
        let body = SignupForm {
            username: username.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
        };
        let req = srv.client(Method::POST, "/signup").json(body).unwrap();
        let r = srv.execute(req.send()).unwrap();
        assert_eq!(r.status(), expected);

        let cookie = extract_cookie(&r);

        if expected == StatusCode::OK {
            authenticated(srv, &cookie);
        }
        cookie
    }

    fn signin(srv: &mut TestServer, login: &str, password: &str, expected: StatusCode) {
        let body = SigninForm {
            login: login.to_owned(),
            password: password.to_owned(),
        };
        let req = srv.client(Method::POST, "/signin").json(body).unwrap();
        let r = srv.execute(req.send()).unwrap();
        assert_eq!(r.status(), expected);

        let cookie = extract_cookie(&r);

        if expected == StatusCode::OK {
            authenticated(srv, &cookie);
        }
    }

    fn logout(srv: &mut TestServer, cookie: &str) {
        let mut req = srv.client(Method::POST, "/logout");
        opt_cookie!(req, cookie);

        let r = srv.execute(req.finish().unwrap().send()).unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        let cookie = extract_cookie(&r);
        not_authenticated(srv, &cookie);
    }

    #[test]
    fn user() {
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        not_authenticated(&mut srv, "");
        show(&mut srv, "", "test", StatusCode::UNAUTHORIZED);

        // Conflict rolls-back test transaction, so we have to start pool again (and authenticate)
        // TODO: this should be fixed
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        let cookie = signup(
            &mut srv,
            "test",
            "test@example.com",
            "password",
            StatusCode::OK,
        );
        show(&mut srv, &cookie, "test", StatusCode::OK);
        signin(&mut srv, "test", "password", StatusCode::OK);
        signin(&mut srv, "test@example.com", "password", StatusCode::OK);

        show(&mut srv, &cookie, "test2", StatusCode::NOT_FOUND);

        signup(
            &mut srv,
            "test2",
            "test@example.com",
            "password",
            StatusCode::CONFLICT,
        );

        // Conflict rolls-back test transaction, so we have to start pool again (and authenticate)
        // TODO: this should be fixed
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        show(&mut srv, &cookie, "test2", StatusCode::NOT_FOUND);

        show(&mut srv, &cookie, "test2", StatusCode::NOT_FOUND);

        signup(
            &mut srv,
            "test2",
            "test@example.com2",
            "password",
            StatusCode::OK,
        );
        signin(&mut srv, "test2", "password", StatusCode::OK);
        show(&mut srv, &cookie, "test2", StatusCode::OK);

        signup(&mut srv, "", "aa", "aa", StatusCode::BAD_REQUEST);
        signup(&mut srv, "aa", "", "aa", StatusCode::BAD_REQUEST);
        signup(&mut srv, "aa", "aa", "", StatusCode::BAD_REQUEST);

        logout(&mut srv, &cookie);
        logout(&mut srv, "");

        signin(&mut srv, "test", "passwor", StatusCode::UNAUTHORIZED);
        signin(&mut srv, "tes", "password", StatusCode::UNAUTHORIZED);
        signin(&mut srv, "test", "", StatusCode::UNAUTHORIZED);
        signin(&mut srv, "", "password", StatusCode::UNAUTHORIZED);
        signin(&mut srv, "test3", "password", StatusCode::UNAUTHORIZED);

        signup(
            &mut srv,
            "test",
            "test@example.com2",
            "password",
            StatusCode::CONFLICT,
        );
    }
}

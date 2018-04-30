#[macro_export]
macro_rules! debug {
    ($($x: expr),*) => (debug!($($x,)*));
    ($($x: expr,)*) => ({print!("DEBUG: "); println!($($x,)*)});
}

#[macro_export]
macro_rules! error {
    ($($x: expr),*) => (error!($($x,)*));
    ($($x: expr,)*) => ({eprint!("ERROR "); eprintln!($($x,)*)});
}

#[macro_export]
macro_rules! get_user_id {
    ($state: expr) => (match ::lib::auth::user_id(&$state) {
        Some(uid) => uid,
        None => return ::lib::http::bad_request($state)
    });
}

#[macro_export]
macro_rules! assert_csrf {
    ($state: expr, $path_key: expr) => ({
        use lib::auth::is_csrf_valid;
        use gotham::state::FromState;
        use hyper::{Method, Post};
        if *Method::borrow_from(&mut $state) == Post {
            if !is_csrf_valid(&mut $state) {
                return redirect($state, url_for!($path_key));
            }
        }
    });
}

#[macro_export]
macro_rules! assert_auth {
    ($state: expr) => (assert_auth!($state, "signup"));
    ($state: expr, $path_key: expr) => ({
        use lib::auth::is_auth;

        assert_csrf!($state, $path_key);
        if !is_auth(&$state) {
            return redirect($state, url_for!($path_key));
        }
    });
}

#[macro_export]
macro_rules! __from_body {
    ($state: expr, $type: ty) => ({
        use middlewares::body::BodyData;
        use gotham::state::FromState;
        use serde_urlencoded::from_str;

        match BodyData::try_borrow_from($state) {
            Some(&BodyData(ref raw)) => {
                match from_str::<$type>(raw) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        error!("From body: {}", err);
                        None
                    }
                }
            },
            None => {
                error!("From body: no content");
                None
            }
        }
    });
}

#[macro_export]
macro_rules! try_from_body {
    ($state: expr, $error_route: expr, $type: ty) => ({
        assert_csrf!($state, $error_route);
        __from_body!(&mut $state, $type)
    });
}

#[macro_export]
macro_rules! from_body {
    ($state: expr, $error_route: expr, $type: ty) => ({
        use lib::http::bad_request;
        match try_from_body!($state, $error_route, $type) {
            Some(content) => content,
            None => return bad_request($state)
        }
    });
}

#[macro_export]
macro_rules! from_path {
    ($state: expr, $type: ty) => ({
        use gotham::state::FromState;
        IdPath::borrow_from(&$state)
    });
}

macro_rules! router {
    (($chain: expr, $pipeline: expr), $($key: expr => ($uri: expr, $method: expr, $route: expr)),*) => {
        hash_map!(($chain, $pipeline), $($key => ($uri, $method, $path),)*)
    };
    (($chain: expr, $pipeline: expr), $($key: expr => ($uri: expr, $method: expr, $route: expr),)*) => ({
        use lib::utils::REVERSE_ROUTE_TABLE;
        build_router($chain, $pipeline, |route| {
            $(
                match $method {
                    Get => route.get($uri).to($route),
                    Post => route.post($uri).to($route),
                    _ => panic!("Method {:?} not implemented in router.rs",
                                $method)
                }
                REVERSE_ROUTE_TABLE.write()
                                   .unwrap()
                                   .insert($key.to_owned(),
                                           $uri.to_owned());
            )*
        })
    });
}

#[macro_export]
macro_rules! url_for {
    ($key: expr) => ({
        use lib::utils::REVERSE_ROUTE_TABLE;
        match REVERSE_ROUTE_TABLE.read().unwrap().get($key) {
            Some(uri) => uri.to_owned(),
            None => {
                println!("URL with key {} doesn't exist, check routes.rs",
                         $key);
                "/".to_owned()
            }
        }
    });
}

#[macro_export]
macro_rules! try_db_empty {
    ($state: expr, $expr: expr) => ({
        use lib::http::internal_server_error;
        use diesel::NotFound;
        match $expr {
            Ok(value) => value,
            Err(NotFound) => Vec::new(),
            Err(_) => return internal_server_error($state)
        }
    });
}

#[macro_export]
macro_rules! try_db {
    ($state: expr, $expr: expr) => ({
        use lib::http::{not_found, internal_server_error};
        use diesel::{NotFound};
        match $expr {
            Ok(value) => value,
            Err(NotFound) => return not_found($state),
            Err(_) => return internal_server_error($state)
        }
    });
}

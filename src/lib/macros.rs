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
        assert_csrf!($state, $path_key);
        if !is_auth(&$state) {
            return redirect(state, url_for!($path_key));
        }
    });
}

#[macro_export]
macro_rules! __from_body {
    ($state: expr, $type: ty) => ({
        use middleware::body::BodyData;
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

macro_rules! router {
    (($chain: expr, $pipeline: expr), $($key: expr => ($uri: expr, $method: expr, $func: expr)),*) => {
        hash_map!(($chain, $pipeline), $($key => ($uri, $method, $func),)*)
    };
    (($chain: expr, $pipeline: expr), $($key: expr => ($uri: expr, $method: expr, $func: expr),)*) => ({
        use lib::utils::REVERSE_ROUTE_TABLE;
        build_router($chain, $pipeline, |router| {
            $(
                match $method {
                    Get => router.get($uri).to($func),
                    Post => router.post($uri).to($func),
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
macro_rules! query_one {
    ($state: expr, $exec: expr, $type: ty) => ({
        use lib::http::not_found;
        let values = query!($state, $exec, $type);
        if values.len() > 0 {
            values[0].clone()
        } else {
            return not_found($state);
        }
    });
}

#[macro_export]
macro_rules! query {
    ($state: expr, $exec: expr, $type: ty) => ({
        use middleware::db::Connection;
        use gotham::state::FromState;
        use lib::http::internal_server_error;
        let value = match Connection::try_borrow_from(&$state) {
            Some(&Connection(ref conn)) => match $exec.load::<$type>(&**conn) {
                Ok(v) => Some(v),
                Err(error) => {
                    error!("{:?}", error);
                    None
                }
            },
            None => None
        };
        match value {
            Some(v) => v,
            None => return internal_server_error($state)
        }
    });
}

#[macro_export]
macro_rules! insert {
    ($state: expr, $table: ident, $value: expr) => ({
        use middleware::db::Connection;
        use gotham::state::FromState;
        use diesel::insert_into;
        use lib::http::internal_server_error;
        let value = match Connection::try_borrow_from(&$state) {
            Some(&Connection(ref conn)) => match insert_into($table)
                .values(&$value)
                .returning(id)
                .get_results(&**conn) {
                    Ok(v) => Some(v[0]),
                    Err(error) => {
                        error!("{:?}", error);
                        None
                    }
            },
            None => None
        };
        match value {
            Some(v) => v,
            None => return internal_server_error($state)
        }
    });
}

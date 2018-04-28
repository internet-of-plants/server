#[macro_export]
macro_rules! debug {
    ($($x: expr),*) => (debug!($($x,)*));
    ($($x: expr,)*) => ({print!("DEBUG: "); println!($($x,)*)});
}

#[macro_export]
macro_rules! error {
    ($($x: expr),*) => (error!($($x,)*));
    ($($x: expr,)*) => ({print!("ERROR "); println!($($x,)*)});
}

#[macro_export]
macro_rules! assert_csrf {
    ($state: expr, $path_key: expr) => ({
        use lib::auth::is_csrf_valid;
        use gotham::state::FromState;
        use hyper::{Method, Post};
        if Method::take_from(&mut $state) == Post {
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
        use middlewares::body::BodyData;
        use gotham::state::FromState;
        use serde_urlencoded::from_str;
        {
            use middlewares::body::BodyData;
            use gotham::state::FromState;
            if let Some(BodyData(ref value)) = BodyData::try_take_from($state) {
                println!("utf8 {}", value);
                println!("{:?}", from_str::<$type>(value));
            }
        }

        match BodyData::try_take_from($state) {
            Some(BodyData(ref raw)) => match from_str::<$type>(raw) {
                Ok(value) => Some(value),
                Err(err) => {
                    error!("From body: {}", err);
                    None
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
macro_rules! from_body {
    ($state: expr, $error_route: expr, $type: ty) => ({
        assert_csrf!($state, $error_route);
        __from_body!(&mut $state, $type)
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
macro_rules! db {
    ($x: expr, $y: ty) => ({
        use db::connection;
        match $x.load::<$y>(&*connection()) {
            Ok(v) => v,
            Err(error) => {
                error!("{:?}", error);
                Vec::new()
            }
        }
    });
}

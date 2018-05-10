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
                return redirect($state, $path_key);
            }
        }
    });
}

#[macro_export]
macro_rules! assert_auth {
    ($state: expr) => (assert_auth!($state, url_for!("signin")));
    ($state: expr, $path_key: expr) => ({
        use lib::auth::is_auth;

        assert_csrf!($state, $path_key);
        if !is_auth(&$state) {
            return redirect($state, $path_key);
        }
    });
}

#[macro_export]
macro_rules! try_from_body {
    ($state: expr, $error_route: expr, $type: ident) => ({
        assert_csrf!($state, $error_route);
        ::lib::utils::from_body::<$type>(&mut $state)
    });
}

#[macro_export]
macro_rules! from_body {
    ($state: expr, $error_route: expr, $type: ident) => ({
        match try_from_body!($state, $error_route, $type) {
            Some(content) => content,
            None => return ::lib::http::bad_request($state)
        }
    });
}

#[macro_export]
macro_rules! from_path {
    ($state: expr, $type: ident) => ({
        use gotham::state::FromState;
        $type::take_from(&mut $state)
    });
}

#[macro_export]
macro_rules! route {
    ($key: expr => ($uri: expr, $method: expr, $route: expr)) => ({
        ::lib::utils::REVERSE_ROUTE_TABLE
            .write()
            .unwrap()
            .insert($key.to_owned(),
                    $uri.to_owned());
        match $method {
            Get => $route.get($uri),
            Post => $route.post($uri),
            _ => panic!("Method {:?} not implemented in router.rs",
                        $method)
        }
    });
}

#[macro_export]
macro_rules! url_for {
    ($key: expr) => ({
        use lib::utils::REVERSE_ROUTE_TABLE;
        match REVERSE_ROUTE_TABLE.read().unwrap().get($key) {
            Some(uri) => uri.to_owned(),
            None => {
                error!("URL with key {} doesn't exist, check routes.rs",
                         $key);
                "/".to_string()
            }
        }
    });
    ($key: expr, $($name: expr => $param: expr),*) => (url_for!($key, $($name => $param,)*));
    ($key: expr, $($name: expr => $param: expr,)*) => ({
        let mut hm: HashMap<String, String> = HashMap::new();
        $(hm.insert($name, $param);)*
        url_for!($key, hm)
    });
    ($key: expr, $map: expr) => ({
        let path = url_for!($key);
        let mut path_split = path[1..].split('/');

        let mut path_out = String::new();
        let mut wildcard = String::new();

        let len = $map.len();
        for index in 0..len {
            let index = format!("_{}", index);
            match $map.get(&index) {
                Some(value) => {
                    wildcard.push('/');
                    wildcard.push_str(&value);
                },
                None => break
            }
            $map.remove(&index);
        }

        let mut exit = false;
        for (_, param) in $map {
            // TODO: accept get parameters
            loop {
                if let Some(part) = path_split.next() {
                    if part.len() == 0 {
                        continue;
                    }

                    let (c, _) = part.split_at(1);
                    if c == "*" {
                        exit = true;
                        break
                    } else if c == ":" {
                        path_out.push('/');
                        path_out.push_str(&param);
                        break;
                    } else {
                        path_out.push('/');
                        path_out.push_str(part);
                    }
                } else {
                    break;
                }
            }

            if exit {
                break;
            }
        }

        loop {
            match path_split.next() {
                Some(part) => {
                    let (c, _) = part.split_at(1);
                    if c == "*" || c == ":" {
                        break;
                    }

                    path_out.push('/');
                    path_out.push_str(&part);
                },
                None => break
            }
        }

        path_out.push_str(&wildcard);
        path_out
    });
}

#[macro_export]
macro_rules! try_db {
    ($state: expr, $expr: expr) => (try_db!($state, $expr, return ::lib::http::not_found($state)));
    ($state: expr, $expr: expr, $default: expr) => ({
        use lib::http::internal_server_error;
        use diesel::NotFound;
        match $expr {
            Ok(value) => value,
            Err(NotFound) => $default,
            Err(err) => {
                error!("{:?}", err);
                return internal_server_error($state)
            }
        }
    });
}

#[macro_export]
macro_rules! try_db_option {
    ($state: expr, $expr: expr) => ({
        use lib::http::internal_server_error;
        use diesel::NotFound;
        match $expr {
            Ok(value) => Some(value),
            Err(NotFound) => None,
            Err(err) => {
                error!("{:?}", err);
                return internal_server_error($state)
            }
        }
    });
}

#[macro_export]
macro_rules! try_get_num {
    ($values: expr, $name: expr, $type: tt) => ({
        use std::str::FromStr;
        match $values.get(stringify!($name)) {
            Some(value) => match $type::from_str(value) {
                Ok(value) => value,
                Err(_) => return None
            },
            None => return None
        }
    });
}

#[macro_export]
macro_rules! skip_newline {
    ($content: expr, end) => (while $content.len() > 2 && &$content[$content.len() - 2..] == "\r\n".as_bytes() {
        $content = &$content[..$content.len() - 2];
    });
    ($content: expr, start) => (while $content.len() > 2 && &$content[..2] == "\r\n".as_bytes() {
        $content = &$content[2..];
    });
    ($content: expr) => (skip_newline!($content, start));
}

#[macro_export]
macro_rules! save_file {
    ($state: expr, $old: expr, $new: expr) => ({
        use lib::utils::{multipart_boundary, parse_multipart_files};
        use middlewares::body::{BodyData, Multipart};
        use gotham::state::FromState;
        let boundary = multipart_boundary(&$state);
        let ret: Option<fn(State) -> (State, Response)> = match (BodyData::try_borrow_from(&$state), boundary) {
            (Some(&BodyData(Multipart(ref raw))), Some(ref boundary)) => {
                match parse_multipart_files(raw, boundary.as_bytes()).get($old) {
                    Some(ref content) => {
                        ::lib::image::save_image(content, $new).unwrap();
                        None
                    },
                    None => Some(::lib::http::internal_server_error)
                }
            }
            _ => Some(::lib::http::bad_request)
        };

        if let Some(ret) = ret {
            return ret($state);
        }
    });
}

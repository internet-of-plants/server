#[macro_export]
/// Implement From<Error> trait and ResponseError for crate error
macro_rules! impl_err {
    ($name: ident; $($variant: ident($error: ty)),*) => {
        impl_err! { $name; $($variant($error),)* }
    };
    ($name: ident; $($variant: ident($error: ty),)*) => {
        $(impl From<$error> for $name {
            fn from(err: $error) -> Self {
                $name::$variant(err)
            }
        })*

        impl ::actix_web::ResponseError for $name {
            fn error_response(&self) -> ::actix_web::HttpResponse {
                self.response_type().error_response()
            }
        }
    }
}

#[macro_export]
/// Register route with url_for
macro_rules! route {
    ($method:expr, $func:expr) => {
        |r| {
            r.name(stringify!($func));
            r.method($method).with($func);
        }
    };
}

#[macro_export]
/// Return error if field is empty
macro_rules! filled {
    ($str:expr) => {{
        if $str == "" {
            return Err(::lib::error::Error::InvalidData);
        }
        $str
    }};
}

#[macro_export]
/// Extract DB connection from pool
macro_rules! conn {
    ($pool:expr) => {
        &*(::lib::db::connection(&$pool)?)
    };
}

#[cfg(test)]
#[macro_export]
/// Set cookie optionally
macro_rules! opt_cookie {
    ($req:expr, $cookie:expr) => {{
        use actix_web::http::Cookie;
        if let Ok(cookie) = Cookie::parse($cookie) {
            $req.cookie(cookie);
        }
    }};
}

#[cfg(test)]
#[macro_export]
/// Retrieve header value as String
macro_rules! header {
    ($req:expr, $header:expr) => {{
        $req.headers().get($header).unwrap().to_str().unwrap()
    }};
}

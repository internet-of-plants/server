pub mod api;
pub mod controllers;
pub mod error;
pub mod models;
pub mod utils;

use crate::prelude::*;

pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::models::*;
    pub use crate::{api, controllers, utils};
    #[allow(unused_imports)]
    pub use log::{debug, error, info, trace, warn};
    pub use sqlx::prelude::*;
    pub use tokio::prelude::*;
    pub use warp::{http::StatusCode, Filter, Rejection, Reply};
}

#[tokio::main]
async fn main() {
    //#[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    //#[cfg(debug_assertions)]
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            "server=info,warp=info,event=info,now=info,timer=info",
        );
    }

    pretty_env_logger::init();

    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop";
    utils::run_migrations(url).await;

    let pool = Pool::new(url).await.expect("Unable to connect to database");
    let pool: &'static Pool = Box::leak(Box::new(pool));
    let pool = warp::any().map(move || pool);

    let auth = warp::any().and(warp::header::optional("Authorization"))
        .and(pool)
        .and_then(|token: Option<String>, pool: &'static Pool| async move {
            match token {
                Some(mut token) if token.starts_with("Basic ") => {
                    token.drain(.."Basic ".len());
                    api::user::authenticate(pool, token).await
                      .map_err(warp::Rejection::from)
                }
                _ => Err(warp::Rejection::from(Error::Forbidden))
            }
        });

    let log = warp::log::custom(utils::http_log);

    let routes = warp::any()
        .and(warp::path("user")
            .and(warp::path("login")
                .and(warp::path::end())
                .and(warp::post())
                .and(pool)
                .and(warp::body::content_length_limit(1024))
                .and(warp::body::json())
                .and_then(controllers::user::login)
            .or(warp::path::end()
                .and(warp::post())
                .and(pool)
                .and(warp::body::content_length_limit(1024))
                .and(warp::body::json())
                .and_then(controllers::user::new))))
        .or(warp::path("plant")
            .and(warp::path("index")
                .and(warp::path::end())
                .and(warp::get())
                .and(pool)
                .and(auth)
                .and_then(controllers::plant::index)
            .or(warp::path("history")
                .and(warp::path::end())
                .and(warp::get())
                .and(pool)
                .and(auth)
                .and(warp::query::query())
                .and_then(controllers::plant::history))
            .or(warp::path("owns")
                .and(warp::path::end())
                .and(warp::post())
                .and(pool)
                .and(auth)
                .and(warp::body::content_length_limit(1024))
                .and(warp::body::json())
                .and_then(controllers::plant::owns))
            .or(warp::path::end()
                .and(warp::get())
                .and(pool)
                .and(auth)
                .and(warp::query::query())
                .and_then(controllers::plant::get))
            .or(warp::path::end()
                .and(warp::put())
                .and(pool)
                .and(auth)
                .and(warp::body::content_length_limit(1024))
                .and(warp::body::json())
                .and_then(controllers::plant::put))))
        .or(warp::path("event")
            .and(warp::path::end())
            .and(warp::post())
            .and(pool)
            .and(auth)
            .and(warp::body::content_length_limit(1024))
            .and(warp::body::json())
            .and_then(controllers::event::new))
        .or(warp::path("error")
            .and(warp::path("index")
                .and(warp::path::end())
                .and(warp::get())
                .and(pool)
                .and(auth)
                .and_then(controllers::error::index)
            .or(warp::path::end()
                .and(warp::post())
                .and(pool)
                .and(auth)
                .and(warp::body::content_length_limit(1024))
                .and(warp::body::json())
                .and_then(controllers::error::new)
            .or(warp::path::end()
                .and(warp::delete())
                .and(pool)
                .and(auth)
                .and(warp::query::query())
                .and_then(controllers::error::solve)))))
        .with(log)
        .with(
            warp::cors()
                .allow_origins(vec!["http://localhost:3002", "https://internet-of-plants.github.io"])
                .allow_credentials(false)
                .allow_headers(vec!["Authorization", "Content-Type"])
                .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS", "PUT"]),
        )
        .recover(Error::handle);

    let server = warp::serve(routes);

    #[cfg(not(debug_assertions))]
    let server = server
        .tls()
        .cert_path("cert.pem")
        .key_path("privkey.pem");

    server.run(([0, 0, 0, 0], 4001)).await;
}

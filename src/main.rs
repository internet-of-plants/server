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
    pub use tokio::io::AsyncWriteExt;
    pub use warp::{http::StatusCode, Filter, Rejection, Reply};
}

#[tokio::main]
async fn main() {
    //#[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if std::env::var("RUST_LOG").is_err() {
        #[cfg(not(debug_assertions))]
        let val = "server=debug,tracing=info,hyper=info,warp=debug,event=info,now=info,timer=info";
        #[cfg(debug_assertions)]
        let val = "server=trace,tracing=trace,hyper=trace,warp=trace,event=trace,now=trace,timer=trace";
        std::env::set_var("RUST_LOG", val);
    }

    pretty_env_logger::init();

    info!("RUST_LOG is {}", std::env::var("RUST_LOG").ok().unwrap_or_else(String::new));

    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop";
    utils::run_migrations(url).await;

    let pool = Pool::connect(url).await.expect("Unable to connect to database");
    let pool: &'static Pool = Box::leak(Box::new(pool));
    let pool = warp::any().map(move || pool);

    let auth = warp::any()
        .and(warp::header::optional("Authorization"))
        .and(pool)
        .and_then(|token: Option<String>, pool: &'static Pool| async move {
            match token {
                Some(mut token) if token.starts_with("Basic ") => {
                    token.drain(.."Basic ".len());
                    api::user::authenticate(pool, token)
                        .await
                        .map_err(warp::Rejection::from)
                }
                _ => Err(warp::Rejection::from(Error::Forbidden)),
            }
        });

    let log = warp::log::custom(utils::http_log);

    let routes = warp::any()
        .and(warp::path("v1"))
        .and(
            warp::path("user").and(
                warp::path("login")
                    .and(warp::path::end())
                    .and(warp::post())
                    .and(pool)
                    .and(warp::body::content_length_limit(1024))
                    .and(warp::body::json())
                    .and(warp::filters::header::optional("MAC_ADDRESS"))
                    .and_then(controllers::user::login)
                .or(warp::path::end()
                    .and(warp::post())
                    .and(pool)
                    .and(warp::body::content_length_limit(1024))
                    .and(warp::body::json())
                    .and_then(controllers::user::new)),
            )
            .or(warp::path("plant").and(
                warp::path("index")
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
                .or(warp::path::end()
                    .and(warp::get())
                    .and(pool)
                    .and(auth)
                    .and(warp::query::query())
                    .and_then(controllers::plant::get)))
            )
            .or(warp::path("event")
                .and(warp::path::end())
                .and(warp::post())
                .and(pool)
                .and(auth)
                .and(warp::body::content_length_limit(1024))
                .and(warp::body::json())
                .and(warp::header::headers_cloned())
                .and_then(controllers::event::new))
            .or(warp::path("panic")
                .and(warp::path("index")
                    .and(warp::path::end())
                    .and(warp::get())
                    .and(pool)
                    .and(auth)
                    .and_then(controllers::device_panic::index)
                .or(warp::path::end()
                    .and(warp::post())
                    .and(pool)
                    .and(auth)
                    .and(warp::body::content_length_limit(2048))
                    .and(warp::body::json())
                    .and(warp::filters::header::header("MAC_ADDRESS"))
                    .and_then(controllers::device_panic::new))
                .or(warp::path::end()
                    .and(warp::delete())
                    .and(pool)
                    .and(auth)
                    .and(warp::query::query())
                    .and_then(controllers::device_panic::solve))),
            )
            .or(warp::path("log")
                .and(warp::path("index")
                    .and(warp::path::param())
                    .and(warp::path::end())
                    .and(warp::get())
                    .and(pool)
                    .and(auth)
                    .and_then(controllers::device_log::index)
                .or(warp::path::end()
                    .and(warp::post())
                    .and(pool)
                    .and(auth)
                    .and(warp::body::content_length_limit(2048))
                    .and(warp::body::bytes())
                    .and(warp::filters::header::header("MAC_ADDRESS"))
                    .and_then(controllers::device_log::new)),
            ))
            .or(warp::path("update")
                .and(warp::path::end()
                    .and(warp::get())
                    .and(pool)
                    .and(auth)
                    .and(warp::header::headers_cloned())
                    .and_then(controllers::update::get))
                .or(warp::path::param()
                    .and(warp::path::end())
                    .and(warp::post())
                    .and(pool)
                    .and(auth)
                    // 1 MB max size
                    .and(warp::filters::multipart::form().max_length(1024 * 1024))
                    .and_then(controllers::update::new)))
                /*
                .or(warp::path("index")
                    .and(warp::path::end())
                    .and(warp::get())
                    .and(pool)
                    .and(auth)
                    .and_then(controllers::update::index)
                    .or(warp::path::param()
                    .and(warp::path::end())
                    .and(warp::post())
                    .and(pool)
                    .and(auth)
                    .and(warp::body::content_length_limit(1024))
                    .and(warp::body::bytes())
                    .and_then(controllers::update::get)))
                */
        )
        .with(log)
        .with(
            warp::cors()
                .allow_origins(vec![
                    "http://127.0.0.1:8080",
                    "http://localhost:8080",
                    "https://internet-of-plants.github.io",
                ])
                .allow_credentials(false)
                .allow_headers(vec!["Authorization", "Content-Type"])
                .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS", "PUT"]),
        )
        .recover(Error::handle);

    let server = warp::serve(routes);

    #[cfg(not(debug_assertions))]
    let server = server.tls().cert_path("cert.pem").key_path("privkey.pem");

    server.run(([0, 0, 0, 0], 4001)).await;
}

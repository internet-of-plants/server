extern crate base64;

#[cfg(test)]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate slog;
extern crate env_logger;
extern crate slog_async;
extern crate slog_term;

extern crate actix;
extern crate actix_web;
//extern crate openssl;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;

extern crate hex;
extern crate rand;
extern crate sodiumoxide;

extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slugify;

extern crate image;

use actix::System;
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::middleware::{Logger as ActixLogger, cors::Cors};
use actix_web::{server, App, fs::StaticFiles, http::Method};
//use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use slog::Logger as SlogLogger;
use std::{env, process::exit, sync::RwLock};

#[cfg(not(test))]
use slog::Drain;
#[cfg(not(test))]
use slog_async::Async;
#[cfg(not(test))]
use slog_term::FullFormat;

#[macro_use]
mod lib;
#[macro_use]
mod models;
mod config;
mod controllers;

use config::{HOST, REQUEST_SIZE_LIMIT, STATIC_PATH};
use controllers::*;
use lib::db::{pool, DbPool};

#[cfg(release)]
use config::LOG_PATH;

lazy_static! {
    /// During release uses log file, dev uses terminal, test ignore logs
    pub static ref LOG: RwLock<SlogLogger> = {
        #[cfg(not(test))]
        let drain = {
            #[cfg(release)]
            let decorator = {
                use slog_term::PlainDecorator;
                use std::fs::OpenOptions;
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(false)
                    .open(LOG_PATH)
                    .unwrap();
                PlainDecorator::new(file);
            };

            #[cfg(not(release))]
            let decorator = {
                use slog_term::TermDecorator;
                TermDecorator::new().build()
            };

            let drain = FullFormat::new(decorator).build().fuse();
            Async::new(drain).build().fuse()
        };

        #[cfg(test)]
        let drain = slog::Discard;

        RwLock::new(SlogLogger::root(drain, o!()))
    };
}

pub struct State {
    pub pool: DbPool,
    pub log: SlogLogger,
}

/// Generate server app instance
fn build_app(key: &'static [u8; 32], pool: DbPool) -> impl Fn() -> App<State> {
    move || {
        let cookie_backend = CookieSessionBackend::private(key).name("s").secure(false); // TODO: TLS .secure(true);
        let session_storage = SessionStorage::new(cookie_backend);

        App::with_state(State {
            pool: pool.clone(),
            log: LOG.read().unwrap().clone(),
        }).middleware(session_storage)
            .middleware(ActixLogger::new("%t %a %r %s %b %D %{User-Agent}i"))
            .handler("/static", StaticFiles::new(STATIC_PATH))
            .configure(|app| {
                Cors::for_app(app)
                    .supports_credentials()
                    .resource("/plant", route!(Method::POST, plant_post))
                    .resource("/plant/{id}", route!(Method::GET, plant))
                    .resource("/plants", route!(Method::GET, plant_index))
                    .resource("/plant_type", |r| {
                        r.name("plant_type_post");
                        r.method(Method::POST)
                            .with(plant_type_post)
                            .0
                            .limit(REQUEST_SIZE_LIMIT);
                    })
                    .resource("/plant_type/{slug}", route!(Method::GET, plant_type))
                    .resource("/plant_types", route!(Method::GET, plant_type_index))
                    .resource("/user/{username}", route!(Method::GET, user))
                    .resource("/signup", route!(Method::POST, signup))
                    .resource("/signin", route!(Method::POST, signin))
                    .resource("/logout", route!(Method::POST, logout))
                    .resource("/plant/{id}/events", route!(Method::GET, event_index))
                    .resource("/event", route!(Method::POST, event_post))
                    .register()
            })
    }
}

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .default_format_timestamp(false)
        .init();

    let sys = System::new("iop");

    /*
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("dependencies/ssl.key", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("dependencies/ssl.crt")
        .unwrap();
    */

    // TODO: use actual key
    server::new(build_app(&[0; 32], pool()))
        //.bind_ssl(HOST, builder)
        .bind(HOST)
        .unwrap()
        .start();
    println!("Listening to {}", HOST);

    exit(sys.run());
}

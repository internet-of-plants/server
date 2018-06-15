#[macro_use]
extern crate lazy_static;
extern crate actix_web;

use actix_web::{server, App};
use std::{env::var, process::exit};

lazy_static! {
    static ref AUTH: String = var("AUTH_TOKEN").unwrap();
}

fn main() {
    let port = match var("WATCHER_PORT") {
        Ok(ref port) => port.clone(),
        _ => "4000".to_owned(),
    };

    if *AUTH == String::new() {
        loop {}
    }

    server::new(|| {
        App::new().resource("/{auth}", |r| {
            r.f(|req| match req.match_info().get("auth") {
                Some(token) if token == &*AUTH => exit(0),
                _ => "",
            })
        })
    }).bind(&format!("0.0.0.0:{}", port))
        .unwrap()
        .run();
}

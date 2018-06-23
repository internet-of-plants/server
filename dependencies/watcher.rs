#[macro_use]
extern crate lazy_static;
extern crate actix_web;

use actix_web::{server, App};
use std::{env::var, process::exit};

lazy_static! {
    static ref AUTH: String = var("AUTH_TOKEN")
        .expect("Expected AUTH_TOKEN env var, to authenticate the deploy by the CI");
}

fn main() {
    let port = match var("WATCHER_PORT") {
        Ok(ref port) => port.clone(),
        _ => "4000".to_owned(),
    };

    if *AUTH == String::new() {
        println!("AUTH_TOKEN is an empty string, watcher disabled, this will now loop forever");
        loop {}
    }

    println!(
        "Starting watcher, listening at 0.0.0.0:{} with AUTH_TOKEN {}",
        port, *AUTH
    );
    server::new(|| {
        App::new().resource("/{auth}", |r| {
            r.f(|req| {
                println!("Received request: {}", req.path());
                println!("Expected AUTH_TOKEN: {}", *AUTH);
                match req.match_info().get("auth") {
                    Some(token) if token == &*AUTH => exit(0),
                    _ => {}
                }
                println!("Deploy failed, watcher will keep running until receives the correct token to exit");
                ""
            })
        })
    }).bind(&format!("0.0.0.0:{}", port))
        .unwrap()
        .run();
}

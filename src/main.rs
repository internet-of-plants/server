extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;

#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;

#[macro_use]
extern crate serde_derive;
extern crate serde_urlencoded;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate tera;

extern crate futures;

extern crate hex;
extern crate sodiumoxide;
extern crate rand;

mod schema;
#[macro_use]
mod lib;
mod middlewares;
#[macro_use]
mod db;
#[macro_use]
mod router;
mod controllers;
mod models;
mod forms;

use router::router;

fn main() {
    let addr = "127.0.0.1:2000";
    println!("Listening at http://{}", addr);
    gotham::start(addr, router());
}

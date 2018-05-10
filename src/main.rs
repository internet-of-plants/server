extern crate image;

extern crate proc_macro;

extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;

#[macro_use]
extern crate serde_derive;
extern crate serde_urlencoded;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate slugify;
#[macro_use]
extern crate tera;

extern crate futures;

extern crate hex;
extern crate rand;
extern crate sodiumoxide;

mod schema;
#[macro_use]
mod lib;
mod middlewares;
mod router;
#[macro_use]
mod models;
mod controllers;
mod forms;

use router::router;

fn main() {
    let addr = "127.0.0.1:2000";
    println!("Listening at http://{}", addr);
    gotham::start(addr, router());
}

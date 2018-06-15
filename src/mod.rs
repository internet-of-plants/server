#![cfg_attr(release, deny(warnings))]

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

extern crate base64;

#[macro_use]
extern crate diesel;
#[cfg(not(release))]
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

#[macro_use]
pub mod lib;
#[macro_use]
pub mod models;
pub mod config;
pub mod controllers;

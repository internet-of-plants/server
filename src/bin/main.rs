extern crate env_logger;
extern crate iop;

use iop::lib::utils::start;
use std::env;

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::Builder::from_default_env()
        .default_format_module_path(false)
        .default_format_timestamp(false)
        .init();
    start();
}

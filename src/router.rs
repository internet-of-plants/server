use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::middleware::session::NewSessionMiddleware;
use hyper::{Get, Post};
use middlewares::body::BodyMiddleware;
use lib::auth::Session;
use controllers;

pub fn router() -> Router {
    let (chain, pipeline) = single_pipeline(new_pipeline()
        .add(NewSessionMiddleware::default()
                .with_session_type::<Option<Session>>()
                .insecure())
        .add(BodyMiddleware)
        .build());

    router! {
        (chain, pipeline),
        "home" => ("/", Get, controllers::home),
        "signup" => ("/signup", Get, controllers::signup),
        "signup_post" => ("/signup", Post, controllers::signup_post),
        "logs" => ("/logs", Get, controllers::logs),
    }
}

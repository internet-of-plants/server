use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::middleware::session::NewSessionMiddleware;
use hyper::{Get, Post};
use middlewares::body::BodyMiddleware;
use middlewares::db::DbMiddleware;
use lib::auth::Session;
use controllers;

pub fn router() -> Router {
    let (chain, pipeline) = single_pipeline(new_pipeline()
        .add(NewSessionMiddleware::default()
                .with_session_type::<Option<Session>>()
                .insecure())
        .add(BodyMiddleware)
        .add(DbMiddleware)
        .build());

    router! {
        (chain, pipeline),
        "home" => ("/", Get, controllers::plant_post),

        "signup" => ("/signup", Get, controllers::signup),
        "signup_post" => ("/signup", Post, controllers::signup_post),
        "signin" => ("/signin", Get, controllers::signin),
        "signin_post" => ("/signin", Post, controllers::signin_post),
        "logout" => ("/logout", Get, controllers::logout),

        "plants" => ("/plants", Get, controllers::plant_index),
        "plant_post" => ("/plant", Post, controllers::plant_post),
        "plant_type_post" => ("/plant_type", Post, controllers::plant_type_post),

        "logs" => ("/logs", Get, controllers::logs),
    }
}

use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::middleware::session::NewSessionMiddleware;
use hyper::{Get, Post};
use middlewares::body::BodyMiddleware;
use lib::auth::Session;
use controllers;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct IdPath {
    pub id: i32,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct SlugPath {
    pub slug: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender, Debug)]
pub struct PathExtractor {
    #[serde(rename = "*")] pub parts: Vec<String>,
}

pub fn router() -> Router {
    let (chain, pipeline) = single_pipeline(
        new_pipeline()
            .add(
                NewSessionMiddleware::default()
                    .with_session_type::<Option<Session>>()
                    .insecure(),
            )
            .add(BodyMiddleware)
            .build(),
    );

    build_router(chain, pipeline, |r| {
        route!("home" => ("/", Get, r)).to(controllers::plant_index);

        route!("signup" => ("/signup", Get, r)).to(controllers::signup);
        route!("signup_post" => ("/signup", Post, r)).to(controllers::signup_post);
        route!("signin" => ("/signin", Get, r)).to(controllers::signin);
        route!("signin_post" => ("/signin", Post, r)).to(controllers::signin_post);
        route!("logout" => ("/logout", Post, r)).to(controllers::logout);

        route!("user" => ("/user/:slug:[a-zA-Z0-9_-]+", Get, r))
            .with_path_extractor::<SlugPath>()
            .to(controllers::user);

        route!("plant" => ("/plant/:id:[0-9]+", Get, r))
            .with_path_extractor::<IdPath>()
            .to(controllers::plant);

        route!("create_plant_type" => ("/create_plant_type", Get, r))
            .to(controllers::create_plant_type);

        route!("plant_type" => ("/plant_type/:slug:[a-zA-Z0-9_-]+", Get, r))
            .with_path_extractor::<SlugPath>()
            .to(controllers::plant_type);

        route!("plants" => ("/plants", Get, r)).to(controllers::plant_index);
        route!("plant_post" => ("/plant", Post, r)).to(controllers::plant_post);
        route!("plant_type_post" => ("/plant_type", Post, r)).to(controllers::plant_type_post);

        route!("event_post" => ("/event", Post, r)).to(controllers::event_post);
        route!("events" => ("/events", Get, r)).to(controllers::event_index);

        route!("static" => ("/static/*", Get, r))
            .with_path_extractor::<PathExtractor>()
            .to(controllers::static_file);
    })
}

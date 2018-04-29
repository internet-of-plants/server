pub mod user;

pub use self::user::*;

use gotham::state::State;
use lib::http::render_template;
use hyper::Response;
use tera::Context;

pub fn home(state: State) -> (State, Response) {
    render_template(state, "home.html", &mut Context::new())
}

use schema::events::dsl::*;
use diesel::prelude::*;
use models::Event;

pub fn logs(state: State) -> (State, Response) {
    let mut ctx = Context::new();
    let event_vec = query!(state, events.filter(timestamp.gt(0)), Event);
    ctx.add("events", &event_vec);

    render_template(state, "logs.html", &mut ctx)
}

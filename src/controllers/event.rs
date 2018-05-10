use gotham::state::State;
use lib::db::connection;
use hyper::Response;
use tera::Context;

use diesel::prelude::*;
use diesel::insert_into;
use schema::{events, plants};
use models::{EventView, NewEvent};
use forms::EventForm;
use lib::http::{bad_request, render_template, success};
use lib::utils::from_body;

pub fn event_index(state: State) -> (State, Response) {
    let mut ctx = Context::new();

    let event_vec = try_db!(
        state,
        events::table
            .inner_join(plants::table)
            .filter(plants::user_id.eq(get_user_id!(state)))
            .select(EventViewSql!())
            .load::<EventView>(&*connection()),
        Vec::new()
    );
    ctx.add("events", &event_vec);

    render_template(state, "events.html", &mut ctx)
}

pub fn event_post(mut state: State) -> (State, Response) {
    let form = match from_body::<EventForm>(&mut state) {
        Some(form) => form,
        None => return bad_request(state),
    };

    let event = NewEvent {
        plant_id: form.pid,
        air_temperature_celsius: form.at,
        air_humidity_percentage: form.ah,
        soil_temperature_celsius: form.st,
        soil_resistivity: form.sr,
        light: form.l,
        device_timestamp: form.t,
    };

    try_db!(
        state,
        insert_into(events::table)
            .values(&event)
            .execute(&*connection())
    );

    success(state)
}

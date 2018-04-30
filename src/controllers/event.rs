use gotham::state::State;
use lib::http::render_template;
use lib::db::connection;
use hyper::Response;
use tera::Context;

use schema::{events, plants, plant_types, users};
use diesel::prelude::*;
use models::EventView;

pub fn event_index(state: State) -> (State, Response) {
    let mut ctx = Context::new();

    let event_vec = try_db_empty!(state, events::table
         .inner_join(plants::table.on(events::plant_id.eq(plants::id)))
         .inner_join(plant_types::table.on(plants::type_id.eq(plant_types::id)))
         .inner_join(users::table.on(plants::user_id.eq(users::id)))
         .filter(users::id.eq(get_user_id!(state)))
         .select((events::id, (plants::all_columns),
                  events::air_temperature_celsius,
                  events::air_humidity_percentage,
                  events::soil_temperature_celsius,
                  events::soil_resistivity,
                  events::light, events::timestamp))
         .load::<EventView>(&*connection()));
    ctx.add("events", &event_vec);

    render_template(state, "events.html", &mut ctx)
}

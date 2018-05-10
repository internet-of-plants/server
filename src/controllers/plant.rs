use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::{Event, NewPlant, PlantTypeView, PlantView};
use forms::PlantForm;
use lib::http::{redirect, render_template};
use lib::db::connection;

use diesel::prelude::*;
use diesel::insert_into;
use schema::{events, plant_types, plants, users};
use router::IdPath;

pub fn plant(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let id = from_path!(state, IdPath).id;
    let user_id = get_user_id!(state);

    let plant = try_db!(
        state,
        plants::table
            .inner_join(plant_types::table)
            .inner_join(users::table)
            .filter(plants::id.eq(id).and(users::id.eq(user_id)))
            .select(PlantViewSql!())
            .first::<PlantView>(&*connection())
    );
    let last_event = try_db_option!(
        state,
        events::table
            .filter(events::plant_id.eq(plant.id))
            .first::<Event>(&*connection())
    );

    let mut ctx = Context::new();
    ctx.add("plant", &plant);
    ctx.add("last_event", &last_event);
    render_template(state, "plant.html", &mut ctx)
}

pub fn plant_index(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let (plant_vec, plant_type_vec) = {
        let conn = connection();

        let plant_vec = try_db!(
            state,
            plants::table
                .inner_join(plant_types::table)
                .inner_join(users::table)
                .filter(users::id.eq(get_user_id!(state)))
                .select(PlantViewSql!())
                .load::<PlantView>(&*conn),
            Vec::new()
        );

        let plant_type_vec = try_db!(
            state,
            plant_types::table
                .inner_join(users::table)
                .select(PlantTypeViewSql!())
                .load::<PlantTypeView>(&*conn),
            Vec::new()
        );

        (plant_vec, plant_type_vec)
    };

    let mut ctx = Context::new();
    ctx.add("plants", &plant_vec);
    ctx.add("plant_types", &plant_type_vec);
    render_template(state, "plants.html", &mut ctx)
}

pub fn plant_post(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let form = from_body!(state, url_for!("plants"), PlantForm);
    let plant = NewPlant {
        name: form.name,
        type_id: form.type_id,
        user_id: get_user_id!(state),
    };

    try_db!(
        state,
        insert_into(plants::table)
            .values(&plant)
            .execute(&*connection())
    );

    redirect(state, url_for!("plants"))
}

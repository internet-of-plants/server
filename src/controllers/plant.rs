use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::{NewPlant, PlantView, PlantTypeView};
use forms::PlantForm;
use lib::http::{render_template, redirect};
use lib::db::connection;

use diesel::prelude::*;
use diesel::insert_into;
use schema::plants;
use schema::plant_types;
use schema::users;
use router::IdPath;

pub fn plant(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let id = from_path!(state, IdPath).id;

    let plant = try_db!(state, plants::table
            .inner_join(plant_types::table.on(plants::type_id.eq(plant_types::id)))
            .inner_join(users::table.on(plants::user_id.eq(users::id)))
            .filter(plants::id.eq(id))
            .select((plants::id, plants::name, (plant_types::all_columns), (users::all_columns)))
            .first::<PlantView>(&*connection()));

    let mut ctx = Context::new();
    ctx.add("plant", &plant);
    render_template(state, "plant.html", &mut ctx)
}

pub fn plant_index(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let (plant_vec, plant_type_vec) = {
        let conn = connection();

        let plant_vec = try_db_empty!(state, plants::table
            .inner_join(plant_types::table.on(plants::type_id.eq(plant_types::id)))
            .inner_join(users::table.on(plants::user_id.eq(users::id)))
            .filter(users::id.eq(get_user_id!(state)))
            .select((plants::id, plants::name, (plant_types::all_columns), (users::all_columns)))
            .load::<PlantView>(&*conn));

        let plant_type_vec = try_db_empty!(state, plant_types::table
            .inner_join(users::table.on(plant_types::user_id.eq(users::id)))
            .select((plant_types::id, plant_types::name, plant_types::slug, (users::all_columns)))
            .load::<PlantTypeView>(&*conn));

        (plant_vec, plant_type_vec)
    };

    let mut ctx = Context::new();
    ctx.add("plants", &plant_vec);
    ctx.add("plant_types", &plant_type_vec);
    render_template(state, "plants.html", &mut ctx)
}

pub fn plant_post(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let form = from_body!(state, "plants", PlantForm);
    let plant = NewPlant {
        name: form.name,
        type_id: form.type_id,
        user_id: get_user_id!(state)
    };

    try_db!(state, insert_into(plants::table)
        .values(&plant)
        .execute(&*connection()));

    redirect(state, url_for!("plants"))
}

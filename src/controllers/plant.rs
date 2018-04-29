use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::{NewPlant, Plant};
use forms::PlantForm;
use lib::http::{render_template, redirect};

use diesel::prelude::*;
use schema::plants::dsl::*;

pub fn plant_index(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let uid = get_user_id!(state);
    let plant_vec = query!(state, plants.filter(user_id.eq(uid)), Plant);

    let mut ctx = Context::new();
    ctx.add("plants", &plant_vec);
    render_template(state, "plants.html", &mut ctx)
}

pub fn plant_post(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let form = from_body!(state, "plants", PlantForm);
    let plant = NewPlant {
        name: form.name,
        type_slug: form.type_slug,
        user_id: get_user_id!(state)
    };

    let _plant = insert!(state, plants, plant, Plant);

    redirect(state, url_for!("plants"))
}

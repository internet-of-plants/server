use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::{NewPlantType, PlantType};
use forms::PlantTypeForm;
use lib::http::{render_template, redirect};

use diesel::prelude::*;
use schema::plant_types::dsl::*;

/*
pub fn plant_type_index(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let plant_type_vec = query!(state, plant_types, PlantType);

    let mut ctx = Context::new();
    ctx.add("plant_types", &plant_type_vec);
    render_template(state, "plants.html", &mut ctx)
}
*/

pub fn plant_type_post(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let form = from_body!(state, "plants", PlantTypeForm);
    let plant_type = NewPlantType {
        name: form.name,
        slug: form.slug,
        user_id: get_user_id!(state)
    };
    let _plant_type = insert!(state, plant_types, plant_type, PlantType);

    redirect(state, url_for!("plants"))
}

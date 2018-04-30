use gotham::state::State;
use hyper::Response;

use models::NewPlantType;
use forms::PlantTypeForm;
use lib::http::redirect;
use lib::db::connection;

use diesel::prelude::*;
use diesel::insert_into;
use schema::plant_types;

pub fn plant_type_post(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let form = from_body!(state, "plants", PlantTypeForm);
    let plant_type = NewPlantType {
        name: form.name,
        slug: form.slug,
        user_id: get_user_id!(state)
    };

    try_db!(state, insert_into(plant_types::table)
        .values(&plant_type)
        .execute(&*connection()));

    redirect(state, url_for!("plants"))
}

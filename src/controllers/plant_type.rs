use gotham::state::State;
use hyper::Response;
use tera::Context;

use models::{NewPlantType, PlantTypeView};
use forms::PlantTypeForm;
use lib::http::{redirect, render_template};
use lib::db::connection;
use lib::utils::random_string;

use diesel::prelude::*;
use diesel::insert_into;
use schema::{plant_types, users};
use router::SlugPath;

use slugify::slugify;

pub fn create_plant_type(mut state: State) -> (State, Response) {
    assert_auth!(state);
    render_template(state, "create_plant_type.html", &mut Context::new())
}

pub fn plant_type(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let slug = from_path!(state, SlugPath).slug;
    let user_id = get_user_id!(state);

    let plant_type = try_db!(
        state,
        plant_types::table
            .inner_join(users::table)
            .filter(plant_types::slug.eq(slug).and(users::id.eq(user_id)))
            .select(PlantTypeViewSql!())
            .first::<PlantTypeView>(&*connection())
    );

    let mut ctx = Context::new();
    ctx.add("plant_type", &plant_type);
    render_template(state, "plant_type.html", &mut ctx)
}

pub fn plant_type_post(mut state: State) -> (State, Response) {
    assert_auth!(state);

    let form = from_body!(state, url_for!("plants"), PlantTypeForm);
    let filename = random_string(20);
    let plant_type = NewPlantType {
        slug: slugify!(&form.name),
        name: form.name,
        filename: filename.clone(),
        user_id: get_user_id!(state),
    };

    save_file!(state, "filename", &filename);

    try_db!(
        state,
        insert_into(plant_types::table)
            .values(&plant_type)
            .execute(&*connection())
    );

    redirect(state, url_for!("plants"))
}

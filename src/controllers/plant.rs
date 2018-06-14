use actix_web::{HttpRequest, HttpResponse, Json, Path};
use diesel::insert_into;
use diesel::prelude::*;

use lib::{auth::user_id, error::Error, schema::events, schema::plant_types, schema::plants,
          schema::users};
use models::{NewPlant, Plant, PlantForm, PlantView};
use State;

pub fn plant((id, req): (Path<i32>, HttpRequest<State>)) -> Result<HttpResponse, Error> {
    let user_id = user_id(&req)?;
    let plant_id = id.into_inner();
    trace!(
        req.state().log,
        "Plant (user_id: {}): {}",
        user_id,
        plant_id
    );

    let plant = plants::table
        .inner_join(users::table)
        .inner_join(plant_types::table)
        .left_join(events::table.on(plants::last_event_id.eq(events::id.nullable())))
        .filter(plants::id.eq(plant_id).and(plants::user_id.eq(user_id)))
        .select(PlantViewSql!())
        .first::<PlantView>(conn!(req.state().pool))?;
    debug!(req.state().log, "Plant: {:?}", plant);
    Ok(HttpResponse::Ok().json(plant))
}

pub fn plant_index(req: HttpRequest<State>) -> Result<HttpResponse, Error> {
    let user_id = user_id(&req)?;
    trace!(req.state().log, "Plants (user_id: {})", user_id);

    let plants = plants::table
        .inner_join(users::table)
        .inner_join(plant_types::table)
        .left_join(events::table.on(plants::last_event_id.eq(events::id.nullable())))
        .filter(plants::user_id.eq(user_id))
        .select(PlantViewSql!())
        .load::<PlantView>(conn!(req.state().pool))?;
    debug!(req.state().log, "Plants: {:?}", plants);
    Ok(HttpResponse::Ok().json(plants))
}

pub fn plant_post(
    (form, req): (Json<PlantForm>, HttpRequest<State>),
) -> Result<HttpResponse, Error> {
    let form = form.into_inner();
    let user_id = user_id(&req)?;
    trace!(
        req.state().log,
        "Create Plant (user_id: {}): {:?}",
        user_id,
        form
    );

    let plant = NewPlant {
        name: filled!(form.name),
        type_id: form.type_id,
        user_id: user_id,
    };

    let plant = insert_into(plants::table)
        .values(&plant)
        .get_result::<Plant>(conn!(req.state().pool))?;
    debug!(req.state().log, "Plants: {:?}", plant);
    Ok(HttpResponse::Ok().json(plant))
}

#[cfg(test)]
mod tests {
    use actix_web::{http::Method, http::StatusCode, test::TestServer, HttpMessage};
    use build_app;
    use futures::future::Future;
    use lib::{db::test_pool, utils::authenticate_tester, utils::create_plant_type};
    use models::{PlantForm, PlantView};

    fn show(srv: &mut TestServer, cookie: &str, id: i32, expected: StatusCode) {
        let mut req = srv.client(Method::GET, &format!("/plant/{}", id));
        opt_cookie!(req, cookie);

        let r = srv.execute(req.finish().unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);

        if expected == StatusCode::OK {
            assert!(header!(r, "content-length") != "0");
            assert_eq!(header!(r, "content-type"), "application/json");
        } else {
            assert_eq!(header!(r, "content-length"), "0");
        }
    }

    fn index(srv: &mut TestServer, cookie: &str, count: usize, expected: StatusCode) {
        let mut req = srv.client(Method::GET, "/plants");
        opt_cookie!(req, cookie);

        let r = srv.execute(req.finish().unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);

        if count == 0 {
            let size = if expected == StatusCode::OK { "2" } else { "0" };
            assert_eq!(header!(r, "content-length"), size);
        } else {
            assert_eq!(r.json::<Vec<PlantView>>().wait().unwrap().len(), count);
        }
    }

    fn create(srv: &mut TestServer, cookie: &str, name: &str, type_id: i32, expected: StatusCode) {
        let body = PlantForm {
            name: name.to_owned(),
            type_id: type_id,
        };
        let mut req = srv.client(Method::POST, "/plant");
        opt_cookie!(req, cookie);

        let r = srv.execute(req.json(body).unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);

        if expected == StatusCode::OK {
            assert!(header!(r, "content-length") != "0");
            assert_eq!(header!(r, "content-type"), "application/json");
        } else {
            assert_eq!(header!(r, "content-length"), "0");
        }
    }

    #[test]
    fn plant() {
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        let cookie = authenticate_tester(&mut srv);

        index(&mut srv, "", 0, StatusCode::UNAUTHORIZED);
        index(&mut srv, &cookie, 0, StatusCode::OK);

        create(&mut srv, "", "plant", 0, StatusCode::UNAUTHORIZED);
        create(&mut srv, &cookie, "plant", 0, StatusCode::BAD_REQUEST);

        // Conflict rolls-back test transaction, so we have to start pool again (and authenticate)
        // TODO: this should be fixed
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        let cookie = authenticate_tester(&mut srv);
        let id = create_plant_type(&mut srv, &cookie);

        show(&mut srv, "", 1, StatusCode::UNAUTHORIZED);
        show(&mut srv, &cookie, 1, StatusCode::NOT_FOUND);
        index(&mut srv, &cookie, 0, StatusCode::OK);

        create(&mut srv, &cookie, "plant", id, StatusCode::OK);
        index(&mut srv, &cookie, 1, StatusCode::OK);

        create(&mut srv, &cookie, "plant", id, StatusCode::OK);
        create(&mut srv, &cookie, "", id, StatusCode::BAD_REQUEST);
        create(&mut srv, &cookie, "", 0, StatusCode::BAD_REQUEST);
        index(&mut srv, &cookie, 2, StatusCode::OK);
        create(&mut srv, &cookie, "plant", 0, StatusCode::BAD_REQUEST);
    }
}

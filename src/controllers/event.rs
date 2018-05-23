use actix_web::{HttpRequest, HttpResponse, Json, Path};
use diesel::insert_into;
use diesel::prelude::*;

use State;
use lib::{auth::user_id, error::Error, schema::events, schema::plants};
use models::{Event, EventForm, EventView, NewEvent};

pub fn event_index((path, req): (Path<i32>, HttpRequest<State>)) -> Result<HttpResponse, Error> {
    let user_id = user_id(&req)?;
    let plant_id = path.into_inner();
    trace!(
        req.state().log,
        "Event Index (user_id: {}): {}",
        user_id,
        plant_id
    );

    // Assure non-existant plants are detected
    let _ = plants::table
        .find(plant_id)
        .select(plants::id)
        .first::<i32>(&*req.state().connection()?)?;

    let events = events::table
        .inner_join(plants::table.on(events::plant_id.eq(plants::id)))
        .filter(
            events::plant_id
                .eq(plant_id)
                .and(plants::user_id.eq(user_id)),
        )
        .select(EventViewSql!())
        .load::<EventView>(&*req.state().connection()?)?;
    info!(req.state().log, "Events: {:?}", events);
    Ok(HttpResponse::Ok().json(events))
}

pub fn event_post(
    (form, req): (Json<EventForm>, HttpRequest<State>),
) -> Result<HttpResponse, Error> {
    let form = form.into_inner();
    trace!(req.state().log, "Create Event: {:?}", form);
    let event = NewEvent {
        plant_id: form.pid,
        air_temperature_celsius: form.at,
        air_humidity_percentage: form.ah,
        soil_temperature_celsius: form.st,
        soil_resistivity: form.sr,
        light: form.l,
        device_timestamp: form.t,
    };

    let event = insert_into(events::table)
        .values(&event)
        .get_result::<Event>(&*req.state().connection()?)?;
    info!(req.state().log, "Created event: {:?}", event);
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::{HttpMessage, http::Method, http::StatusCode, test::TestServer};
    use build_app;
    use futures::future::Future;
    use lib::{utils::authenticate_tester, utils::clean_db, utils::create_plant};
    use models::{EventForm, EventView};

    fn index(
        srv: &mut TestServer,
        cookie: &str,
        plant_id: i32,
        count: usize,
        expected: StatusCode,
    ) {
        let mut req = srv.client(Method::GET, &format!("/plant/{}/events", plant_id));
        opt_cookie!(req, cookie);

        let r = srv.execute(req.finish().unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);

        if count == 0 {
            let size = if expected == StatusCode::OK { "2" } else { "0" };
            assert_eq!(header!(r, "content-length"), size);
        } else {
            assert_eq!(r.json::<Vec<EventView>>().wait().unwrap().len(), count);
        }
    }

    fn create(srv: &mut TestServer, form: EventForm, expected: StatusCode) {
        let mut req = srv.client(Method::POST, "/event");
        let r = srv.execute(req.json(form).unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);
    }

    #[test]
    fn event() {
        clean_db();
        let mut srv = TestServer::with_factory(build_app);

        let cookie = authenticate_tester(&mut srv);
        let id = create_plant(&mut srv, &cookie);

        let mut form = EventForm {
            pid: id,
            at: 0,
            ah: 1,
            st: 0,
            sr: 0,
            l: 1,
            t: 1,
        };

        index(&mut srv, "", 0, 0, StatusCode::UNAUTHORIZED);
        index(&mut srv, &cookie, 0, 0, StatusCode::NOT_FOUND);
        index(&mut srv, &cookie, id, 0, StatusCode::OK);

        create(&mut srv, form.clone(), StatusCode::OK);
        index(&mut srv, &cookie, id, 1, StatusCode::OK);
        form.t += 1;

        create(&mut srv, form.clone(), StatusCode::OK);
        index(&mut srv, &cookie, id, 2, StatusCode::OK);
        form.t += 1;

        create(&mut srv, form.clone(), StatusCode::OK);
        index(&mut srv, &cookie, id, 3, StatusCode::OK);
    }
}

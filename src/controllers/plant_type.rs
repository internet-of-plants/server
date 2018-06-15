use actix_web::{HttpRequest, HttpResponse, Json, Path};
use diesel::insert_into;
use diesel::prelude::*;
use slugify::slugify;

use config::FILENAME_SIZE;
#[cfg(not(test))]
use diesel::delete;
#[cfg(not(test))]
use lib::utils::save_image;
use lib::{auth::user_id, error::Error, schema::plant_types, schema::users,
          utils::decode_b64_image, utils::random_string, utils::State};
use models::{NewPlantType, PlantType, PlantTypeForm, PlantTypeView};

pub fn plant_type((path, req): (Path<String>, HttpRequest<State>)) -> Result<HttpResponse, Error> {
    let slug = path.into_inner();
    trace!(
        req.state().log,
        "Plant Type (user_id: {:?}): {}",
        user_id(&req),
        slug.clone()
    );

    let plant_type = plant_types::table
        .inner_join(users::table.on(plant_types::user_id.eq(users::id)))
        .filter(plant_types::slug.eq(slug))
        .select(PlantTypeViewSql!())
        .first::<PlantTypeView>(conn!(req.state().pool))?;
    debug!(req.state().log, "Plant Type: {:?}", plant_type);

    Ok(HttpResponse::Ok().json(plant_type))
}

pub fn plant_type_post(
    (form, req): (Json<PlantTypeForm>, HttpRequest<State>),
) -> Result<HttpResponse, Error> {
    let form = form.into_inner();
    trace!(
        req.state().log,
        "Plant Type (user_id: {:?}): {:?}",
        user_id(&req),
        form
    );
    let user_id = user_id(&req)?;

    let filename = random_string(FILENAME_SIZE)?;
    let plant_type = NewPlantType {
        slug: filled!(slugify!(&form.name)),
        name: filled!(form.name),
        filename: filename,
        user_id: user_id,
    };

    let _image = decode_b64_image(&filled!(form.image))?;

    let plant_type = insert_into(plant_types::table)
        .values(&plant_type)
        .get_result::<PlantType>(conn!(req.state().pool))?;
    debug!(req.state().log, "Plant Type: {:?}", plant_type);

    #[cfg(not(test))]
    {
        if let Err(err) = save_image(&plant_type.filename, &_image) {
            // Image-less plant_type will exist if ? returns some error
            delete(plant_types::table.find(plant_type.id)).execute(conn!(req.state().pool))?;
            return Err(Error::Image(err));
        }
        debug!(
            req.state().log,
            "Saved Image (and thumbnail): {}.jpg",
            plant_type.filename
        );
    }

    Ok(HttpResponse::Ok().json(plant_type))
}

pub fn plant_type_index(req: HttpRequest<State>) -> Result<HttpResponse, Error> {
    trace!(req.state().log, "PlantTypes");

    let plant_types = plant_types::table
        .inner_join(users::table)
        .select(PlantTypeViewSql!())
        .load::<PlantTypeView>(conn!(req.state().pool))?;
    debug!(req.state().log, "PlantTypes: {:?}", plant_types);
    Ok(HttpResponse::Ok().json(plant_types))
}

#[cfg(test)]
mod tests {
    use actix_web::{http::Method, http::StatusCode, test::TestServer, HttpMessage};
    use base64::encode;
    use futures::future::Future;
    use lib::{db::test_pool, utils::authenticate_tester, utils::build_app};
    use models::{PlantTypeForm, PlantTypeView};

    fn create(srv: &mut TestServer, cookie: &str, name: &str, image: &str, expected: StatusCode) {
        let body = PlantTypeForm {
            name: name.to_owned(),
            image: image.to_owned(),
        };

        let mut req = srv.client(Method::POST, "/plant_type");
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

    fn index(srv: &mut TestServer, cookie: &str, count: usize, expected: StatusCode) {
        let mut req = srv.client(Method::GET, "/plant_types");
        opt_cookie!(req, cookie);

        let r = srv.execute(req.finish().unwrap().send()).unwrap();
        assert_eq!(r.status(), expected);

        if count == 0 {
            let size = if expected == StatusCode::OK { "2" } else { "0" };
            assert_eq!(header!(r, "content-length"), size);
        } else {
            assert_eq!(r.json::<Vec<PlantTypeView>>().wait().unwrap().len(), count);
        }
    }

    fn show(srv: &mut TestServer, cookie: &str, slug: &str, expected: StatusCode) {
        let mut req = srv.client(Method::GET, &format!("/plant_type/{}", slug));
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

    #[test]
    fn plant_type() {
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        let image = ::config::TEST_IMAGE;
        let cookie = authenticate_tester(&mut srv);

        index(&mut srv, "", 0, StatusCode::OK);
        index(&mut srv, &cookie, 0, StatusCode::OK);

        create(&mut srv, "", "planttype", image, StatusCode::UNAUTHORIZED);

        show(&mut srv, &cookie, "planttype", StatusCode::NOT_FOUND);
        show(&mut srv, "", "planttype", StatusCode::NOT_FOUND);
        index(&mut srv, &cookie, 0, StatusCode::OK);
        index(&mut srv, "", 0, StatusCode::OK);

        create(&mut srv, &cookie, "planttype", "", StatusCode::BAD_REQUEST);
        create(&mut srv, &cookie, "planttype", "a", StatusCode::BAD_REQUEST);
        create(&mut srv, &cookie, "planttype", "^", StatusCode::BAD_REQUEST);
        create(&mut srv, &cookie, "planttype", image, StatusCode::OK);

        show(&mut srv, &cookie, "planttype", StatusCode::OK);
        show(&mut srv, "", "planttype", StatusCode::OK);
        index(&mut srv, &cookie, 1, StatusCode::OK);
        index(&mut srv, "", 1, StatusCode::OK);

        create(&mut srv, &cookie, "planttype", image, StatusCode::CONFLICT);

        // Conflict rolls-back test transaction, so we have to start pool again (and authenticate)
        // TODO: this should be fixed
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        let cookie = authenticate_tester(&mut srv);
        create(&mut srv, &cookie, "planttype", image, StatusCode::OK);
        create(&mut srv, &cookie, "planttype2", image, StatusCode::OK);
        show(&mut srv, &cookie, "planttype2", StatusCode::OK);
        index(&mut srv, &cookie, 2, StatusCode::OK);

        create(&mut srv, &cookie, "", image, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn plant_type_payload_size() {
        let mut srv = TestServer::with_factory(build_app(&[0; 32], test_pool()));
        let cookie = authenticate_tester(&mut srv);

        let body = PlantTypeForm {
            name: "Test".to_owned(),
            image: format!("data:image/jpeg;base64,{}", encode(&"A".repeat(1000))),
        };
        let mut req = srv.client(Method::POST, "/plant_type");
        opt_cookie!(req, cookie);

        let r = srv.execute(req.json(body).unwrap().send()).unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }

}

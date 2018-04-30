use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{Response, StatusCode};
use hyper::header::Location;
use tera::{Tera, Context};
use mime;
use lib::template_filter::url_for_filter;
use lib::auth::csrf_token;

lazy_static! {
    pub static ref TERA: Tera = {
        let mut t = compile_templates!("src/templates/*");
        t.register_filter("url_for", url_for_filter);
        t
    };
}

pub fn redirect(state: State, uri: String) -> (State, Response) {
    let mut response = create_response(&state,
                                       StatusCode::SeeOther,
                                       None);
    response.headers_mut().set(Location::new(uri));
    (state, response)
}

pub fn internal_server_error(state: State) -> (State, Response) {
    let response = create_response(&state,
                                   StatusCode::InternalServerError,
                                   None);
    (state, response)
}

pub fn not_found(state: State) -> (State, Response) {
    let response = create_response(&state,
                                   StatusCode::NotFound,
                                   None);
    (state, response)
}

pub fn bad_request(state: State) -> (State, Response) {
    let response = create_response(&state,
                                   StatusCode::BadRequest,
                                   None);
    (state, response)
}

pub fn render_template(state: State, template: &str,
                       ctx: &mut Context) -> (State, Response) {
    // `is_auth` sets csrf token in GET requests if non existent
    ctx.add("csrf_token", &csrf_token(&state).unwrap());

    match TERA.render(template, ctx) {
        Ok(content) => {
            let response = create_response(
                &state,
                StatusCode::Ok,
                Some((content.into_bytes(), mime::TEXT_HTML))
            );
            (state, response)
        },
        Err(err) => {
            error!("Error compiling template {}", template);
            error!("{:?}", err);
            let response = create_response(
                &state,
                StatusCode::InternalServerError,
                Some((vec![], mime::TEXT_HTML))
            );
            (state, response)
        }
    }
}

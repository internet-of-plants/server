use gotham::state::{FromState, State};
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use futures::stream::Stream;
use futures::Future;
use lib::auth::set_csrf_token;
use hyper::{Body, Headers, Method, Post};
use hyper::header::ContentType;
use mime;
use std::str;

#[derive(Debug)]
pub enum Encoding {
    UrlEncoded(String),
    Multipart(Vec<u8>),
}
pub use self::Encoding::*;

#[derive(StateData, Debug)]
pub struct BodyData(pub Encoding);

#[derive(Clone, NewMiddleware)]
pub struct BodyMiddleware;

impl Middleware for BodyMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: 'static + FnOnce(State) -> Box<HandlerFuture>,
    {
        set_csrf_token(&mut state);

        match Method::borrow_from(&mut state) {
            &Post => {}
            _ => return chain(state),
        }

        Box::new(Body::take_from(&mut state).concat2().then(|raw| {
            let urlencoded = mime::APPLICATION_WWW_FORM_URLENCODED.as_ref();
            let form_data = mime::MULTIPART_FORM_DATA.as_ref();
            match raw {
                Ok(chunk) => {
                    let body_data = if let Some(content_type) =
                        Headers::borrow_from(&state).get::<ContentType>()
                    {
                        let content_type: &str = content_type.as_ref();
                        if content_type == urlencoded {
                            match str::from_utf8(&*chunk) {
                                Ok(string) => if string.len() > 0 {
                                    Some(BodyData(UrlEncoded(string.to_owned())))
                                } else {
                                    None
                                },
                                Err(err) => {
                                    error!("Body middleware utf-8 error: {}", err);
                                    None
                                }
                            }
                        } else if &content_type[..form_data.len()] == form_data {
                            Some(BodyData(Multipart(chunk.to_vec())))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(data) = body_data {
                        state.put(data);
                    }
                }
                Err(err) => error!("Body middleware error: {}", err),
            }
            chain(state)
        }))
    }
}

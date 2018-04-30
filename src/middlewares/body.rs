use gotham::state::{FromState, State};
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use futures::stream::Stream;
use futures::Future;
use lib::auth::set_csrf_token;
use hyper::{Body, Method, Post};
use std::str;

#[derive(StateData, Debug)]
pub struct BodyData(pub String);

#[derive(Clone, NewMiddleware)]
pub struct BodyMiddleware;

impl Middleware for BodyMiddleware {
    fn call<Chain>(self, mut state: State,
                   chain: Chain) -> Box<HandlerFuture>
      where Chain: 'static + FnOnce(State) -> Box<HandlerFuture> {
        set_csrf_token(&mut state);

        match Method::borrow_from(&mut state) {
            &Post => {},
            _ => return chain(state)
        }

        Box::new(Body::take_from(&mut state)
            .concat2()
            .then(|raw| {
                match raw {
                    Ok(chunk) => match str::from_utf8(&*chunk) {
                        Ok(bytes) => if bytes.len() > 0 {
                            state.put(BodyData(bytes.to_owned()))
                        },
                        Err(err) => error!("Body middleware utf-8 error: {}", err)
                    },
                    Err(err) => error!("Body middleware error: {}", err)
                }
                chain(state)
            })
        )
    }
}

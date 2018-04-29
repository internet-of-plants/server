use gotham::state::State;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use lib::db::{connection, DbConnection};

#[derive(StateData)]
pub struct Connection(pub DbConnection);

#[derive(Clone, NewMiddleware)]
pub struct DbMiddleware;

impl Middleware for DbMiddleware {
    fn call<Chain>(self, mut state: State,
                   chain: Chain) -> Box<HandlerFuture>
      where Chain: 'static + FnOnce(State) -> Box<HandlerFuture> {
        state.put(Connection(connection()));
        chain(state)
    }
}

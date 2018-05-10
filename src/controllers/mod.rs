pub mod user;
pub mod plant;
pub mod plant_type;
pub mod event;

pub use self::user::*;
pub use self::plant::*;
pub use self::plant_type::*;
pub use self::event::*;

use hyper::{Response, StatusCode};
use gotham::state::{FromState, State};
use gotham::http::response::create_response;

use mime;
use lib::http::not_found;
use router::PathExtractor;
use std::fs::File;
use std::io::Read;

pub fn static_file(state: State) -> (State, Response) {
    let path = {
        let path_vec = &PathExtractor::borrow_from(&state).parts;
        let mut path = "./src/static".to_owned();
        for part in path_vec {
            if part == ".." {
                continue;
            }
            path.push('/');
            path.push_str(&part);
        }
        path
    };

    if path.len() <= 3 {
        return not_found(state);
    }

    let mime = match &path[path.len() - 3..] {
        "css" => mime::TEXT_CSS,
        ".js" => mime::TEXT_JAVASCRIPT,
        "jpg" => mime::IMAGE_JPEG,
        "png" => mime::IMAGE_PNG,
        "gif" => mime::IMAGE_GIF,
        _ => mime::TEXT_PLAIN,
    };

    match File::open(path) {
        Ok(mut file) => {
            let mut buffer: Vec<u8> = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let response = create_response(&state, StatusCode::Ok, Some((buffer, mime)));
            return (state, response);
        }
        Err(_) => {}
    }
    not_found(state)
}

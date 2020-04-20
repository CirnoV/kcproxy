#[macro_use]
extern crate lazy_static;

mod auth;
mod filters;
mod handlers;

use std::env;

#[tokio::main]
async fn main() {
    if env::var_os("SECRET_KEY").is_none() {
        panic!("env SECRET_KEY must not be empty.");
    }
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "kcproxy=info");
    }
    pretty_env_logger::init();

    let routes = filters::kcproxy();
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
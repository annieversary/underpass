use std::fs::read_to_string;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use muxa::errors::*;

mod osm_to_geojson;
mod search;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(home))
        .route("/style.css", get(css))
        .route("/index.js", get(js))
        .route("/search", post(search::search));

    #[cfg(debug_assertions)]
    println!("listening on http://localhost:3000");

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Result<Html<String>, ErrResponse> {
    read_to_string("./src/index.html")
        .map_err(internal_error)
        .map(Html)

    // Html(include_str!("index.html"))
}

async fn css() -> Result<([(&'static str, &'static str); 1], String), ErrResponse> {
    read_to_string("./src/style.css")
        .map_err(internal_error)
        .map(|a| ([("content-type", "text/css")], a))
}

async fn js() -> Result<([(&'static str, &'static str); 1], String), ErrResponse> {
    read_to_string("./src/index.js")
        .map_err(internal_error)
        .map(|a| ([("content-type", "text/javascript")], a))
}

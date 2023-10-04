#[cfg(debug_assertions)]
use std::fs::read_to_string;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};

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

async fn home() -> Html<String> {
    // read file when on debug, embed file when on release
    // this way we can live edit in local, and dont have to keep the files next to the executable in prod

    #[cfg(debug_assertions)]
    return Html(read_to_string("./src/index.html").unwrap());

    #[cfg(not(debug_assertions))]
    Html(include_str!("index.html").to_string())
}

async fn css() -> ([(&'static str, &'static str); 1], String) {
    #[cfg(debug_assertions)]
    let a = read_to_string("./src/style.css").unwrap();
    #[cfg(not(debug_assertions))]
    let a = include_str!("style.css").to_string();

    ([("content-type", "text/css")], a)
}

async fn js() -> ([(&'static str, &'static str); 1], String) {
    #[cfg(debug_assertions)]
    let a = read_to_string("./src/index.js").unwrap();
    #[cfg(not(debug_assertions))]
    let a = include_str!("index.js").to_string();

    ([("content-type", "text/css")], a)
}

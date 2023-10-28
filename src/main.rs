#[cfg(debug_assertions)]
use std::fs::read_to_string;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};

mod nominatim;
mod osm_to_geojson;
mod preprocess;
mod road_angle;
mod search;

#[tokio::main]
async fn main() {
    // we only care if the error is a line parse
    if let Err(err @ dotenv::Error::LineParse(..)) = dotenv::dotenv() {
        panic!("{:?}", err);
    }

    // build our application with a single route
    let app = Router::new()
        .route("/", get(home))
        .route("/index.css", get(css))
        .route("/index.js", get(js))
        .route("/search", post(search::search));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or(3000);

    #[cfg(debug_assertions)]
    println!("listening on http://localhost:{port}");

    // run it with hyper on localhost:3000
    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Html<String> {
    // read file when on debug, embed file when on release
    // this way we can live edit in local, and dont have to keep the files next to the executable in prod

    #[cfg(debug_assertions)]
    return Html(read_to_string("./frontend/index.html").unwrap());

    #[cfg(not(debug_assertions))]
    Html(include_str!("../public/index.html").to_string())
}

async fn css() -> ([(&'static str, &'static str); 1], String) {
    #[cfg(debug_assertions)]
    let a = read_to_string("./public/index.css").unwrap();
    #[cfg(not(debug_assertions))]
    let a = include_str!("../public/index.css").to_string();

    ([("content-type", "text/css")], a)
}

async fn js() -> ([(&'static str, &'static str); 1], String) {
    #[cfg(debug_assertions)]
    let a = read_to_string("./public/index.js").unwrap();
    #[cfg(not(debug_assertions))]
    let a = include_str!("../public/index.js").to_string();

    ([("content-type", "text/js")], a)
}

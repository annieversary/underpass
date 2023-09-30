use std::fs::read_to_string;

use axum::{
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use muxa::errors::*;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(home))
        .route("/search", post(search));

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

#[derive(Deserialize)]
struct Bbox {
    ne: [f32; 2],
    sw: [f32; 2],
}

#[derive(Deserialize)]
struct SearchParams {
    query: String,
    bbox: Bbox,
}
#[derive(Serialize)]
struct SearchResults {
    data: [f32; 2],
}

async fn search(Json(json): Json<SearchParams>) -> Json<SearchResults> {
    let query = preprocess_query(json.query, &json.bbox);

    // TODO send query to overpass, get data
    // https://gitlab.com/trailstash/overpass-ultra/-/blob/main/overpass.js?ref_type=heads

    // TODO osmtogeojson???
    // https://lib.rs/crates/osm-to-geojson

    // TODO further process the data
    // we will probably need to construct a set of filters that will then process this data

    Json(SearchResults {
        data: [
            (json.bbox.ne[0] + json.bbox.sw[0]) / 2.0,
            (json.bbox.ne[1] + json.bbox.sw[1]) / 2.0,
        ],
    })
}

fn preprocess_query(query: String, bbox: &Bbox) -> String {
    // TODO process query, do rewriting

    // TODO replace {{bbox}}
    // TODO geocode area https://github.com/tyrasd/overpass-turbo/blob/eb216aa08b06590a4efc4e10d6a25140d53fcf70/js/shortcuts.ts#L92

    query
}

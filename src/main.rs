use std::fs::read_to_string;

use axum::{
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use muxa::errors::*;
use osm_to_geojson::Osm;
use serde::{Deserialize, Serialize};

use crate::osm_to_geojson::osm_to_geojson;

mod osm_to_geojson;

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
    // we probably want like a list of Filter nodes or smth
}
#[derive(Serialize)]
struct SearchResults {
    data: geojson::GeoJson,
}

// TODO add error handling
async fn search(Json(json): Json<SearchParams>) -> Json<SearchResults> {
    let query = preprocess_query(json.query, &json.bbox);

    let client = reqwest::Client::new();
    let res: Osm = client
        .post("https://overpass-api.de/api/interpreter")
        .body(dbg!(query))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let geojson = osm_to_geojson(res);

    // TODO further process the data
    // we will probably need to construct a set of filters that will then process this data

    Json(SearchResults { data: geojson })
}

fn preprocess_query(query: String, bbox: &Bbox) -> String {
    // TODO process query, do rewriting

    // TODO geocode area https://github.com/tyrasd/overpass-turbo/blob/eb216aa08b06590a4efc4e10d6a25140d53fcf70/js/shortcuts.ts#L92

    query.replace(
        "{{bbox}}",
        &format!(
            "{},{},{},{}",
            bbox.sw[0], bbox.sw[1], bbox.ne[0], bbox.ne[1]
        ),
    )
}

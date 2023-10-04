use std::fs::read_to_string;

use axum::{
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use muxa::errors::*;
use osm_to_geojson::Osm;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::osm_to_geojson::osm_to_geojson;

mod osm_to_geojson;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(home))
        .route("/style.css", get(css))
        .route("/index.js", get(js))
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
    query: String,
    geocode_areas: Vec<GeocodeaArea>,
}

#[derive(Serialize)]
pub struct GeocodeaArea {
    id: u64,
    ty: String,
    name: String,
}

#[derive(Error, Debug)]
enum SearchError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("json parse error")]
    JsonParse(reqwest::Error),
    #[error("{0}")]
    Syntax(String),
    #[error("Nominatim: {0}")]
    Nominatim(String),
}

impl IntoResponse for SearchError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("{self}"),
                "format": if matches!(self, Self::Syntax(..)) { "xml" } else { "text" },
            })),
        )
            .into_response()
    }
}

async fn search(Json(json): Json<SearchParams>) -> Result<Json<SearchResults>, SearchError> {
    let (query, geocode_areas) = preprocess_query(json.query, &json.bbox).await?;

    dbg!(&query);

    let client = reqwest::Client::new();
    let res = client
        .post("https://overpass-api.de/api/interpreter")
        .body(query.clone())
        .send()
        .await?;

    if res.status() == 200 {
        let res: Osm = res.json().await.map_err(SearchError::JsonParse)?;

        let geojson = osm_to_geojson(res);

        Ok(Json(SearchResults {
            data: geojson,
            query,
            geocode_areas,
        }))
    } else {
        let res = res.text().await?;
        Err(SearchError::Syntax(res))
    }
}

async fn preprocess_query(
    query: String,
    bbox: &Bbox,
) -> Result<(String, Vec<GeocodeaArea>), SearchError> {
    let mut geocode_areas = vec![];

    let re = Regex::new(r"\{\{\s*(\w+):?([\S\s]+)?\}\}").unwrap();

    let mut new = String::with_capacity(query.len());
    let mut last_match = 0;
    for caps in re.captures_iter(&query) {
        let m = caps.get(0).unwrap();
        new.push_str(&query[last_match..m.start()]);

        let replacement = match &caps[1] {
            "bbox" => format!(
                "{},{},{},{}",
                bbox.sw[0], bbox.sw[1], bbox.ne[0], bbox.ne[1]
            ),
            "geocodeArea" => {
                let (id, area) = nominatim_search(caps[2].trim()).await?;
                geocode_areas.push(id);
                area
            }
            _ => caps[0].to_string(),
        };

        new.push_str(&replacement);
        last_match = m.end();
    }
    new.push_str(&query[last_match..]);

    Ok((new, geocode_areas))
}

/// returns ($id,area(id:$id))
async fn nominatim_search(search: &str) -> Result<(GeocodeaArea, String), SearchError> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://nominatim.openstreetmap.org/search?format=jsonv2&q={search}"
        ))
        .header("User-Agent", "Underpass, annie@bursary.town")
        .send()
        .await?;

    let res: serde_json::Value = res.json().await?;
    let arr = res
        .as_array()
        .ok_or_else(|| SearchError::Nominatim("response was not an array".to_string()))?;
    if let Some(serde_json::Value::Object(obj)) = arr.get(0) {
        let orig_id = obj
            .get("osm_id")
            .ok_or_else(|| {
                SearchError::Nominatim("nominatim response did not contain osm_id".to_string())
            })?
            .as_number()
            .ok_or_else(|| SearchError::Nominatim("osm_id was not a number".to_string()))?
            .as_u64()
            .ok_or_else(|| SearchError::Nominatim("osm_id was not a u64".to_string()))?;
        let ty = obj
            .get("osm_type")
            .ok_or_else(|| {
                SearchError::Nominatim("nominatim response did not contain osm_type".to_string())
            })?
            .as_str()
            .ok_or_else(|| SearchError::Nominatim("osm_type was not a string".to_string()))?;
        let name = obj
            .get("display_name")
            .ok_or_else(|| {
                SearchError::Nominatim(
                    "nominatim response did not contain display_name".to_string(),
                )
            })?
            .as_str()
            .ok_or_else(|| SearchError::Nominatim("display_name was not a string".to_string()))?;

        // https://github.com/tyrasd/overpass-turbo/blob/eb216aa08b06590a4efc4e10d6a25140d53fcf70/js/shortcuts.ts#L92

        let mut id = orig_id;
        if ty == "relation" {
            id += 3600000000;
        }

        let id = if ty == "way" {
            format!("{},{id}", id + 2400000000)
        } else {
            format!("{id}")
        };

        Ok((
            GeocodeaArea {
                id: orig_id,
                ty: ty.to_string(),
                name: name.to_string(),
            },
            format!("area(id:{})", id),
        ))
    } else {
        Err(SearchError::Nominatim(format!(
            "no results found for {search}"
        )))
    }
}

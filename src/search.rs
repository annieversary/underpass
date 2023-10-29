use axum::response::{IntoResponse, Json};
use geojson::GeoJson;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    graph::{process_graph, Graph},
    nominatim::OsmNominatim,
    preprocess::preprocess_query,
};

pub async fn search(Json(json): Json<SearchParams>) -> Result<Json<SearchResults>, SearchError> {
    let (query, geocode_areas) = preprocess_query(json.query, &json.bbox, OsmNominatim).await?;

    let collection = process_graph(json.graph, query.clone()).await?;

    let geojson = GeoJson::FeatureCollection(collection);

    Ok(Json(SearchResults {
        data: geojson,
        query,
        geocode_areas,
    }))
}

#[derive(Deserialize, Default)]
pub struct Bbox {
    pub ne: [f32; 2],
    pub sw: [f32; 2],
}

#[derive(Deserialize)]
pub struct SearchParams {
    query: String,
    bbox: Bbox,
    graph: Graph,
}

#[derive(Serialize)]
pub struct SearchResults {
    pub data: geojson::GeoJson,
    pub query: String,
    pub geocode_areas: Vec<GeocodeaArea>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct GeocodeaArea {
    pub id: u64,
    pub ty: String,
    pub name: String,
    pub original: String,
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("json parse error")]
    JsonParse(reqwest::Error),
    #[error("{error}")]
    Syntax { error: String, query: String },
    #[error("Nominatim: {0}")]
    Nominatim(String),
    #[error("Road angle: {0}")]
    RoadAngle(String),
}

impl IntoResponse for SearchError {
    fn into_response(self) -> axum::response::Response {
        let mut json = json!({
            "error": format!("{self}"),
            "format": if matches!(self, Self::Syntax{ .. }) { "xml" } else { "text" },
        });
        if let Self::Syntax { query, .. } = self {
            json.as_object_mut()
                .unwrap()
                .insert("query".to_string(), query.into());
        }
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response()
    }
}

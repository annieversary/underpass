use std::collections::HashMap;

use axum::response::{IntoResponse, Json};
use geojson::GeoJson;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::graph::{process_graph, Graph};

pub async fn search(Json(json): Json<SearchParams>) -> Result<Json<SearchResults>, SearchError> {
    let result = process_graph(json.graph, json.bbox).await?;

    let geojson = GeoJson::FeatureCollection(result.collection);

    Ok(Json(SearchResults {
        data: geojson,
        processed_queries: result.processed_queries,
        geocode_areas: result.geocode_areas,
    }))
}

#[derive(Deserialize, Default, Clone, Copy)]
pub struct Bbox {
    pub ne: [f32; 2],
    pub sw: [f32; 2],
}

#[derive(Deserialize)]
pub struct SearchParams {
    bbox: Bbox,
    graph: Graph,
}

#[derive(Serialize)]
pub struct SearchResults {
    pub data: geojson::GeoJson,
    /// Node Id -> Processed query
    pub processed_queries: HashMap<String, String>,
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
    #[error("The provided graph contains a cycle")]
    CyclicalGraph,
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

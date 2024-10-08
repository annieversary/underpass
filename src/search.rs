use std::{collections::HashMap, hash::Hash, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use geojson::GeoJson;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    app_state::AppState,
    graph::{errors::GraphError, process::process_graph, Graph},
};

pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(json): Json<SearchParams>,
) -> Result<Json<SearchResults>, SearchError> {
    let result = process_graph(
        json.graph,
        json.bbox,
        &state.elevation_map,
        state.caches.clone(),
    )
    .await?;

    let geojson = GeoJson::FeatureCollection(result.collection);

    Ok(Json(SearchResults {
        data: geojson,
        processed_queries: result.processed_queries,
        geocode_areas: result.geocode_areas,
    }))
}

#[derive(Deserialize, Default, Clone, Copy, PartialEq)]
pub struct Bbox {
    pub ne: [f32; 2],
    pub sw: [f32; 2],
}

impl Hash for Bbox {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // shhh baby dont worry about it
        // these floats come from Deserialize, so they should be the same
        self.ne[0].to_bits().hash(state);
        self.ne[1].to_bits().hash(state);
        self.sw[0].to_bits().hash(state);
        self.sw[1].to_bits().hash(state);
    }
}

// ummmmmm
impl Eq for Bbox {}

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
    #[error("{0}")]
    Graph(#[from] GraphError),
}

impl SearchError {
    fn node_id(&self) -> Option<&String> {
        match self {
            Self::Graph(GraphError::OqlSyntax { node_id, .. }) => Some(node_id),
            Self::Graph(GraphError::InputMissing { node_id, .. }) => Some(node_id),
            _ => None,
        }
    }
}

impl IntoResponse for SearchError {
    fn into_response(self) -> axum::response::Response {
        let data = if let Self::Graph(GraphError::OqlSyntax { query, error, .. }) = &self {
            json!({
                "format": "xml",
                "query": query,
                "message": error,
            })
        } else {
            json!({
                "format": "text",
            })
        };

        let json = json!({
            "error": format!("{self}"),
            "data": data,
            "node_id": self.node_id(),
        });

        (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response()
    }
}

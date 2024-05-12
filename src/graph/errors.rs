use std::sync::Arc;

use thiserror::Error;

use crate::nominatim::NominatimError;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Connection refers to a non-existing node")]
    ConnectionNodeMissing,
    #[error("The provided graph contains a cycle")]
    Cycle,
    #[error("Graph is missing a Map node")]
    MapMissing,
    #[error("Node has no input")]
    InputMissing { node_id: String },
    #[error("Oql syntax error")]
    OqlSyntax {
        node_id: String,
        error: String,
        query: String,
    },
    #[error("Road angle: {message}")]
    RoadAngle { message: String, node_id: String },
    #[error("Road length: {message}")]
    RoadLength { message: String, node_id: String },
    #[error("Node has wrong input type {got}, expected {expected}")]
    WrongInputType { got: String, expected: String },
    #[error("Error parsing Overpass json")]
    OverpassJsonError,
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("nominatim error {0}")]
    Nominatim(#[from] NominatimError),
    #[error("{0}")]
    Arced(#[from] Arc<Self>),
}

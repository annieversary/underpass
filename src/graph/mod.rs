use std::collections::HashMap;

use geojson::FeatureCollection;
use serde::Deserialize;
use thiserror::Error;

use crate::search::GeocodeaArea;

mod nodes;
pub mod process;
mod utils;

pub struct GraphResult {
    pub collection: FeatureCollection,
    pub geocode_areas: Vec<GeocodeaArea>,
    pub processed_queries: HashMap<String, String>,
}

impl Default for GraphResult {
    fn default() -> Self {
        Self {
            collection: FeatureCollection {
                bbox: None,
                features: vec![],
                foreign_members: None,
            },
            geocode_areas: Default::default(),
            processed_queries: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Graph {
    nodes: Vec<GraphNode>,
    connections: Vec<GraphConnection>,
}
#[derive(Deserialize, Debug)]
pub struct GraphNode {
    id: String,
    #[serde(flatten)]
    node: GraphNodeInternal,
}

// we dont need the inputs/outputs in here since they dont contain any data
// we know what inputs and outputs each node has
#[derive(Deserialize, Debug)]
#[serde(tag = "label", content = "controls")]
pub enum GraphNodeInternal {
    Map {},
    #[serde(rename = "Overpass QL")]
    Oql {
        query: Control<String>,
        timeout: Control<u32>,
    },
    #[serde(rename = "Road Angle Filter")]
    RoadAngleFilter {
        min: Control<f64>,
        max: Control<f64>,
    },
    #[serde(rename = "Road Length Filter")]
    RoadLengthFilter {
        min: Control<f64>,
        max: Control<f64>,
        tolerance: Control<f64>,
    },
    InViewOf {},
}

#[derive(Deserialize, Debug)]
pub struct Control<T> {
    #[serde(rename = "id")]
    _id: String,
    value: T,
}

#[derive(Deserialize, Debug)]
pub struct GraphConnection {
    #[serde(rename = "id")]
    _id: String,
    source: String,
    #[serde(rename = "sourceOutput")]
    _source_output: String,
    target: String,
    #[serde(rename = "targetInput")]
    target_input: String,
}

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
    #[error("Node requires an input but it has none")]
    OqlSyntax {
        node_id: String,
        error: String,
        query: String,
    },
    #[error("Road angle: {message}")]
    RoadAngle { message: String, node_id: String },
    #[error("Road length: {message}")]
    RoadLength { message: String, node_id: String },
}

use serde::Deserialize;

use self::{
    errors::GraphError,
    nodes::{GraphNode, GraphNodeInternal},
};

pub mod errors;
mod nodes;
mod output;
pub mod process;
mod utils;

#[derive(Deserialize, Debug)]
pub struct Graph {
    nodes: Vec<GraphNode>,
    connections: Vec<GraphConnection>,
}

impl Graph {
    fn map_node(&self) -> Result<&GraphNode, GraphError> {
        self.nodes
            .iter()
            .find(|n| matches!(n.node, GraphNodeInternal::Map(_)))
            .ok_or(GraphError::MapMissing)
    }
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

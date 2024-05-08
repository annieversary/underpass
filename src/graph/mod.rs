use serde::Deserialize;

use self::process::NodeProcessor;

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

#[async_trait::async_trait]
pub trait Node {
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
    ) -> Result<output::NodeOutput, errors::GraphError>;

    fn id(&self) -> &str;
}

// we dont need the inputs/outputs in here since they dont contain any data
// we know what inputs and outputs each node has
#[derive(Deserialize, Debug)]
#[serde(tag = "label", content = "controls")]
pub enum GraphNode {
    Map(nodes::map::Map),

    // query nodes
    #[serde(rename = "OQL Code")]
    Oql(nodes::oql::Oql),
    #[serde(rename = "Oql Statement")]
    OqlStatement(nodes::oql_statement::OqlStatement),
    #[serde(rename = "Oql Union")]
    OqlUnion(nodes::oql_union::OqlUnion),
    #[serde(rename = "Oql Difference")]
    OqlDifference(nodes::oql_difference::OqlDifference),

    // geojson nodes
    Overpass(nodes::overpass::Overpass),
    #[serde(rename = "Road Angle Filter")]
    RoadAngleFilter(nodes::road_angle_filter::RoadAngleFilter),
    #[serde(rename = "Road Length Filter")]
    RoadLengthFilter(nodes::road_length_filter::RoadLengthFilter),
    #[serde(rename = "Elevation Filter")]
    ElevationFilter(nodes::elevation_filter::ElevationFilter),
    Union(nodes::union::Union),
    InViewOf(nodes::in_view_of::InViewOf),
}

// TODO use a macro to generate all of this
// the ambassador crate did not work with async_trait fsr
impl GraphNode {
    pub fn id(&self) -> &str {
        match self {
            GraphNode::Map(m) => m.id(),
            GraphNode::Oql(m) => m.id(),
            GraphNode::OqlStatement(m) => m.id(),
            GraphNode::OqlUnion(m) => m.id(),
            GraphNode::OqlDifference(m) => m.id(),
            GraphNode::Overpass(m) => m.id(),
            GraphNode::RoadAngleFilter(m) => m.id(),
            GraphNode::RoadLengthFilter(m) => m.id(),
            GraphNode::ElevationFilter(m) => m.id(),
            GraphNode::Union(m) => m.id(),
            GraphNode::InViewOf(m) => m.id(),
        }
    }

    pub async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
    ) -> Result<output::NodeOutput, errors::GraphError> {
        match self {
            GraphNode::Map(m) => m.process(processor).await,
            GraphNode::Oql(m) => m.process(processor).await,
            GraphNode::OqlStatement(m) => m.process(processor).await,
            GraphNode::OqlUnion(m) => m.process(processor).await,
            GraphNode::OqlDifference(m) => m.process(processor).await,
            GraphNode::Overpass(m) => m.process(processor).await,
            GraphNode::RoadAngleFilter(m) => m.process(processor).await,
            GraphNode::RoadLengthFilter(m) => m.process(processor).await,
            GraphNode::ElevationFilter(m) => m.process(processor).await,
            GraphNode::Union(m) => m.process(processor).await,
            GraphNode::InViewOf(m) => m.process(processor).await,
        }
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

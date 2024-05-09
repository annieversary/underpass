use super::{errors::GraphError, output::NodeOutput, process::NodeProcessor};

pub mod elevation_filter;
pub mod in_view_of;
pub mod map;
pub mod oql;
pub mod oql_difference;
pub mod oql_statement;
pub mod oql_union;
pub mod overpass;
pub mod road_angle_filter;
pub mod road_length_filter;
pub mod union;

#[async_trait::async_trait]
pub trait Node {
    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError>;

    fn id(&self) -> &str;
}

// we dont need the inputs/outputs in here since they dont contain any data
// we know what inputs and outputs each node has
#[derive(serde::Deserialize, Debug)]
#[serde(tag = "label", content = "controls")]
pub enum GraphNode {
    Map(map::Map),

    // query nodes
    #[serde(rename = "OQL Code")]
    Oql(oql::Oql),
    #[serde(rename = "Oql Statement")]
    OqlStatement(oql_statement::OqlStatement),
    #[serde(rename = "Oql Union")]
    OqlUnion(oql_union::OqlUnion),
    #[serde(rename = "Oql Difference")]
    OqlDifference(oql_difference::OqlDifference),

    // geojson nodes
    Overpass(overpass::Overpass),
    #[serde(rename = "Road Angle Filter")]
    RoadAngleFilter(road_angle_filter::RoadAngleFilter),
    #[serde(rename = "Road Length Filter")]
    RoadLengthFilter(road_length_filter::RoadLengthFilter),
    #[serde(rename = "Elevation Filter")]
    ElevationFilter(elevation_filter::ElevationFilter),
    Union(union::Union),
    InViewOf(in_view_of::InViewOf),
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
    ) -> Result<NodeOutput, GraphError> {
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

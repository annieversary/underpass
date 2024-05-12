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
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
        node_id: &str,
    ) -> Result<NodeOutput, GraphError>;
}

#[derive(serde::Deserialize, Debug)]
pub struct GraphNode {
    pub id: String,
    #[serde(flatten)]
    pub node: GraphNodeInternal,
}

// we dont need the inputs/outputs in here since they dont contain any data
// we know what inputs and outputs each node has
#[derive(serde::Deserialize, Debug)]
#[serde(tag = "label", content = "controls")]
pub enum GraphNodeInternal {
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
    pub async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
    ) -> Result<NodeOutput, GraphError> {
        match &self.node {
            GraphNodeInternal::Map(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::Oql(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::OqlStatement(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::OqlUnion(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::OqlDifference(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::Overpass(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::RoadAngleFilter(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::RoadLengthFilter(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::ElevationFilter(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::Union(m) => m.process(processor, &self.id).await,
            GraphNodeInternal::InViewOf(m) => m.process(processor, &self.id).await,
        }
    }
}

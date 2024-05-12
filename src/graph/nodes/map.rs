use serde::Deserialize;

use crate::graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor};

#[derive(Deserialize, Debug)]
pub struct Map {}

#[async_trait::async_trait]
impl Node for Map {
    async fn process(
        &self,
        _processor: &mut NodeProcessor<'_>,
        _node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        unreachable!()
    }
}

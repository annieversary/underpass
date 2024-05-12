use serde::Deserialize;

use crate::graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor};

#[derive(Deserialize, Debug)]
pub struct InViewOf {}

#[async_trait::async_trait]
impl Node for InViewOf {
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
        node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        let _input_collection = processor.get_input(node_id, "in").await?.into_features()?;
        let _aux_collection = processor.get_input(node_id, "aux").await?.into_features()?;

        // TODO filter input_collection by whether it can see the aux_collection

        todo!()
    }
}

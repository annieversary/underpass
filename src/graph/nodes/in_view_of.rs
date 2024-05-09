use serde::Deserialize;

use crate::graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor};

#[derive(Deserialize, Debug)]
pub struct InViewOf {
    id: String,
}

#[async_trait::async_trait]
impl Node for InViewOf {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let _input_collection = processor.get_input(self, "in").await?.into_features()?;
        let _aux_collection = processor.get_input(self, "aux").await?.into_features()?;

        // TODO filter input_collection by whether it can see the aux_collection

        todo!()
    }
}

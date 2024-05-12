use serde::Deserialize;

use crate::graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor};

#[derive(Deserialize, Debug)]
pub struct Union;

#[async_trait::async_trait]
impl Node for Union {
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
        node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        let mut a_collection = processor.get_input(node_id, "a").await?.into_features()?;
        let b_collection = processor.get_input(node_id, "b").await?.into_features()?;

        a_collection.features.extend(b_collection.features);
        Ok(a_collection.into())
    }
}

use serde::Deserialize;

use crate::graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor};

#[derive(Deserialize, Debug)]
pub struct OqlDifference {}

#[async_trait::async_trait]
impl Node for OqlDifference {
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
        node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        let a = processor.get_input(node_id, "a").await?.into_query()?;
        let b = processor.get_input(node_id, "b").await?.into_query()?;

        let query = format!("({a} - {b});");

        Ok(query.into())
    }
}

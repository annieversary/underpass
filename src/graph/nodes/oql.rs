use serde::Deserialize;

use crate::graph::{
    errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor, Control,
};

#[derive(Deserialize, Debug)]
pub struct Oql {
    query: Control<String>,
}

#[async_trait::async_trait]
impl Node for Oql {
    async fn process(
        &self,
        _processor: &mut NodeProcessor<'_>,
        _node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        Ok(self.query.value.clone().into())
    }
}

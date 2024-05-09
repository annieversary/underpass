use serde::Deserialize;

use crate::graph::{
    errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor, Control,
};

#[derive(Deserialize, Debug)]
pub struct Oql {
    id: String,
    query: Control<String>,
}

#[async_trait::async_trait]
impl Node for Oql {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, _processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        Ok(self.query.value.clone().into())
    }
}

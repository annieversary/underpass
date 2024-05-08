use serde::Deserialize;

use crate::graph::{errors::GraphError, output::NodeOutput, process::NodeProcessor, Node};

#[derive(Deserialize, Debug)]
pub struct Map {
    id: String,
}

#[async_trait::async_trait]
impl Node for Map {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, _processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        unreachable!()
    }
}

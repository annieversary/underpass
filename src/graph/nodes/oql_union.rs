use serde::Deserialize;

use crate::graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor};

#[derive(Deserialize, Debug)]
pub struct OqlUnion {
    id: String,
}

#[async_trait::async_trait]
impl Node for OqlUnion {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let a = processor.get_input(self, "a").await?.into_query()?;
        let b = processor.get_input(self, "b").await?.into_query()?;

        let query = format!("({a} {b});");

        Ok(query.into())
    }
}

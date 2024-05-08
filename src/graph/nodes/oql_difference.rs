use serde::Deserialize;

use crate::graph::{errors::GraphError, output::NodeOutput, process::NodeProcessor, Node};

#[derive(Deserialize, Debug)]
pub struct OqlDifference {
    id: String,
}

impl Node for OqlDifference {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let a = processor.get_input(self, "a").await?.into_query()?;
        let b = processor.get_input(self, "b").await?.into_query()?;

        let query = format!("({a} - {b});");

        Ok(query.into())
    }
}
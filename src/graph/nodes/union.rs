use serde::Deserialize;

use crate::graph::{errors::GraphError, output::NodeOutput, process::NodeProcessor, Node};

#[derive(Deserialize, Debug)]
pub struct Union {
    id: String,
}

impl Node for Union {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let mut a_collection = processor.get_input(self, "a").await?.into_features()?;
        let b_collection = processor.get_input(self, "b").await?.into_features()?;

        a_collection.features.extend(b_collection.features);
        Ok(a_collection.into())
    }
}

use serde::Deserialize;

use crate::graph::{
    errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor, Control,
};

#[derive(Deserialize, Debug)]
pub struct OqlStatement {
    id: String,

    nodes: Control<bool>,
    ways: Control<bool>,
    relations: Control<bool>,

    key: Control<String>,
    value: Control<String>,
}

#[async_trait::async_trait]
impl Node for OqlStatement {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, _processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let f = match (self.nodes.value, self.ways.value, self.relations.value) {
            (true, true, true) => "nwr",
            (true, true, false) => "nw",
            (true, false, true) => "nr",
            (false, true, true) => "wr",
            (true, false, false) => "node",
            (false, true, false) => "way",
            (false, false, true) => "relation",
            (false, false, false) => "nwr",
        };

        let round = "{{bbox}}";
        if self.value.value.is_empty() {
            Ok(format!("{f}[{}]({round});", self.key.value).into())
        } else {
            Ok(format!("{f}[{}={}]({round});", self.key.value, self.value.value).into())
        }
    }
}

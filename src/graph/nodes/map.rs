use serde::Deserialize;

use crate::graph::Node;

#[derive(Deserialize, Debug)]
pub struct Map {
    id: String,
}

impl Node for Map {
    fn id(&self) -> &str {
        &self.id
    }
}

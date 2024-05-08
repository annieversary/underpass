use crate::graph::errors::GraphError;
use geojson::FeatureCollection;

#[derive(Clone)]
pub enum NodeOutput {
    Features(FeatureCollection),
    Query(String),
}
impl NodeOutput {
    pub fn into_features(self) -> Result<FeatureCollection, GraphError> {
        if let NodeOutput::Features(val) = self {
            Ok(val)
        } else {
            Err(GraphError::WrongInputType {
                got: "query".into(),
                expected: "geojson".into(),
            })
        }
    }
    pub fn into_query(self) -> Result<String, GraphError> {
        if let NodeOutput::Query(val) = self {
            Ok(val)
        } else {
            Err(GraphError::WrongInputType {
                got: "geojson".into(),
                expected: "query".into(),
            })
        }
    }
}
impl From<FeatureCollection> for NodeOutput {
    fn from(value: FeatureCollection) -> Self {
        Self::Features(value)
    }
}
impl From<String> for NodeOutput {
    fn from(value: String) -> Self {
        Self::Query(value)
    }
}

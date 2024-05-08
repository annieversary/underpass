use crate::graph::{
    errors::GraphError,
    output::NodeOutput,
    process::NodeProcessor,
    utils::{new_id, RAF_NUMBER},
    Control, Node,
};
use geo::{GeodesicBearing, Point};
use geojson::{Feature, FeatureCollection, Value};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RoadAngleFilter {
    id: String,

    min: Control<f64>,
    max: Control<f64>,
}

#[async_trait::async_trait]
impl Node for RoadAngleFilter {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let collection = processor.get_input(self, "in").await?.into_features()?;

        let res = filter(collection, self.min.value, self.max.value, &self.id)?;
        Ok(res.into())
    }
}

fn filter(
    collection: FeatureCollection,
    min: f64,
    max: f64,
    node_id: &str,
) -> Result<FeatureCollection, GraphError> {
    // TODO add client-side validation too

    if min > max {
        Err(GraphError::RoadAngle {
            message: "The min flag has a greater value than the max flag!".to_string(),
            node_id: node_id.to_string(),
        })?;
    }

    let ways =
        collection
            .features
            .iter()
            .filter_map(|w| match w.geometry.as_ref().map(|g| &g.value) {
                Some(Value::LineString(coords)) => Some((w, coords)),
                _ => None,
            });

    let features = ways
        .flat_map(|(way, coords)| {
            if coords.len() < 2 {
                return vec![];
            }

            let coords = coords
                .iter()
                .map(|vec| Point::new(vec[0], vec[1]))
                .collect::<Vec<_>>();

            coords
                .windows(2)
                .flat_map(|pair| {
                    let bearing: f64 = get_bearing(pair[0], pair[1]);
                    if bearing > min && bearing < max {
                        Some(Feature {
                            id: way.id.clone().and_then(|id| new_id(id, RAF_NUMBER)),
                            geometry: Some(
                                Value::LineString(vec![
                                    vec![pair[0].x(), pair[0].y()],
                                    vec![pair[1].x(), pair[1].y()],
                                ])
                                .into(),
                            ),
                            properties: way.properties.clone(),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
}

fn get_bearing(p1: Point, p2: Point) -> f64 {
    let mut bearing = p1.geodesic_bearing(p2);

    // we want bearing only between -90 and +90
    if bearing < -90.0 {
        bearing += 180.0;
    } else if bearing > 90.0 {
        bearing -= 180.0;
    }

    bearing
}

use crate::{
    elevation::ElevationMap,
    graph::{
        errors::GraphError,
        nodes::Node,
        output::NodeOutput,
        process::NodeProcessor,
        utils::{new_id, RAF_NUMBER},
        Control,
    },
};
use geojson::{Feature, FeatureCollection, Value};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ElevationFilter {
    min: Control<i32>,
    max: Control<i32>,
}

#[async_trait::async_trait]
impl Node for ElevationFilter {
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
        node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        let collection = processor.get_input(node_id, "in").await?.into_features()?;

        let res = filter(
            collection,
            self.min.value,
            self.max.value,
            node_id,
            processor.elevation_map,
        )?;
        Ok(res.into())
    }
}

fn filter(
    collection: FeatureCollection,
    min: i32,
    max: i32,
    node_id: &str,
    map: &ElevationMap,
) -> Result<FeatureCollection, GraphError> {
    let _span = tracing::trace_span!("elevation_filter::filter");
    let _span = _span.enter();

    // TODO add client-side validation too

    let mut map = map.cached();

    if min > max {
        Err(GraphError::RoadAngle {
            message: "The min flag has a greater value than the max flag!".to_string(),
            node_id: node_id.to_string(),
        })?;
    }

    let features = collection
        .features
        .into_iter()
        .flat_map(|feature| {
            let Some(geo) = &feature.geometry else {
                return vec![];
            };

            match &geo.value {
                Value::Point(point) => {
                    let elevation = map.lookup_or_0(point[0], point[1]);
                    if min <= elevation && elevation <= max {
                        return vec![feature];
                    }
                }
                Value::LineString(line) => {
                    return line
                        .windows(2)
                        .flat_map(|pair| {
                            let elevation1 = map.lookup_or_0(pair[0][0], pair[0][1]);
                            let elevation2 = map.lookup_or_0(pair[1][0], pair[1][1]);
                            if (min <= elevation1 && elevation1 <= max)
                                || (min <= elevation2 && elevation2 <= max)
                            {
                                return Some(Feature {
                                    id: feature.id.clone().and_then(|id| new_id(id, RAF_NUMBER)),
                                    geometry: Some(Value::LineString(pair.to_vec()).into()),
                                    properties: feature.properties.clone(),
                                    ..Default::default()
                                });
                            }

                            None
                        })
                        .collect();
                }
                Value::GeometryCollection(_) => unimplemented!(),
                // we dont actually have any other type
                _ => unreachable!(),
            }

            vec![]
        })
        .collect();

    tracing::trace!("there were {} cached datasets", map.cache.len());

    Ok(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
}

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
        )
        .await?;
        Ok(res.into())
    }
}

async fn filter(
    collection: FeatureCollection,
    min: i32,
    max: i32,
    node_id: &str,
    map: &ElevationMap,
) -> Result<FeatureCollection, GraphError> {
    // TODO add client-side validation too

    if min > max {
        Err(GraphError::RoadAngle {
            message: "The min flag has a greater value than the max flag!".to_string(),
            node_id: node_id.to_string(),
        })?;
    }

    let mut features = vec![];
    for feature in collection.features {
        let Some(geo) = &feature.geometry else {
            continue;
        };

        match &geo.value {
            Value::Point(point) => {
                let elevation = map.lookup_or_0(point[0], point[1]).await;

                if min <= elevation && elevation <= max {
                    features.push(feature);
                }
            }
            Value::LineString(line) => {
                for pair in line.windows(2) {
                    let elevation1 = map.lookup_or_0(pair[0][0], pair[0][1]).await;
                    let elevation2 = map.lookup_or_0(pair[1][0], pair[1][1]).await;

                    if (min <= elevation1 && elevation1 <= max)
                        || (min <= elevation2 && elevation2 <= max)
                    {
                        features.push(Feature {
                            id: feature.id.clone().and_then(|id| new_id(id, RAF_NUMBER)),
                            geometry: Some(Value::LineString(pair.to_vec()).into()),
                            properties: feature.properties.clone(),
                            ..Default::default()
                        });
                    }
                }
            }
            Value::GeometryCollection(_) => unimplemented!(),
            // we dont actually have any other type
            _ => unreachable!(),
        }
    }

    Ok(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
}

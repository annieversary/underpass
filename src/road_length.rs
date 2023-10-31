use crate::search::GraphError;
use geo::{GeodesicLength, LineString, Point};
use geojson::{Feature, FeatureCollection, Value};

pub fn filter(
    collection: FeatureCollection,
    min: f64,
    max: f64,
    node_id: &str,
) -> Result<FeatureCollection, GraphError> {
    // TODO add client-side validation too

    if min > max {
        Err(GraphError::RoadLength {
            message: "The min flag has a greater value than the max flag".to_string(),
            node_id: node_id.to_string(),
        })?;
    }

    if min < 0.0 || max < 0.0 {
        Err(GraphError::RoadLength {
            message: "Min and Max have to be positive values".to_string(),
            node_id: node_id.to_string(),
        })?;
    }

    let ways =
        collection
            .features
            .iter()
            .filter_map(|w| match w.geometry.as_ref().map(|g| &g.value) {
                Some(Value::LineString(w)) => Some(w),
                _ => None,
            });

    let features = ways
        .flat_map(|way| {
            let coords = way
                .iter()
                .map(|vec| Point::new(vec[0], vec[1]))
                .collect::<Vec<_>>();

            coords
                .windows(2)
                .flat_map(|pair| {
                    let distance: f64 = LineString::from(pair.to_vec()).geodesic_length();

                    if min < distance && distance < max {
                        Some(Feature {
                            // TODO copy properties over and also add a `osm_id` one?
                            geometry: Some(
                                Value::LineString(vec![
                                    vec![pair[0].x(), pair[0].y()],
                                    vec![pair[1].x(), pair[1].y()],
                                ])
                                .into(),
                            ),
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

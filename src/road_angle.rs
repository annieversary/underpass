use crate::search::GraphError;
use geo::{GeodesicBearing, Point};
use geojson::{Feature, FeatureCollection, Value};

pub fn filter(
    collection: FeatureCollection,
    min: f64,
    max: f64,
) -> Result<FeatureCollection, GraphError> {
    // TODO add client-side validation too
    if min > max {
        Err(GraphError::RoadAngle {
            message: "The min flag has a greater value than the max flag!".to_string(),
            // TODO
            node_id: todo!(),
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
                    let bearing: f64 = get_bearing(pair[0], pair[1]);
                    if bearing > min && bearing < max {
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

fn get_bearing(p1: Point, p2: Point) -> f64 {
    let mut bearing = p1.geodesic_bearing(p2);
    if bearing < -90.0 {
        bearing += 180.0;
    } else if bearing > 90.0 {
        bearing -= 180.0;
    }

    bearing
}

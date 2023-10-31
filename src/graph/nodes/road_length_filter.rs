use crate::graph::{
    utils::{bearing_distance, new_id, RLF_NUMBER},
    GraphError,
};
use geo::{GeodesicBearing, Point};
use geojson::{Feature, FeatureCollection, Value};

pub fn filter(
    collection: FeatureCollection,
    min: f64,
    max: f64,
    tolerance: f64,
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
                Some(Value::LineString(coords)) => Some((w, coords)),
                _ => None,
            });

    let features = ways
        .flat_map(|(way, coords)| {
            if coords.len() < 2 {
                return vec![];
            }

            let mut coords = coords
                .iter()
                .map(|vec| Point::new(vec[0], vec[1]))
                .peekable();

            // make groups by tolerance

            // like if tolerance = 10, and the first has bearing 45,
            // grab all the nodes with bearing between 35 and 55
            // as soon as the next one has more bearing than that, start a new group

            let mut points = vec![];

            while let Some(p1) = coords.next() {
                let Some(p2) = coords.next() else {
                    break;
                };
                let base_bearing = p1.geodesic_bearing(p2);

                let mut ps = vec![p1, p2];
                while let Some(p3) = coords.peek() {
                    let bearing = p1.geodesic_bearing(*p3);
                    if bearing_distance(base_bearing, bearing) < tolerance {
                        ps.push(coords.next().unwrap());
                    } else {
                        break;
                    }
                }

                points.push(ps);
            }

            points
                .into_iter()
                .flat_map(|road| {
                    // road has at least two elements
                    let first = road.first().unwrap();
                    let last = road.last().unwrap();
                    let (_bearing, distance) = first.geodesic_bearing_distance(*last);

                    if min < distance && distance < max {
                        Some(Feature {
                            id: way.id.clone().and_then(|id| new_id(id, RLF_NUMBER)),
                            geometry: Some(
                                Value::LineString(
                                    road.into_iter().map(|p| vec![p.x(), p.y()]).collect(),
                                )
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

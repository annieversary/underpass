use crate::{osm_to_geojson::*, search::SearchError};
use geo::{GeodesicBearing, Point};
use geojson::{Feature, FeatureCollection, Value};
use std::collections::BTreeMap;

pub fn filter(osm: Osm, min: f64, max: f64) -> Result<FeatureCollection, SearchError> {
    // TODO change to take in a geojson::FeatureCollection?
    // it would be much nicer if we can make everything be a GeoJson -> GeoJson
    // it simplifies a bunch since LineStrings already include all the coordinates in geometry

    // TODO add client-side validation too
    if min > max {
        return Err(SearchError::RoadAngle(
            "The min flag has a greater value than the max flag!".to_string(),
        ));
    }

    let ways = osm.elements.iter().filter_map(|w| match w {
        Element::Way(w) => Some(w),
        _ => None,
    });
    let nodes = osm.elements.iter().filter_map(|n| match n {
        Element::Node(n) => Some(n),
        _ => None,
    });

    let node_map = BTreeMap::from_iter(nodes.map(|n| (n.id, n)));

    let features = ways
        .flat_map(|way| {
            let coords = way
                .nodes
                .iter()
                .flat_map(|id| {
                    let node = node_map.get(id)?;

                    Some(Point::new(node.lon, node.lat))
                })
                .collect::<Vec<_>>();

            coords
                .windows(2)
                .flat_map(|pair| {
                    let bearing: f64 = get_bearing(pair[0], pair[1]);
                    if bearing > min && bearing < max {
                        Some(Feature {
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

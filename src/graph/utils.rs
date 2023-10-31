use geojson::feature::Id;

use super::GraphConnection;

pub fn detect_cycles(connections: &[GraphConnection]) -> bool {
    use petgraph::{
        prelude::*,
        visit::{depth_first_search, DfsEvent},
    };

    let g = GraphMap::<&str, (), Directed>::from_edges(
        connections
            .iter()
            .map(|c| (c.source.as_str(), c.target.as_str())),
    );

    let g = g.into_graph::<usize>();
    depth_first_search(&g, g.node_indices(), |event| match event {
        DfsEvent::BackEdge(_, _) => Err(()),
        _ => Ok(()),
    })
    .is_err()
}

pub const RLF_NUMBER: u64 = 1000000000000000;
pub const RAF_NUMBER: u64 = 2000000000000000;
pub fn new_id(id: Id, number: u64) -> Option<Id> {
    match id {
        Id::Number(n) if n.is_u64() => Some(Id::Number((n.as_u64().unwrap() + number).into())),
        _ => None,
    }
}

/// returns angular distance between bearings
///
/// return value is always positive, and less than 180
pub fn bearing_distance(bearing1: f64, bearing2: f64) -> f64 {
    // bearing is between -180 and +180
    // we map it to 0 <> 360
    let b1 = bearing1 + 180.0;
    let b2 = bearing2 + 180.0;

    let diff = (b1 - b2).abs();
    // https://stackoverflow.com/a/6193318
    diff.min(360.0 - diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearing_distance() {
        fn t(a: f64, b: f64) -> bool {
            (a - b).abs() < 0.00001
        }

        assert!(t(1.0, bearing_distance(0.0, 1.0)));
        assert!(t(2.0, bearing_distance(-1.0, 1.0)));
        assert!(t(3.0, bearing_distance(-179.0, 178.0)));
    }
}

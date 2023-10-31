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

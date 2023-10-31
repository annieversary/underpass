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

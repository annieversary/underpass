use std::collections::{BTreeMap, HashMap};

use geojson::FeatureCollection;

use crate::{
    cache::Caches,
    elevation::ElevationMap,
    graph::{
        errors::GraphError, nodes::Node, output::NodeOutput, utils::detect_cycles, Graph,
        GraphConnection, GraphNode,
    },
    search::{Bbox, GeocodeaArea, SearchError},
};

pub struct ProcessResult {
    pub collection: FeatureCollection,
    pub geocode_areas: Vec<GeocodeaArea>,
    pub processed_queries: HashMap<String, String>,
}

impl Default for ProcessResult {
    fn default() -> Self {
        Self {
            collection: FeatureCollection {
                bbox: None,
                features: vec![],
                foreign_members: None,
            },
            geocode_areas: Default::default(),
            processed_queries: Default::default(),
        }
    }
}

pub async fn process_graph(
    graph: Graph,
    bbox: Bbox,
    elevation_map: &ElevationMap,
    caches: Caches,
) -> Result<ProcessResult, SearchError> {
    if detect_cycles(&graph.connections) {
        Err(GraphError::Cycle)?;
    }

    let nodes = BTreeMap::from_iter(graph.nodes.iter().map(|n| (n.id(), n)));

    let map_id = graph
        .nodes
        .iter()
        .find(|n| matches!(n, GraphNode::Map(_)))
        .ok_or(GraphError::MapMissing)?
        .id();

    let Some(con) = graph.connections.iter().find(|c| c.target == map_id) else {
        return Ok(ProcessResult::default());
    };
    let prev = nodes
        .get(con.source.as_str())
        .ok_or(GraphError::ConnectionNodeMissing)?;

    let mut np = NodeProcessor {
        nodes: &nodes,
        connections: graph.connections,
        bbox,
        geocode_areas: vec![],
        processed_queries: Default::default(),
        memory: Default::default(),

        elevation_map,
        caches,
    };

    let collection = np.process_node(prev).await?.into_features()?;

    Ok(ProcessResult {
        collection,
        geocode_areas: np.geocode_areas,
        processed_queries: np.processed_queries,
    })
}

pub struct NodeProcessor<'a> {
    // i dont think these two lifetimes are the same but meh
    nodes: &'a BTreeMap<&'a str, &'a GraphNode>,
    connections: Vec<GraphConnection>,
    pub bbox: Bbox,
    pub geocode_areas: Vec<GeocodeaArea>,
    pub processed_queries: HashMap<String, String>,
    memory: HashMap<String, NodeOutput>,

    pub elevation_map: &'a ElevationMap,
    pub caches: Caches,
}

// NOTE: this whole thing assumes every node has only one type of output
// it will need adapting to support multiple outputs

impl<'a> NodeProcessor<'a> {
    /// find a connection that targets `n` on the `target` input
    fn find_connection(&self, n: &dyn Node, target: &str) -> Result<&GraphConnection, GraphError> {
        self.connections
            .iter()
            .find(|c| c.target == n.id() && target == c.target_input)
            .ok_or_else(|| GraphError::InputMissing {
                node_id: n.id().to_string(),
            })
    }

    /// get node by id
    fn get_node<'b>(&'b self, id: &'_ str) -> Result<&'b &'a GraphNode, GraphError> {
        self.nodes.get(id).ok_or(GraphError::ConnectionNodeMissing)
    }

    /// get and compute the node connected to input `name`
    pub async fn get_input(
        &mut self,
        node: &(dyn Node + Send + Sync),
        name: &str,
    ) -> Result<NodeOutput, GraphError> {
        let con = self.find_connection(node, name)?;
        let prev = self.get_node(&con.source)?;
        self.process_node(prev).await
    }

    #[async_recursion::async_recursion]
    async fn process_node(&mut self, n: &GraphNode) -> Result<NodeOutput, GraphError> {
        if let Some(res) = self.memory.get(n.id()) {
            return Ok(res.clone());
        }

        let res = n.process(self).await?;

        self.memory.insert(n.id().to_string(), res.clone());

        Ok(res)
    }
}

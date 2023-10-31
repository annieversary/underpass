use std::collections::{BTreeMap, HashMap};

use geojson::FeatureCollection;
use serde::Deserialize;

use crate::{
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
    road_angle, road_length,
    search::{Bbox, GeocodeaArea, GraphError, SearchError},
};

pub struct GraphResult {
    pub collection: FeatureCollection,
    pub geocode_areas: Vec<GeocodeaArea>,
    pub processed_queries: HashMap<String, String>,
}

impl Default for GraphResult {
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

pub async fn process_graph(graph: Graph, bbox: Bbox) -> Result<GraphResult, SearchError> {
    if detect_cycles(&graph.connections) {
        Err(GraphError::Cycle)?;
    }

    let nodes = BTreeMap::from_iter(graph.nodes.iter().map(|n| (n.id.clone(), n)));

    let map_id = graph
        .nodes
        .iter()
        .find(|n| matches!(n.node, GraphNodeInternal::Map {}))
        .ok_or(GraphError::MapMissing)?
        .id
        .clone();

    let Some(con) = graph.connections.iter().find(|c| c.target == map_id) else {
        return Ok(GraphResult::default());
    };
    let prev = nodes
        .get(&con.source)
        .ok_or(GraphError::ConnectionNodeMissing)?;

    let mut np = NodeProcessor {
        nodes: &nodes,
        connections: graph.connections,
        bbox,
        geocode_areas: vec![],
        processed_queries: Default::default(),
        memory: Default::default(),
    };

    let collection = np.process_node(prev).await?;

    Ok(GraphResult {
        collection,
        geocode_areas: np.geocode_areas,
        processed_queries: np.processed_queries,
    })
}

struct NodeProcessor<'a> {
    // i dont think these two lifetimes are the same but meh
    nodes: &'a BTreeMap<String, &'a GraphNode>,
    connections: Vec<GraphConnection>,
    bbox: Bbox,
    geocode_areas: Vec<GeocodeaArea>,
    processed_queries: HashMap<String, String>,
    memory: HashMap<String, FeatureCollection>,
}

impl<'a> NodeProcessor<'a> {
    fn find_connection(
        &self,
        n: &GraphNode,
        target: Option<&str>,
    ) -> Result<&GraphConnection, GraphError> {
        self.connections
            .iter()
            .find(|c| c.target == n.id && target.map(|t| t == c.target_input).unwrap_or(true))
            .ok_or_else(|| GraphError::InputMissing {
                node_id: n.id.clone(),
            })
    }

    fn get_node<'b>(&'b self, id: &'_ str) -> Result<&'b &'a GraphNode, GraphError> {
        self.nodes.get(id).ok_or(GraphError::ConnectionNodeMissing)
    }

    #[async_recursion::async_recursion]
    async fn process_node(&mut self, n: &GraphNode) -> Result<FeatureCollection, SearchError> {
        if let Some(res) = self.memory.get(&n.id) {
            return Ok(res.clone());
        }

        let res: Result<FeatureCollection, SearchError> = match &n.node {
            GraphNodeInternal::Map {} => unreachable!(),
            GraphNodeInternal::Oql { query } => {
                let (query, found_areas) =
                    preprocess_query(&query.value, &self.bbox, OsmNominatim).await?;

                let client = reqwest::Client::new();
                let res = client
                    .post("https://overpass-api.de/api/interpreter")
                    .body(query.clone())
                    .send()
                    .await?;

                if res.status() == 200 {
                    let osm: Osm = res.json().await.map_err(SearchError::JsonParse)?;

                    self.geocode_areas.extend(found_areas);
                    self.processed_queries.insert(n.id.clone(), query);

                    Ok(osm_to_geojson(osm))
                } else {
                    let res = res.text().await?;
                    Err(GraphError::OqlSyntax {
                        node_id: n.id.clone(),
                        error: res,
                        query,
                    }
                    .into())
                }
            }
            GraphNodeInternal::RoadAngleFilter { min, max } => {
                let con = self.find_connection(n, None)?;
                let prev = self.get_node(&con.source)?;

                let collection = self.process_node(prev).await?;
                let res = road_angle::filter(collection, min.value, max.value, &n.id)?;
                Ok(res)
            }
            GraphNodeInternal::RoadLengthFilter { min, max } => {
                let con = self.find_connection(n, None)?;
                let prev = self.get_node(&con.source)?;

                let collection = self.process_node(prev).await?;
                let res = road_length::filter(collection, min.value, max.value, &n.id)?;
                Ok(res)
            }
            // not actually implemented
            GraphNodeInternal::InViewOf {} => {
                let input_con = self.find_connection(n, Some("in"))?;
                let input_prev = self.get_node(&input_con.source)?;
                let _input_collection = self.process_node(input_prev).await?;

                let aux_con = self.find_connection(n, Some("aux"))?;
                let aux_prev = self.get_node(&aux_con.source)?;
                let _aux_collection = self.process_node(aux_prev).await?;

                // TODO filter input_collection by whether it can see the aux_collection

                todo!()
            }
        };

        let res = res?;

        self.memory.insert(n.id.clone(), res.clone());

        Ok(res)
    }
}

fn detect_cycles(connections: &[GraphConnection]) -> bool {
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

#[derive(Deserialize, Debug)]
pub struct Graph {
    nodes: Vec<GraphNode>,
    connections: Vec<GraphConnection>,
}
#[derive(Deserialize, Debug)]
pub struct GraphNode {
    id: String,
    #[serde(flatten)]
    node: GraphNodeInternal,
}

// we dont need the inputs/outputs in here cause we can assume they're gonna be the same
#[derive(Deserialize, Debug)]
#[serde(tag = "label", content = "controls")]
pub enum GraphNodeInternal {
    #[serde(rename = "Road Angle Filter")]
    RoadAngleFilter {
        min: Control<f64>,
        max: Control<f64>,
    },
    #[serde(rename = "Road Length Filter")]
    RoadLengthFilter {
        min: Control<f64>,
        max: Control<f64>,
    },
    Oql {
        // this eventually will have the code for this node
        query: Control<String>,
    },
    Map {},
    InViewOf {},
}

#[derive(Deserialize, Debug)]
pub struct Control<T> {
    #[serde(rename = "id")]
    _id: String,
    value: T,
}

#[derive(Deserialize, Debug)]
pub struct GraphConnection {
    #[serde(rename = "id")]
    _id: String,
    source: String,
    #[serde(rename = "sourceOutput")]
    _source_output: String,
    target: String,
    #[serde(rename = "targetInput")]
    target_input: String,
}

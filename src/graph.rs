use std::collections::{BTreeMap, HashMap};

use geojson::FeatureCollection;
use serde::Deserialize;

use crate::{
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
    road_angle,
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

    let mut geocode_areas = vec![];
    let mut processed_queries = HashMap::new();

    let Some(con) = graph.connections.iter().find(|c| c.target == map_id) else {
        return Ok(GraphResult::default());
    };
    let prev = nodes
        .get(&con.source)
        .ok_or(GraphError::ConnectionNodeMissing)?;

    let collection = process_node(
        prev,
        &nodes,
        &graph.connections,
        bbox,
        &mut geocode_areas,
        &mut processed_queries,
    )
    .await?;

    Ok(GraphResult {
        collection,
        geocode_areas,
        processed_queries,
    })
}

#[async_recursion::async_recursion]
async fn process_node(
    n: &GraphNode,
    nodes: &BTreeMap<String, &GraphNode>,
    connections: &[GraphConnection],
    bbox: Bbox,
    geocode_areas: &mut Vec<GeocodeaArea>,
    processed_queries: &mut HashMap<String, String>,
) -> Result<FeatureCollection, SearchError> {
    match &n.node {
        GraphNodeInternal::RoadAngleFilter { min, max } => {
            let con = connections
                .iter()
                .find(|c| c.target == n.id)
                .ok_or_else(|| GraphError::InputMissing {
                    node_id: n.id.clone(),
                })?;

            let prev = nodes
                .get(&con.source)
                .ok_or(GraphError::ConnectionNodeMissing)?;
            let collection = process_node(
                prev,
                nodes,
                connections,
                bbox,
                geocode_areas,
                processed_queries,
            )
            .await?;
            let res = road_angle::filter(collection, min.value, max.value)?;
            Ok(res)
        }
        GraphNodeInternal::Oql { query } => {
            let (query, found_areas) = preprocess_query(&query.value, &bbox, OsmNominatim).await?;

            let client = reqwest::Client::new();
            let res = client
                .post("https://overpass-api.de/api/interpreter")
                .body(query.clone())
                .send()
                .await?;

            if res.status() == 200 {
                let osm: Osm = res.json().await.map_err(SearchError::JsonParse)?;

                geocode_areas.extend(found_areas);
                processed_queries.insert(n.id.clone(), query);

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
        // not actually implemented
        GraphNodeInternal::InViewOf {} => {
            let input_con = connections
                .iter()
                .find(|c| c.target == n.id && c.target_input == "in")
                .ok_or_else(|| GraphError::InputMissing {
                    node_id: n.id.clone(),
                })?;
            let input_prev = nodes
                .get(&input_con.source)
                .ok_or(GraphError::ConnectionNodeMissing)?;
            let _input_collection = process_node(
                input_prev,
                nodes,
                connections,
                bbox,
                geocode_areas,
                processed_queries,
            )
            .await?;

            let aux_con = connections
                .iter()
                .find(|c| c.target == n.id && c.target_input == "aux")
                .ok_or_else(|| GraphError::InputMissing {
                    node_id: n.id.clone(),
                })?;
            let aux_prev = nodes
                .get(&aux_con.source)
                .ok_or(GraphError::ConnectionNodeMissing)?;
            let _aux_collection = process_node(
                aux_prev,
                nodes,
                connections,
                bbox,
                geocode_areas,
                processed_queries,
            )
            .await?;

            // TODO filter input_collection by whether it can see the aux_collection

            todo!()
        }
        GraphNodeInternal::Map {} => unreachable!(),
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

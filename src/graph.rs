use std::collections::{BTreeMap, HashMap};

use geojson::FeatureCollection;
use serde::Deserialize;

use crate::{
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
    road_angle,
    search::{Bbox, GeocodeaArea, SearchError},
};

pub struct GraphResult {
    pub collection: FeatureCollection,
    pub geocode_areas: Vec<GeocodeaArea>,
    pub processed_queries: HashMap<String, String>,
}

pub async fn process_graph(graph: Graph, bbox: Bbox) -> Result<GraphResult, SearchError> {
    if detect_cycles(&graph.connections) {
        return Err(SearchError::CyclicalGraph);
    }

    let nodes = BTreeMap::from_iter(graph.nodes.iter().map(|n| (n.id.clone(), n)));

    let map_id = graph
        .nodes
        .iter()
        .find(|n| matches!(n.node, GraphNodeInternal::Map {}))
        .unwrap()
        .id
        .clone();

    let mut geocode_areas = vec![];
    let mut processed_queries = HashMap::new();

    let con = graph
        .connections
        .iter()
        .find(|c| c.target == map_id)
        .unwrap();
    let prev = nodes.get(&con.source).unwrap();

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
            let con = connections.iter().find(|c| c.target == n.id).unwrap();
            let prev = nodes.get(&con.source).unwrap();
            let collection = process_node(
                prev,
                nodes,
                connections,
                bbox,
                geocode_areas,
                processed_queries,
            )
            .await?;
            road_angle::filter(collection, min.value, max.value)
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
                Err(SearchError::Syntax { error: res, query })
            }
        }
        // not actually implemented
        GraphNodeInternal::InViewOf {} => {
            let input_con = connections
                .iter()
                .find(|c| c.target == n.id && c.target_input == "in")
                .unwrap();
            let input_prev = nodes.get(&input_con.source).unwrap();
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
                .unwrap();
            let aux_prev = nodes.get(&aux_con.source).unwrap();
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
    id: String,
    value: T,
}

#[derive(Deserialize, Debug)]
pub struct GraphConnection {
    id: String,
    source: String,
    #[serde(rename = "sourceOutput")]
    source_output: String,
    target: String,
    #[serde(rename = "targetInput")]
    target_input: String,
}

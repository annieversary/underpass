use std::collections::BTreeMap;

use geojson::FeatureCollection;
use serde::Deserialize;

use crate::{
    osm_to_geojson::{osm_to_geojson, Osm},
    road_angle,
    search::SearchError,
};

pub async fn process_graph(graph: Graph, query: String) -> Result<FeatureCollection, SearchError> {
    let nodes = BTreeMap::from_iter(graph.nodes.iter().map(|n| (n.id.clone(), n)));

    let map_id = graph
        .nodes
        .iter()
        .find(|n| matches!(n.node, GraphNodeInternal::Map {}))
        .unwrap()
        .id
        .clone();

    let con = graph
        .connections
        .iter()
        .find(|c| c.target == map_id)
        .unwrap();
    let prev = nodes.get(&con.source).unwrap();
    process_node(prev, &nodes, &graph.connections, query).await
}

#[async_recursion::async_recursion]
async fn process_node(
    n: &GraphNode,
    nodes: &BTreeMap<String, &GraphNode>,
    connections: &[GraphConnection],
    query: String,
) -> Result<FeatureCollection, SearchError> {
    match &n.node {
        GraphNodeInternal::RoadAngleFilter { min, max } => {
            let con = connections.iter().find(|c| c.target == n.id).unwrap();
            let prev = nodes.get(&con.source).unwrap();
            let collection = process_node(prev, nodes, connections, query).await?;
            road_angle::filter(collection, min.value, max.value)
        }
        GraphNodeInternal::Oql { query: _ } => {
            let client = reqwest::Client::new();
            let res = client
                .post("https://overpass-api.de/api/interpreter")
                .body(query.clone())
                .send()
                .await?;

            if res.status() == 200 {
                let osm: Osm = res.json().await.map_err(SearchError::JsonParse)?;

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
            let _input_collection =
                process_node(input_prev, nodes, connections, query.clone()).await?;

            let aux_con = connections
                .iter()
                .find(|c| c.target == n.id && c.target_input == "aux")
                .unwrap();
            let aux_prev = nodes.get(&aux_con.source).unwrap();
            let _aux_collection = process_node(aux_prev, nodes, connections, query).await?;

            // filter input_collection by whether it can see the aux_collection

            todo!()
        }
        GraphNodeInternal::Map {} => unreachable!(),
    }
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
        query: String,
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

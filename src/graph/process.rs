use std::collections::{BTreeMap, HashMap};

use geojson::FeatureCollection;

use crate::{
    graph::{
        nodes, utils::detect_cycles, Graph, GraphConnection, GraphError, GraphNode,
        GraphNodeInternal, GraphResult,
    },
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
    search::{Bbox, GeocodeaArea, SearchError},
};

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
            GraphNodeInternal::Oql { query, timeout } => {
                let (query, found_areas) =
                    preprocess_query(&query.value, &self.bbox, timeout.value, OsmNominatim).await?;

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
                let res =
                    nodes::road_angle_filter::filter(collection, min.value, max.value, &n.id)?;
                Ok(res)
            }
            GraphNodeInternal::RoadLengthFilter {
                min,
                max,
                tolerance,
            } => {
                let con = self.find_connection(n, None)?;
                let prev = self.get_node(&con.source)?;

                let collection = self.process_node(prev).await?;
                let res = nodes::road_length_filter::filter(
                    collection,
                    min.value,
                    max.value,
                    tolerance.value,
                    &n.id,
                )?;
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

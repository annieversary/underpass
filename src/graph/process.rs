use std::collections::{BTreeMap, HashMap};

use geojson::FeatureCollection;

use crate::{
    elevation::ElevationMap,
    graph::{
        nodes, utils::detect_cycles, Graph, GraphConnection, GraphError, GraphNode,
        GraphNodeInternal, GraphResult,
    },
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
    search::{Bbox, GeocodeaArea, SearchError},
};

pub async fn process_graph(
    graph: Graph,
    bbox: Bbox,
    elevation_map: &ElevationMap,
) -> Result<GraphResult, SearchError> {
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

        elevation_map,
    };

    let collection = np.process_node(prev).await?.into_features()?;

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
    memory: HashMap<String, NodeOutput>,

    elevation_map: &'a ElevationMap,
}

// NOTE: this whole thing assumes every node has only one type of output
// it will need adapting to support multiple outputs

impl<'a> NodeProcessor<'a> {
    /// find a connection that targets `n` on the `target` input
    fn find_connection(&self, n: &GraphNode, target: &str) -> Result<&GraphConnection, GraphError> {
        self.connections
            .iter()
            .find(|c| c.target == n.id && target == c.target_input)
            .ok_or_else(|| GraphError::InputMissing {
                node_id: n.id.clone(),
            })
    }

    /// get node by id
    fn get_node<'b>(&'b self, id: &'_ str) -> Result<&'b &'a GraphNode, GraphError> {
        self.nodes.get(id).ok_or(GraphError::ConnectionNodeMissing)
    }

    /// get and compute the node connected to input `name`
    async fn get_input(&mut self, node: &GraphNode, name: &str) -> Result<NodeOutput, SearchError> {
        let con = self.find_connection(node, name)?;
        let prev = self.get_node(&con.source)?;
        self.process_node(prev).await
    }

    #[async_recursion::async_recursion]
    async fn process_node(&mut self, n: &GraphNode) -> Result<NodeOutput, SearchError> {
        if let Some(res) = self.memory.get(&n.id) {
            return Ok(res.clone());
        }

        let res: Result<NodeOutput, SearchError> = match &n.node {
            GraphNodeInternal::Map {} => unreachable!(),
            // query nodes
            GraphNodeInternal::Oql { query } => Ok(query.value.clone().into()),
            GraphNodeInternal::OqlStatement {
                nodes,
                ways,
                relations,
                key,
                value,
            } => {
                let f = match (nodes.value, ways.value, relations.value) {
                    (true, true, true) => "nwr",
                    (true, true, false) => "nw",
                    (true, false, true) => "nr",
                    (false, true, true) => "wr",
                    (true, false, false) => "node",
                    (false, true, false) => "way",
                    (false, false, true) => "relation",
                    (false, false, false) => "nwr",
                };

                let round = "{{bbox}}";
                if value.value.is_empty() {
                    Ok(format!("{f}[{}]({round});", key.value).into())
                } else {
                    Ok(format!("{f}[{}={}]({round});", key.value, value.value).into())
                }
            }
            GraphNodeInternal::OqlUnion {} => {
                let a = self.get_input(n, "a").await?.into_query()?;
                let b = self.get_input(n, "b").await?.into_query()?;

                let query = format!("({a} {b});");

                Ok(query.into())
            }
            GraphNodeInternal::OqlDifference {} => {
                let a = self.get_input(n, "a").await?.into_query()?;
                let b = self.get_input(n, "b").await?.into_query()?;

                let query = format!("({a} - {b});");

                Ok(query.into())
            }

            // geojson nodes
            GraphNodeInternal::Overpass { timeout } => {
                let query = self.get_input(n, "query").await?.into_query()?;

                let (query, found_areas) =
                    preprocess_query(&query, &self.bbox, timeout.value, OsmNominatim).await?;

                let client = reqwest::Client::new();
                let res = client
                    .post("https://overpass-api.de/api/interpreter")
                    .body(query.clone())
                    .send()
                    .await?;

                if res.status() == 200 {
                    let osm: Osm = res
                        .json()
                        .await
                        .map_err(|_| GraphError::OverpassJsonError)?;

                    self.geocode_areas.extend(found_areas);
                    self.processed_queries.insert(n.id.clone(), query);

                    Ok(osm_to_geojson(osm).into())
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
            GraphNodeInternal::Union {} => {
                let mut a_collection = self.get_input(n, "a").await?.into_features()?;
                let b_collection = self.get_input(n, "b").await?.into_features()?;

                a_collection.features.extend(b_collection.features);
                Ok(a_collection.into())
            }
            GraphNodeInternal::RoadAngleFilter { min, max } => {
                let collection = self.get_input(n, "in").await?.into_features()?;

                let res =
                    nodes::road_angle_filter::filter(collection, min.value, max.value, &n.id)?;
                Ok(res.into())
            }
            GraphNodeInternal::RoadLengthFilter {
                min,
                max,
                tolerance,
            } => {
                let collection = self.get_input(n, "in").await?.into_features()?;

                let res = nodes::road_length_filter::filter(
                    collection,
                    min.value,
                    max.value,
                    tolerance.value,
                    &n.id,
                )?;
                Ok(res.into())
            }
            // not actually implemented
            GraphNodeInternal::InViewOf {} => {
                let _input_collection = self.get_input(n, "in").await?.into_features()?;
                let _aux_collection = self.get_input(n, "aux").await?.into_features()?;

                // TODO filter input_collection by whether it can see the aux_collection

                todo!()
            }
            GraphNodeInternal::ElevationFilter { min, max } => {
                let collection = self.get_input(n, "in").await?.into_features()?;

                let res = nodes::elevation_filter::filter(
                    collection,
                    min.value,
                    max.value,
                    &n.id,
                    &self.elevation_map,
                )?;
                Ok(res.into())
            }
        };

        let res = res?;

        self.memory.insert(n.id.clone(), res.clone());

        Ok(res)
    }
}

#[derive(Clone)]
enum NodeOutput {
    Features(FeatureCollection),
    Query(String),
}
impl NodeOutput {
    fn into_features(self) -> Result<FeatureCollection, GraphError> {
        if let NodeOutput::Features(val) = self {
            Ok(val)
        } else {
            Err(GraphError::WrongInputType {
                got: "query".into(),
                expected: "geojson".into(),
            })
        }
    }
    fn into_query(self) -> Result<String, GraphError> {
        if let NodeOutput::Query(val) = self {
            Ok(val)
        } else {
            Err(GraphError::WrongInputType {
                got: "geojson".into(),
                expected: "query".into(),
            })
        }
    }
}
impl From<FeatureCollection> for NodeOutput {
    fn from(value: FeatureCollection) -> Self {
        Self::Features(value)
    }
}
impl From<String> for NodeOutput {
    fn from(value: String) -> Self {
        Self::Query(value)
    }
}

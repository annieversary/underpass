use geojson::FeatureCollection;
use serde::Deserialize;

use crate::{
    graph::{errors::GraphError, nodes::Node, output::NodeOutput, process::NodeProcessor, Control},
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
    search::{Bbox, GeocodeaArea},
};

#[derive(Deserialize, Debug)]
pub struct Overpass {
    timeout: Control<u32>,
}

#[async_trait::async_trait]
impl Node for Overpass {
    async fn process(
        &self,
        processor: &mut NodeProcessor<'_>,
        node_id: &str,
    ) -> Result<NodeOutput, GraphError> {
        let query = processor.get_input(node_id, "query").await?.into_query()?;

        // cache
        let bbox = processor.bbox;
        let (feature_collection, found_areas, query) = processor
            .caches
            .overpass
            .try_get_with((query.clone(), processor.bbox), async move {
                run(&query, bbox, self.timeout.value, node_id).await
            })
            .await?;

        processor.geocode_areas.extend(found_areas);
        processor
            .processed_queries
            .insert(node_id.to_string(), query);

        Ok(feature_collection.into())
    }
}

async fn run(
    query: &str,
    bbox: Bbox,
    timeout: u32,
    node_id: &str,
) -> Result<(FeatureCollection, Vec<GeocodeaArea>, String), GraphError> {
    let (query, found_areas) = preprocess_query(query, &bbox, timeout, OsmNominatim).await?;

    let client = reqwest::Client::new();
    let res = client
        .post("https://overpass-api.de/api/interpreter")
        .body(query.clone())
        .send()
        .await?;

    if res.status() != 200 {
        let res = res.text().await?;
        return Err(GraphError::OqlSyntax {
            node_id: node_id.to_string(),
            error: res,
            query,
        });
    }

    let osm: Osm = res
        .json()
        .await
        .map_err(|_| GraphError::OverpassJsonError)?;

    let feature_collection = osm_to_geojson(osm);

    Ok((feature_collection, found_areas, query))
}

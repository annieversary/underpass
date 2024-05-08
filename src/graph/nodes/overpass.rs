use serde::Deserialize;

use crate::{
    graph::{errors::GraphError, output::NodeOutput, process::NodeProcessor, Control, Node},
    nominatim::OsmNominatim,
    osm_to_geojson::{osm_to_geojson, Osm},
    preprocess::preprocess_query,
};

#[derive(Deserialize, Debug)]
pub struct Overpass {
    id: String,

    timeout: Control<u32>,
}

#[async_trait::async_trait]
impl Node for Overpass {
    fn id(&self) -> &str {
        &self.id
    }

    async fn process(&self, processor: &mut NodeProcessor<'_>) -> Result<NodeOutput, GraphError> {
        let query = processor.get_input(self, "query").await?.into_query()?;

        let (query, found_areas) =
            preprocess_query(&query, &processor.bbox, self.timeout.value, OsmNominatim).await?;

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

            processor.geocode_areas.extend(found_areas);
            processor.processed_queries.insert(self.id.clone(), query);

            Ok(osm_to_geojson(osm).into())
        } else {
            let res = res.text().await?;
            Err(GraphError::OqlSyntax {
                node_id: self.id.clone(),
                error: res,
                query,
            }
            .into())
        }
    }
}

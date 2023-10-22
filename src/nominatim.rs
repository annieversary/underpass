use crate::search::{GeocodeaArea, SearchError};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait Nominatim {
    async fn search(&self, search: &str) -> Result<(GeocodeaArea, String), SearchError>;
}

pub struct OsmNominatim;
#[async_trait::async_trait]
impl Nominatim for OsmNominatim {
    /// returns ($id,area(id:$id))
    async fn search(&self, search: &str) -> Result<(GeocodeaArea, String), SearchError> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!(
                "https://nominatim.openstreetmap.org/search?format=jsonv2&q={search}"
            ))
            .header("User-Agent", "Underpass, annie@bursary.town")
            .send()
            .await?;

        let res: serde_json::Value = res.json().await?;
        let arr = res
            .as_array()
            .ok_or_else(|| SearchError::Nominatim("response was not an array".to_string()))?;
        if let Some(serde_json::Value::Object(obj)) = arr.get(0) {
            let orig_id = obj
                .get("osm_id")
                .ok_or_else(|| {
                    SearchError::Nominatim("nominatim response did not contain osm_id".to_string())
                })?
                .as_number()
                .ok_or_else(|| SearchError::Nominatim("osm_id was not a number".to_string()))?
                .as_u64()
                .ok_or_else(|| SearchError::Nominatim("osm_id was not a u64".to_string()))?;
            let ty = obj
                .get("osm_type")
                .ok_or_else(|| {
                    SearchError::Nominatim(
                        "nominatim response did not contain osm_type".to_string(),
                    )
                })?
                .as_str()
                .ok_or_else(|| SearchError::Nominatim("osm_type was not a string".to_string()))?;
            let name = obj
                .get("display_name")
                .ok_or_else(|| {
                    SearchError::Nominatim(
                        "nominatim response did not contain display_name".to_string(),
                    )
                })?
                .as_str()
                .ok_or_else(|| {
                    SearchError::Nominatim("display_name was not a string".to_string())
                })?;

            // https://github.com/tyrasd/overpass-turbo/blob/eb216aa08b06590a4efc4e10d6a25140d53fcf70/js/shortcuts.ts#L92

            let mut id = orig_id;
            if ty == "relation" {
                id += 3600000000;
            }

            // TODO change this to return a vec of ids
            // preprocess query should be the one to put it in a string
            let id = if ty == "way" {
                format!("{},{id}", id + 2400000000)
            } else {
                format!("{id}")
            };

            Ok((
                GeocodeaArea {
                    id: orig_id,
                    ty: ty.to_string(),
                    name: name.to_string(),
                    original: search.to_string(),
                },
                format!("area(id:{})", id),
            ))
        } else {
            Err(SearchError::Nominatim(format!(
                "no results found for {search}"
            )))
        }
    }
}

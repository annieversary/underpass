use crate::search::GeocodeaArea;

use thiserror::Error;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[derive(Clone, Debug)]
pub struct NominatimOuput {
    pub ids: Vec<u64>,
    pub area: GeocodeaArea,
}

#[derive(Error, Debug)]
pub enum NominatimError {
    #[error("Nominatim: {0}")]
    Nominatim(String),
    #[error("network error")]
    Network(#[from] reqwest::Error),
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait Nominatim {
    async fn search(&self, search: &str, lang: &str) -> Result<NominatimOuput, NominatimError>;
}

pub struct OsmNominatim;
#[async_trait::async_trait]
impl Nominatim for OsmNominatim {
    /// returns ($id,area(id:$id))
    async fn search(&self, search: &str, lang: &str) -> Result<NominatimOuput, NominatimError> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!(

                "https://nominatim.openstreetmap.org/search?format=jsonv2&accept-language={lang}&q={search}"
            ))
            .header("User-Agent", "Underpass, underpass.versary.town, annie@versary.town")
            .send()
            .await?;

        let res: serde_json::Value = res.json().await?;
        let arr = res
            .as_array()
            .ok_or_else(|| NominatimError::Nominatim("response was not an array".to_string()))?;
        if let Some(serde_json::Value::Object(obj)) = arr.get(0) {
            let id = obj
                .get("osm_id")
                .ok_or_else(|| {
                    NominatimError::Nominatim(
                        "nominatim response did not contain osm_id".to_string(),
                    )
                })?
                .as_number()
                .ok_or_else(|| NominatimError::Nominatim("osm_id was not a number".to_string()))?
                .as_u64()
                .ok_or_else(|| NominatimError::Nominatim("osm_id was not a u64".to_string()))?;
            let ty = obj
                .get("osm_type")
                .ok_or_else(|| {
                    NominatimError::Nominatim(
                        "nominatim response did not contain osm_type".to_string(),
                    )
                })?
                .as_str()
                .ok_or_else(|| {
                    NominatimError::Nominatim("osm_type was not a string".to_string())
                })?;
            let name = obj
                .get("display_name")
                .ok_or_else(|| {
                    NominatimError::Nominatim(
                        "nominatim response did not contain display_name".to_string(),
                    )
                })?
                .as_str()
                .ok_or_else(|| {
                    NominatimError::Nominatim("display_name was not a string".to_string())
                })?;

            // https://github.com/tyrasd/overpass-turbo/blob/eb216aa08b06590a4efc4e10d6a25140d53fcf70/js/shortcuts.ts#L92

            Ok(NominatimOuput {
                // Do not +2400000000 for ways since version 0.7.57,
                // for backward compatibility query both IDs, see
                ids: if ty == "way" {
                    vec![id + 2400000000, id]
                } else if ty == "relation" {
                    vec![id + 3600000000]
                } else {
                    vec![id]
                },
                area: GeocodeaArea {
                    id,
                    ty: ty.to_string(),
                    name: name.to_string(),
                    original: search.to_string(),
                },
            })
        } else {
            Err(NominatimError::Nominatim(format!(
                "no results found for {search}"
            )))
        }
    }
}

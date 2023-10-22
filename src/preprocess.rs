use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

use crate::search::{Bbox, GeocodeaArea, SearchError};

pub async fn preprocess_query(
    query: String,
    bbox: &Bbox,
) -> Result<(String, Vec<GeocodeaArea>), SearchError> {
    let mut geocode_areas = vec![];

    let re = Regex::new(r"\{\{\s*([\w.]+)(:([\S\s]+?))?\}\}").unwrap();

    let mut new = String::with_capacity(query.len());
    let mut last_match = 0;
    for caps in re.captures_iter(&query) {
        let m = caps.get(0).unwrap();
        new.push_str(&query[last_match..m.start()]);

        let replacement = match &caps[1] {
            "out" => "out;>;out skel qt;".to_string(),
            "bbox" => format!(
                "{},{},{},{}",
                bbox.sw[0], bbox.sw[1], bbox.ne[0], bbox.ne[1]
            ),
            "geocodeArea" => {
                let mut r = "(".to_string();
                for s in caps[3].split(';') {
                    let (id, area) = nominatim_search(s.trim()).await?;
                    geocode_areas.push(id);
                    r.push_str(&area);
                    r.push(';');
                }
                r.push(')');
                r
            }
            m if m.starts_with("aroundSelf.") => {
                let set = m.trim_start_matches("aroundSelf.");
                let distance = &caps[3];

                let it = internal_id("it");
                let nearby = internal_id("nearby");
                let empty = internal_id("empty");
                let others = internal_id("others");
                let collect = internal_id("collect");

                format!(
                    "foreach.{set}->.{it}(nwr.{set}(around.{it}:{distance})->.{nearby}; (.{nearby}; - .{it};)->.{others}; (.{collect}; .{others};)->.{collect};); .{empty}->._; .{collect}"
                )
            }
            _ => caps[0].to_string(),
        };

        new.push_str(&replacement);
        last_match = m.end();
    }
    new.push_str(&query[last_match..]);

    Ok((new, geocode_areas))
}

/// returns a unique set id in the format
/// `internal__{id}_{random characters}`
fn internal_id(id: &str) -> String {
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    format!("internal__{id}_{}", random)
}

/// returns ($id,area(id:$id))
async fn nominatim_search(search: &str) -> Result<(GeocodeaArea, String), SearchError> {
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
                SearchError::Nominatim("nominatim response did not contain osm_type".to_string())
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
            .ok_or_else(|| SearchError::Nominatim("display_name was not a string".to_string()))?;

        // https://github.com/tyrasd/overpass-turbo/blob/eb216aa08b06590a4efc4e10d6a25140d53fcf70/js/shortcuts.ts#L92

        let mut id = orig_id;
        if ty == "relation" {
            id += 3600000000;
        }

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

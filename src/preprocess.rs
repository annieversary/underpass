use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

use crate::{
    nominatim::*,
    search::{Bbox, GeocodeaArea, SearchError},
};

pub async fn preprocess_query(
    query: String,
    bbox: &Bbox,
    nominatim: impl Nominatim,
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
                    let out = nominatim.search(s.trim()).await?;

                    let ids = out
                        .ids
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    r.push_str(&format!("area(id:{ids});"));

                    geocode_areas.push(out.area);
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
    #[cfg(test)]
    let rng = rand::rngs::mock::StepRng::new(2, 1);
    #[cfg(not(test))]
    let rng = rand::thread_rng();

    let random: String = rng
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    format!("internal__{id}_{}", random)
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;

    use super::*;

    #[tokio::test]
    async fn test_no_macros_does_nothing() {
        let query = "[out:json][timeout:60];
node[place=city];
out;>;out skel qt;"
            .to_string();
        let nominatim = MockNominatim::new();

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), nominatim)
            .await
            .unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:60];
node[place=city];
out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_out_macro() {
        let query = "[out:json][timeout:60];
nw[amenity=drinking_water];
{{out}}"
            .to_string();
        let nominatim = MockNominatim::new();

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), nominatim)
            .await
            .unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:60];
nw[amenity=drinking_water];
out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_bbox_macro() {
        let query = "[out:json][timeout:60];
node[amenity=drinking_water]({{bbox}});
out;>;out skel qt;"
            .to_string();
        let nominatim = MockNominatim::new();

        let bbox = Bbox {
            ne: [0.3, 1.2345],
            sw: [2.1, 3.0],
        };

        let (processed, _areas) = preprocess_query(query, &bbox, nominatim).await.unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:60];
node[amenity=drinking_water](2.1,3,0.3,1.2345);
out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_geocode_area() {
        let query = "[out:json][timeout:60];
{{geocodeArea:Hokkaido, Japan}}->.japan;
node[place=city](area.japan);
{{out}}"
            .to_string();
        let mut nominatim = MockNominatim::new();

        nominatim
            .expect_search()
            .with(eq("Hokkaido, Japan"))
            .times(1)
            .returning(|_| {
                Ok(NominatimOuput {
                    ids: vec![3606679920],
                    area: GeocodeaArea::default(),
                })
            });

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), nominatim)
            .await
            .unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:60];
(area(id:3606679920);)->.japan;
node[place=city](area.japan);
out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_geocode_area_multiple() {
        let query = "[out:json][timeout:60];
{{geocodeArea:Hokkaido, Japan;Aomori, Japan}}->.japan;
node[place=city](area.japan);
{{out}}"
            .to_string();

        let mut nominatim = MockNominatim::new();
        nominatim
            .expect_search()
            .with(eq("Hokkaido, Japan"))
            .times(1)
            .returning(|_| {
                Ok(NominatimOuput {
                    ids: vec![3606679920],
                    area: GeocodeaArea::default(),
                })
            });
        nominatim
            .expect_search()
            .with(eq("Aomori, Japan"))
            .times(1)
            .returning(|_| {
                Ok(NominatimOuput {
                    ids: vec![3601834655],
                    area: GeocodeaArea::default(),
                })
            });

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), nominatim)
            .await
            .unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:60];
(area(id:3606679920);area(id:3601834655);)->.japan;
node[place=city](area.japan);
out;>;out skel qt;"
        )
    }
}

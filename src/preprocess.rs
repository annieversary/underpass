use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

use crate::{
    graph::errors::GraphError,
    nominatim::*,
    search::{Bbox, GeocodeaArea},
};

pub async fn preprocess_query(
    query: &str,
    bbox: &Bbox,
    timeout: u32,
    nominatim: impl Nominatim,
) -> Result<(String, Vec<GeocodeaArea>), GraphError> {
    let mut geocode_areas = vec![];

    let re = Regex::new(r"\{\{\s*([\w.]+)(:([\S\s]+?))?\}\}").unwrap();

    let mut new = String::with_capacity(query.len());
    new.push_str(&format!("[out:json][timeout:{timeout}];\n\n"));
    let mut last_match = 0;
    for caps in re.captures_iter(query) {
        let m = caps.get(0).unwrap();
        new.push_str(&query[last_match..m.start()]);

        let replacement = match &caps[1] {
            // "out" => "out;>;out skel qt;".to_string(),
            "bbox" => format!(
                "{},{},{},{}",
                bbox.sw[0], bbox.sw[1], bbox.ne[0], bbox.ne[1]
            ),
            "center" => format!(
                "{},{}",
                (bbox.sw[0] + bbox.ne[0]) / 2.0,
                (bbox.sw[1] + bbox.ne[1]) / 2.0
            ),
            "geocodeArea" => {
                let mut r = "(".to_string();
                for s in caps[3].split(';') {
                    let mut params = s.split('@').map(str::trim);
                    let search = params
                        .next()
                        .expect("result of split should have at least one element");
                    let lang = params.next().unwrap_or("en");

                    let out = nominatim.search(search.trim(), lang).await?;

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

    new.push_str("\n\nout;>;out skel qt;");

    Ok((new, geocode_areas))
}

/// returns a unique set id in the format
/// `internal__{id}_{random characters}`
fn internal_id(id: &str) -> String {
    #[cfg(test)]
    // this will always return AAAAA but i dont think it matters for tests
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
    async fn test_empty() {
        let query = "";
        let nominatim = MockNominatim::new();

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), 60, nominatim)
            .await
            .unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:60];



out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_no_macros_does_nothing() {
        let query = "node[place=city];";
        let nominatim = MockNominatim::new();

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), 60, nominatim)
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
    async fn test_bbox_macro() {
        let query = "node[amenity=drinking_water]({{bbox}});";
        let nominatim = MockNominatim::new();

        let bbox = Bbox {
            ne: [0.3, 1.2345],
            sw: [2.1, 3.0],
        };

        let (processed, _areas) = preprocess_query(query, &bbox, 54, nominatim).await.unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:54];

node[amenity=drinking_water](2.1,3,0.3,1.2345);

out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_around_self_macro() {
        let query = "node[amenity=bench]->.benches;
{{aroundSelf.benches:7}}->.benchesAroundOtherBenches;";
        let nominatim = MockNominatim::new();

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), 14, nominatim)
            .await
            .unwrap();

        assert_eq!(
            processed,
            "[out:json][timeout:14];

node[amenity=bench]->.benches;
foreach.benches->.internal__it_AAAAAAAAAA(nwr.benches(around.internal__it_AAAAAAAAAA:7)->.internal__nearby_AAAAAAAAAA; (.internal__nearby_AAAAAAAAAA; - .internal__it_AAAAAAAAAA;)->.internal__others_AAAAAAAAAA; (.internal__collect_AAAAAAAAAA; .internal__others_AAAAAAAAAA;)->.internal__collect_AAAAAAAAAA;); .internal__empty_AAAAAAAAAA->._; .internal__collect_AAAAAAAAAA->.benchesAroundOtherBenches;

out;>;out skel qt;"
        )
    }

    #[tokio::test]
    async fn test_geocode_area() {
        let query = "{{geocodeArea:Hokkaido, Japan}}->.japan;
node[place=city](area.japan);";
        let mut nominatim = MockNominatim::new();

        nominatim.expect_search().times(1).returning(|_, _| {
            Ok(NominatimOuput {
                ids: vec![3606679920],
                area: GeocodeaArea::default(),
            })
        });

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), 60, nominatim)
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
        let query = "{{geocodeArea:Hokkaido, Japan;Aomori, Japan}}->.japan;
node[place=city](area.japan);";

        let mut nominatim = MockNominatim::new();
        nominatim
            .expect_search()
            .with(eq("Hokkaido, Japan"), eq("en"))
            .times(1)
            .returning(|_, _| {
                Ok(NominatimOuput {
                    ids: vec![3606679920],
                    area: GeocodeaArea::default(),
                })
            });
        nominatim
            .expect_search()
            .with(eq("Aomori, Japan"), eq("en"))
            .times(1)
            .returning(|_, _| {
                Ok(NominatimOuput {
                    ids: vec![3601834655],
                    area: GeocodeaArea::default(),
                })
            });

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), 60, nominatim)
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

    #[tokio::test]
    async fn test_geocode_area_with_langs() {
        let query = "{{geocodeArea:Hokkaido, Japan@en;Aomori, Japan@es}}->.japan;
node[place=city](area.japan);";

        let mut nominatim = MockNominatim::new();
        nominatim
            .expect_search()
            .with(eq("Hokkaido, Japan"), eq("en"))
            .times(1)
            .returning(|_, _| {
                Ok(NominatimOuput {
                    ids: vec![3606679920],
                    area: GeocodeaArea::default(),
                })
            });
        nominatim
            .expect_search()
            .with(eq("Aomori, Japan"), eq("es"))
            .times(1)
            .returning(|_, _| {
                Ok(NominatimOuput {
                    ids: vec![3601834655],
                    area: GeocodeaArea::default(),
                })
            });

        let (processed, _areas) = preprocess_query(query, &Bbox::default(), 60, nominatim)
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

use std::collections::BTreeMap;

use geojson::*;
use serde::Deserialize;

pub fn osm_to_geojson(osm: Osm) -> FeatureCollection {
    let node_map = BTreeMap::from_iter(osm.elements.iter().filter_map(|n| {
        if let Element::Node(node) = n {
            Some((node.id, vec![node.lon, node.lat]))
        } else {
            None
        }
    }));

    let way_map = BTreeMap::from_iter(osm.elements.iter().flat_map(|n| {
        if let Element::Way(way) = n {
            way.nodes.iter().map(|node| (*node, way.id)).collect()
        } else {
            vec![]
        }
    }));

    let features = osm
        .elements
        .into_iter()
        .flat_map(|el| element_to_feature(&el, &node_map, &way_map))
        .collect();

    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
}

/// Convert an Osm element to a geojson feature
///
/// Returns an option since areas are not converted to geojson
fn element_to_feature(
    el: &Element,
    node_map: &BTreeMap<u64, Vec<f64>>,
    way_map: &BTreeMap<u64, u64>,
) -> Option<Feature> {
    let mut feat = Feature {
        id: Some(feature::Id::Number(el.id().into())),
        geometry: Some(element_to_geometry(el, node_map)?),
        ..Default::default()
    };

    // TODO probably change some lines to polygons
    // > an heuristic has to be applied to determine whether a way is a Line or a Polygon
    // https://wiki.openstreetmap.org/wiki/Overpass_turbo/Polygon_Features

    // NOTE: any properties that begin with `__` will not be displayed

    if let serde_json::Value::Object(mut obj) = el
        .tags()
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()))
    {
        obj.insert("osm_id".to_string(), el.id().into());
        obj.insert("osm_type".to_string(), el.osm_type().to_string().into());

        match &el {
            Element::Way(way) => {
                obj.insert("__children_ids".to_string(), way.nodes.clone().into());
            }
            Element::Node(node) => {
                if let Some(way_id) = way_map.get(&node.id) {
                    obj.insert("__way_id".to_string(), (*way_id).into());
                }
            }
            _ => (),
        }

        feat.properties = Some(obj);
    }

    Some(feat)
}

fn element_to_geometry(el: &Element, node_map: &BTreeMap<u64, Vec<f64>>) -> Option<Geometry> {
    Some(
        match el {
            Element::Node(node) => Value::Point(vec![node.lon, node.lat]),
            Element::Way(way) => Value::LineString(
                way.nodes
                    .iter()
                    .filter_map(|id| node_map.get(id).cloned())
                    .collect(),
            ),
            Element::Relation(rel) => Value::GeometryCollection(
                rel.members
                    .iter()
                    .flat_map(|el| element_to_geometry(el, node_map))
                    .collect(),
            ),
            Element::Area(_) => return None,
        }
        .into(),
    )
}

/// The intro stuff we don't care about, besides elements.
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Osm {
    // why is this a float? who knows lol !
    pub version: f32,
    pub generator: String,
    pub osm3s: serde_json::Value,
    pub elements: Vec<Element>,
}

/// A single point in space defined by its latitude, longitude and node id.
///
/// [OpenStreetMap wiki](https://wiki.openstreetmap.org/wiki/Node)
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Node {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub tags: Option<serde_json::Value>,
}

/// A way is an ordered list of nodes.
///
/// [OpenStreetMap wiki](https://wiki.openstreetmap.org/wiki/Way)
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Way {
    pub id: u64,
    pub nodes: Vec<u64>,
    pub tags: Option<serde_json::Value>,
}

/// A way is an ordered list of nodes.
///
/// [OpenStreetMap wiki](https://wiki.openstreetmap.org/wiki/Way)
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Relation {
    pub id: u64,
    pub tags: Option<serde_json::Value>,

    pub members: Vec<Element>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Area {
    pub id: u64,
    pub tags: Option<serde_json::Value>,
}

/// A generic element, either a node or way.
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Node(Node),
    Way(Way),
    Relation(Relation),
    Area(Area),
}

impl Element {
    fn tags(&self) -> Option<&serde_json::Value> {
        match self {
            Element::Node(n) => n.tags.as_ref(),
            Element::Way(n) => n.tags.as_ref(),
            Element::Relation(n) => n.tags.as_ref(),
            Element::Area(n) => n.tags.as_ref(),
        }
    }

    fn id(&self) -> u64 {
        match self {
            Element::Node(n) => n.id,
            Element::Way(n) => n.id,
            Element::Relation(n) => n.id,
            Element::Area(n) => n.id,
        }
    }

    fn osm_type(&self) -> &'static str {
        match self {
            Element::Node(_) => "node",
            Element::Way(_) => "way",
            Element::Relation(_) => "relation",
            Element::Area(_) => "area",
        }
    }
}

// TODO add tests for osm_to_geojson
// but idk it works well so far and i dont plan on changing it any time soon so im leaving this for later

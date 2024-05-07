use std::{path::PathBuf, sync::Arc, time::Duration};

use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use reqwest::StatusCode;
use scraper::{CaseSensitivity, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::{fs::read_to_string, time::sleep};

use crate::AppState;

fn all_keys_url(page: usize, per_page: usize) -> String {
    format!("https://taginfo.openstreetmap.org/api/4/keys/all?page={page}&rp={per_page}&sortname=count_all&sortorder=desc&filter=in_wiki")
}
fn all_values_url(key: &str, page: usize, per_page: usize) -> String {
    // filter=in_wiki doesnt work
    format!("https://taginfo.openstreetmap.org/api/4/key/values?key={key}&page={page}&rp={per_page}&sortname=count&sortorder=desc")
}

const PER_PAGE: usize = 999;
const SLEEP_TIME: u64 = 100;

pub async fn get_taginfo(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    read_to_string(state.taginfo_path()).await.unwrap()
}

pub async fn update_taginfo_route(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, TagInfoError> {
    update_taginfo(state.taginfo_path()).await
}

pub async fn update_taginfo(taginfo_path: PathBuf) -> Result<impl IntoResponse, TagInfoError> {
    println!("updating taginfo");
    let mut keys: Vec<Key> = vec![];
    let mut page = 1;
    loop {
        let res: TagInfoResponse<Key> = reqwest::get(all_keys_url(page, PER_PAGE))
            .await?
            .json()
            .await?;

        let len = res.data.len();

        keys.extend(res.data);

        if len < PER_PAGE {
            break;
        }
        println!("done with page {page}");
        page += 1;

        sleep(Duration::from_millis(SLEEP_TIME)).await;
    }

    let keys = keys.into_iter().filter(|k| k.in_wiki).collect::<Vec<_>>();

    let keys_len = keys.len();
    println!("found {} keys", keys.len());

    let mut vec = vec![];
    for (i, key) in keys.into_iter().enumerate() {
        println!("searching for {}, {i}/{keys_len}", key.key);

        // then for every key, get all it's values
        let res: TagInfoResponse<Value> = reqwest::get(all_values_url(&key.key, 1, PER_PAGE))
            .await?
            .json()
            .await?;

        let values: Vec<_> = res
            .data
            .into_iter()
            .filter(|v| v.description.as_deref().map(str::is_empty) == Some(false))
            .collect();

        println!("found {} values for {}", values.len(), key.key);

        let description = if key.in_wiki {
            let url = match key.key.as_str() {
                "highway" => "https://wiki.openstreetmap.org/wiki/Highways".to_string(),
                _ => format!("https://wiki.openstreetmap.org/wiki/Key:{}", key.key),
            };

            let page = reqwest::get(url).await?.text().await?;

            let fragment = Html::parse_fragment(&page);
            let selector = Selector::parse(".mw-parser-output > *").unwrap();

            let mut html = String::new();
            for el in fragment.select(&selector) {
                if el
                    .value()
                    .has_class("languages", CaseSensitivity::AsciiCaseInsensitive)
                    || el
                        .value()
                        .has_class("description", CaseSensitivity::AsciiCaseInsensitive)
                    || el.value().name() == "table"
                {
                    continue;
                }

                if el.attr("id") == Some("toc") {
                    break;
                }

                html.push_str(&el.text().collect::<String>());
            }
            Some(html)
        } else {
            None
        };

        vec.push(OutputKey {
            key,
            values,
            description,
        });
    }

    println!("done finding all keys");

    // store all of this as a json file we can serve
    let keys = serde_json::to_string(&vec)?;

    std::fs::write(taginfo_path, keys)?;

    Ok("done")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagInfoResponse<T> {
    total: usize,
    data_until: String,
    data: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Key {
    key: String,

    /// Is there at least one wiki page for this key?
    in_wiki: bool,

    /// Number of objects in the OSM database with this key.
    count_all: usize,
    /// Number of objects in relation to all objects.
    count_all_fraction: f32,

    /// Number of nodes in the OSM database with this key.
    count_nodes: usize,
    /// Number of nodes in relation to all tagged nodes.
    count_nodes_fraction: f32,
    /// Number of ways in the OSM database with this key.
    count_ways: usize,
    /// Number of ways in relation to all ways.
    count_ways_fraction: f32,
    /// Number of relations in the OSM database with this key.
    count_relations: usize,
    /// Number of relations in relation to all relations.
    count_relations_fraction: f32,

    /// Number of different values for this key.
    values_all: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    value: String,

    /// Description of the tag from the wiki.
    description: Option<String>,

    /// Number of times this key/value is in the OSM database.
    count: usize,
    /// Number of times in relation to number of times this key is in the OSM database.
    fraction: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputKey {
    #[serde(flatten)]
    key: Key,
    values: Vec<Value>,
    /// html
    description: Option<String>,
}

#[derive(Error, Debug)]
pub enum TagInfoError {
    #[error("{0:?}")]
    Network(#[from] reqwest::Error),
    #[error("{0:?}")]
    Serde(#[from] serde_json::Error),
    #[error("{0:?}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for TagInfoError {
    fn into_response(self) -> axum::response::Response {
        let json = json!({
            "error": format!("{self}"),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response()
    }
}

use crate::{app_state::AppState, search, taginfo::taginfo_path};

use std::sync::Arc;

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use tokio::fs::read_to_string;

pub fn make_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(home))
        .route("/index.css", get(css))
        .route("/index.js", get(js))
        .route("/taginfo.json", get(get_taginfo))
        .route("/search", post(search::search))
}

async fn home() -> Html<String> {
    // read file when on debug, embed file when on release
    // this way we can live edit locally, and we don't need to keep the files next to the executable in prod

    #[cfg(debug_assertions)]
    return Html(read_to_string("./frontend/index.html").await.unwrap());

    #[cfg(not(debug_assertions))]
    Html(
        const_format::str_replace!(
            include_str!("../public/index.html"),
            "GIT_HASH",
            env!("GIT_HASH")
        )
        .to_string(),
    )
}

async fn css() -> ([(&'static str, &'static str); 1], String) {
    #[cfg(debug_assertions)]
    let a = read_to_string("./public/index.css").await.unwrap();
    #[cfg(not(debug_assertions))]
    let a = include_str!("../public/index.css").to_string();

    ([("content-type", "text/css")], a)
}

async fn js() -> ([(&'static str, &'static str); 1], String) {
    #[cfg(debug_assertions)]
    let a = read_to_string("./public/index.js").await.unwrap();
    #[cfg(not(debug_assertions))]
    let a = include_str!("../public/index.js").to_string();

    ([("content-type", "text/js")], a)
}

async fn get_taginfo(State(state): State<Arc<AppState>>) -> String {
    let taginfo_path = taginfo_path(&state.data_path);

    if taginfo_path.exists() {
        read_to_string(taginfo_path).await.unwrap()
    } else {
        "[]".to_string()
    }
}

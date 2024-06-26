use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use backtrace::Backtrace;
use cache::Caches;
use elevation::ElevationMap;
use tokio::{fs::read_to_string, net::TcpListener};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod cache;
mod elevation;
mod graph;
mod nominatim;
mod osm_to_geojson;
mod preprocess;
mod search;
mod taginfo;

#[tokio::main]
async fn main() {
    // we only care if the error is a line parse
    if let Err(err @ dotenv::Error::LineParse(..)) = dotenv::dotenv() {
        panic!("{:?}", err);
    }

    let log_path = std::env::var("LOG_PATH")
        .expect("failed to get LOG_PATH")
        .into();
    let _guard1 = setup_tracing(log_path);

    let data_path: PathBuf = std::env::var("DATA_PATH")
        .expect("failed to get DATA")
        .into();

    let mut elevation_path = data_path.clone();
    elevation_path.push("elevation");
    let elevation_map =
        elevation::ElevationMap::new(&elevation_path).expect("failed to load elevation map");

    let state = AppState {
        elevation_map,
        data_path,
        caches: Caches::new(),
    };

    let taginfo_path = state.taginfo_path();
    if !taginfo_path.exists() {
        if let Err(err) = taginfo::update_taginfo(taginfo_path).await {
            panic!("{:?}", err);
        }
    }

    // build our application with a single route
    let app = Router::new()
        .route("/", get(home))
        .route("/index.css", get(css))
        .route("/index.js", get(js))
        .route("/taginfo.json", get(taginfo::get_taginfo))
        .route("/update-taginfo", get(taginfo::update_taginfo_route))
        .route("/search", post(search::search))
        .with_state(Arc::new(state));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or(3000);

    #[cfg(debug_assertions)]
    println!("listening on http://localhost:{port}");

    // run it with hyper on localhost:3000
    let listener = TcpListener::bind(SocketAddr::new([0, 0, 0, 0].into(), port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub struct AppState {
    elevation_map: ElevationMap,
    data_path: PathBuf,

    caches: Caches,
}

impl AppState {
    fn taginfo_path(&self) -> PathBuf {
        let mut taginfo_path = self.data_path.clone();
        taginfo_path.push("taginfo");
        taginfo_path.push("taginfo.json");
        taginfo_path
    }
}

async fn home() -> Html<String> {
    // read file when on debug, embed file when on release
    // this way we can live edit in local, and dont have to keep the files next to the executable in prod

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

fn setup_tracing(log_path: PathBuf) -> WorkerGuard {
    std::panic::set_hook(Box::new(|panic| {
        let b = Backtrace::new();
        if let Some(location) = panic.location() {
            tracing::error!(
                message = %panic,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
                backtrace = ?b,
            );
        } else {
            tracing::error!(message = %panic, backtrace = ?b);
        }
    }));

    let log_filter = EnvFilter::from_env("LOG_LEVEL");
    #[allow(unused_variables)]
    let error_filter = EnvFilter::from_env("ERROR_LEVEL");

    // normal logging
    let t = tracing_subscriber::registry().with(log_filter);

    // file
    let file_appender = tracing_appender::rolling::daily(log_path, "app.log");
    #[allow(unused_variables)]
    let (non_blocking, guard1) = tracing_appender::non_blocking(file_appender);
    #[cfg(not(debug_assertions))]
    let t = t.with(tracing_subscriber::fmt::layer().with_writer(non_blocking));
    // stdout
    #[cfg(debug_assertions)]
    let t = t.with(
        tracing_subscriber::fmt::layer().with_writer(std::io::stdout), // .with_filter(log_filter),
    );

    t.init();

    guard1
}

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use tokio::net::TcpListener;

use underpass::{app_state, elevation, routes, taginfo::taginfo_path, tracing};

#[tokio::main]
async fn main() {
    // we only care if the error is a line parse
    if let Err(err @ dotenv::Error::LineParse(..)) = dotenv::dotenv() {
        panic!("{:?}", err);
    }

    let log_path = std::env::var("LOG_PATH")
        .expect("failed to get LOG_PATH")
        .into();
    let _tracing_guard = tracing::setup_tracing(log_path);

    let data_path: PathBuf = std::env::var("DATA_PATH")
        .expect("failed to get DATA")
        .into();

    let mut elevation_path = data_path.clone();
    elevation_path.push("elevation");
    let elevation_map =
        elevation::ElevationMap::new(&elevation_path).expect("failed to load elevation map");

    let taginfo_path = taginfo_path(&data_path);
    if !taginfo_path.exists() {
        ::tracing::error!("{taginfo_path:?} not found");
    }

    let state = app_state::AppState::new(data_path, elevation_map);

    let app = routes::make_router().with_state(Arc::new(state));

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

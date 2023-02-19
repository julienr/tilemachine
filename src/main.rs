use std::env;

use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Router};
use gdal::Dataset;

fn setup_gdal() {
    env::set_var("VSI_CACHE", "TRUE");
    env::set_var("GDAL_DISABLE_READDIR_ON_OPEN", "TRUE");
    env::set_var("AWS_ACCESS_KEY_ID", "minioadmin");
    env::set_var("AWS_SECRET_ACCESS_KEY", "minioadmin");
    env::set_var("AWS_S3_ENDPOINT", "localhost:9000");
    env::set_var("AWS_VIRTUAL_HOSTING", "FALSE");
    env::set_var("AWS_HTTPS", "FALSE");
    // TODO: Enable for verbose debugging
    env::set_var("CPL_DEBUG", "1");
}

// raster_path can be a fullpath, in which case it needs to be urlencoded (%2F instead of /)
async fn get_tile(
    Path((raster_path, z, y, x)): Path<(String, u32, u32, u32)>,
) -> impl IntoResponse {
    println!(
        "get_tile raster_path={:?}, z={:?}, y={:?}, x={:?}",
        raster_path, z, y, x
    );
    let mut vsi_path = "/vsis3/".to_owned();
    vsi_path.push_str(raster_path.as_str());
    match Dataset::open(vsi_path.as_str()) {
        Ok(ds) => {
            println!("Opened raster of size={:?}", ds.raster_size());
            (StatusCode::OK, "ok").into_response()
        }
        Err(err) => {
            println!("Error opening {:?}: {:?}", raster_path, err);
            (
                StatusCode::NOT_FOUND,
                format!("Error opening {:?}", raster_path),
            )
                .into_response()
        }
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

// https://docs.rs/tokio/0.2.20/tokio/index.html#cpu-bound-tasks-and-blocking-code

#[tokio::main]
async fn main() {
    setup_gdal();
    let app = Router::new()
        .fallback(handler_404)
        .route("/", get(|| async { "Hello, World2!" }))
        .route("/tile/:raster_path/:z/:y/:x", get(get_tile));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

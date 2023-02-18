use std::env;

use axum::{extract::Path, routing::get, Router};
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

async fn get_tile(Path((z, y, x)): Path<(u32, u32, u32)>) {
    println!("get_tile z={:?}, y={:?}, x={:?}", z, y, x);
}

// https://docs.rs/tokio/0.2.20/tokio/index.html#cpu-bound-tasks-and-blocking-code

#[tokio::main]
async fn main() {
    setup_gdal();
    let r = Dataset::open("/vsis3/rasters/raster1.tif");
    if r.is_err() {
        println!("Error {:?}", r.err());
        panic!();
    }
    let ds = r.unwrap();
    println!("Opened raster of size={:?}", ds.raster_size());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World2!" }))
        .route("/tile/:z/:y/:x", get(get_tile));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

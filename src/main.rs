use std::env;
use std::path::Path;

use actix_files as fs;
use actix_web::{
    get, http::header::ContentType, middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};
use gdal::Dataset;
use tilemachine::xyz::TileCoords;
use std::collections::HashMap;

use tilemachine::wms;
use tilemachine::utils::Result;
use tilemachine::custom_script::CustomScript;

fn setup_gdal() {
    env::set_var("VSI_CACHE", "TRUE");
    env::set_var("GDAL_DISABLE_READDIR_ON_OPEN", "TRUE");
    env::set_var("AWS_ACCESS_KEY_ID", "minioadmin");
    env::set_var("AWS_SECRET_ACCESS_KEY", "minioadmin");
    env::set_var("AWS_S3_ENDPOINT", "localhost:9000");
    env::set_var("AWS_VIRTUAL_HOSTING", "FALSE");
    env::set_var("AWS_HTTPS", "FALSE");
    // TODO: Enable for verbose debugging
    env::set_var("CPL_DEBUG", "0");
}

// Opens the given raster and response by applying f on it
fn respond_with_raster<F>(raster_path: &String, f: F) -> HttpResponse
where
    F: Fn(&Dataset) -> HttpResponse,
{
    let mut vsi_path = "/vsis3/".to_owned();
    vsi_path.push_str(raster_path.as_str());
    match Dataset::open(vsi_path.as_str()) {
        Ok(ds) => f(&ds),
        Err(err) => {
            println!("Error opening {:?}, err={:?}", err, raster_path);
            HttpResponse::NotFound().body(format!("Error opening {:?}", raster_path))
        }
    }
}

fn respond_with_error<E: std::fmt::Debug>(message:&str, error: &E) -> HttpResponse {
    log::error!("{}: {:?}", message, error);
    HttpResponse::InternalServerError().body(message.to_string())
}

fn open_dataset_from_blobstore(raster_path: &str) -> Result<Dataset> {
    let mut vsi_path = "/vsis3/".to_owned();
    vsi_path.push_str(raster_path);
    Ok(Dataset::open(vsi_path.as_str())?)
}

#[get("/wms/{raster_path:.+}/service")]
async fn get_wms(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let raster_path = path.into_inner();
    let image_name = Path::new(&raster_path)
        .file_name()
        .map_or("image", |s| s.to_str().unwrap());
    // TODO: Parse query params
    println!("query_params: {:?}", query.get("SERVICE"));
    respond_with_raster(&raster_path, |ds| match wms::capabilities(image_name, ds) {
        Ok(xml) => HttpResponse::Ok()
            .content_type(ContentType::xml())
            .body(xml),
        Err(e) => {
            respond_with_error("Failed to generate capabilities", &e)
        }
    })
}

// raster_path can be a fullpath, in which case it needs to be urlencoded (%2F instead of /)
#[get("/tile/xyz/{custom_script:.+}/{z}/{y}/{x}")]
async fn get_xyz_tile(path: web::Path<(String, u64, u64, u64)>) -> HttpResponse {
    let (custom_script, z, y, x) = path.into_inner();
    let custom_script = match CustomScript::new_from_str(&custom_script) {
        Ok(script) => script,
        Err(e) => return respond_with_error("Failed to parse custom script", &e)
    };
    match custom_script.execute_on_tile(&TileCoords{x, y, zoom: z}, &open_dataset_from_blobstore) {
        Ok(image_data) => 
            HttpResponse::Ok()
                .content_type(ContentType::png())
                .body(image_data.to_png()),
        Err(e) => respond_with_error("Failed to extract tile", &e)
    }
}

#[get("/bounds/{custom_script:.+}")]
async fn get_bounds(path: web::Path<String>) -> HttpResponse {
    let custom_script = match CustomScript::new_from_str(&path.into_inner()) {
        Ok(script) => script,
        Err(e) => return respond_with_error("Failed to parse custom script", &e)
    };

    match custom_script.get_bounds(&open_dataset_from_blobstore) {
        Ok(bounds) => HttpResponse::Ok().json(bounds),
        Err(e) => respond_with_error("Failed to compute bounds", &e)
    }
}

async fn default_route(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().body(format!("Not found: {:?}", req.path()))
}

// https://docs.rs/tokio/0.2.20/tokio/index.html#cpu-bound-tasks-and-blocking-code

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    setup_gdal();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .service(get_wms)
            .service(get_xyz_tile)
            .service(get_bounds)
            .service(fs::Files::new("/", "./web").index_file("index.html"))
            .default_service(web::route().to(default_route))
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

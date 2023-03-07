use std::env;
use std::path::Path;

use actix_files as fs;
use actix_web::{
    get, http::header::ContentType, middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};
use gdal::Dataset;

mod bbox;
mod geojson;
mod raster;
mod wms;
mod xyz;

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

#[get("/wms/{raster_path:.+}/service")]
async fn get_wms(path: web::Path<String>) -> HttpResponse {
    let raster_path = path.into_inner();
    let image_name = Path::new(&raster_path)
        .file_name()
        .map_or("image", |s| s.to_str().unwrap());
    respond_with_raster(&raster_path, |ds| match wms::capabilities(image_name, ds) {
        Ok(xml) => HttpResponse::Ok()
            .content_type(ContentType::xml())
            .body(xml),
        Err(e) => {
            println!("Failed to generate capabilities: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to generate capabilities")
        }
    })
}

// raster_path can be a fullpath, in which case it needs to be urlencoded (%2F instead of /)
#[get("/tile/xyz/{raster_path:.+}/{z}/{y}/{x}")]
async fn get_xyz_tile(path: web::Path<(String, u64, u64, u64)>) -> HttpResponse {
    let (raster_path, z, y, x) = path.into_inner();
    respond_with_raster(&raster_path, |ds| {
        let pngdata = xyz::extract_tile(ds, x, y, z);
        HttpResponse::Ok()
            .content_type(ContentType::png())
            .body(pngdata)
    })
}

#[get("/bounds/{raster_path:.+}")]
async fn get_bounds(path: web::Path<String>) -> HttpResponse {
    let raster_path = path.into_inner();
    respond_with_raster(&raster_path, |ds| {
        // TODO: Remove unwrap
        let bounds = raster::bounds(ds).unwrap();
        HttpResponse::Ok().json(bounds)
    })
}

async fn default_route(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().body(format!("Not found: {:?}", req.path()))
}

// https://docs.rs/tokio/0.2.20/tokio/index.html#cpu-bound-tasks-and-blocking-code

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
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

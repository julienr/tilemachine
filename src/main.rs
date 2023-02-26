use std::env;

use actix_files as fs;
use actix_web::{
    get, http::header::ContentType, middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};
use gdal::Dataset;

mod bbox;
mod geojson;
mod raster;
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

// raster_path can be a fullpath, in which case it needs to be urlencoded (%2F instead of /)
#[get("/{raster_path}/{z}/{y}/{x}")]
async fn get_xyz_tile(path: web::Path<(String, u64, u64, u64)>) -> HttpResponse {
    let (raster_path, z, y, x) = path.into_inner();
    let mut vsi_path = "/vsis3/".to_owned();
    vsi_path.push_str(raster_path.as_str());
    match Dataset::open(vsi_path.as_str()) {
        Ok(ds) => {
            let pngdata = xyz::extract_tile(&ds, x, y, z);
            HttpResponse::Ok()
                .content_type(ContentType::png())
                .body(pngdata)
        }
        Err(err) => {
            println!("Error opening {:?}, err={:?}", err, raster_path);
            HttpResponse::NotFound().body(format!("Error opening {:?}", raster_path))
        }
    }
}

#[get("/{raster_path}")]
async fn get_bounds(path: web::Path<String>) -> HttpResponse {
    let raster_path = path.into_inner();
    let mut vsi_path = "/vsis3/".to_owned();
    vsi_path.push_str(raster_path.as_str());
    match Dataset::open(vsi_path.as_str()) {
        Ok(ds) => {
            // TODO: Remove unwrap
            let bounds = raster::bounds(&ds).unwrap();
            HttpResponse::Ok().json(bounds)
        }
        Err(err) => {
            println!("Error opening {:?}, err={:?}", err, raster_path);
            HttpResponse::NotFound().body(format!("Error opening {:?}", raster_path))
        }
    }
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
            .service(web::scope("/tile/xyz").service(get_xyz_tile))
            .service(web::scope("/bounds").service(get_bounds))
            .service(fs::Files::new("/", "./web").index_file("index.html"))
            .default_service(web::route().to(default_route))
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::ContentDisposition;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::StreamExt;
use lazy_static;
use nanoid;
use std::{fs, io::Write, path::PathBuf};
use url::Url;

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref LISTEN_AT: String = std::env::var("LISTEN_AT").unwrap();
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
    pub static ref API_GATEWAY_PUBLIC_URL: String = std::env::var("API_GATEWAY_PUBLIC_URL").unwrap();
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref PUBLIC_ROUTE: String = std::env::var("PUBLIC_ROUTE").unwrap();
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
    pub static ref PUBLIC_FOLDER: String = std::env::var("PUBLIC_FOLDER").unwrap();
}

async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut file_paths: Vec<String> = Vec::new();
    // iterate over multipart stream
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition: ContentDisposition =
            ContentDisposition::from(field.content_disposition().unwrap());
        let filename: &str = content_disposition.get_filename().unwrap(); // filename.fake.extension
        let splitted: Vec<&str> = filename.split('.').collect(); // [filename, extension]
        let file_extension: &str = splitted.last().unwrap(); // extension
        let uploaded_filename: String = format!("{}.{}", nanoid::simple(), file_extension);
        // Create url
        let file_url: String = format!(
            "{}{}{}/{}",
            API_GATEWAY_PUBLIC_URL.to_owned(),
            API_ROUTE.to_owned(),
            PUBLIC_ROUTE.to_owned(),
            uploaded_filename
        );

        // Local filepath
        let mut file_path: PathBuf = PUBLIC_FOLDER.parse::<PathBuf>()?;
        file_path.push(uploaded_filename);
        // File::create is blocking operation, use threadpool
        let mut file = web::block(|| std::fs::File::create(file_path))
            .await
            .unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            file = web::block(move || file.write_all(&data).map(|_| file)).await?;
        }
        file_paths.push(file_url);
    }
    Ok(HttpResponse::Ok().json(file_paths))
}

fn create_public_folder() {
    let absolute_path: PathBuf = PUBLIC_FOLDER.parse::<PathBuf>().unwrap();
    // Recursive won't fail if the folders already exist
    fs::DirBuilder::new()
        .recursive(true)
        .create(absolute_path)
        .unwrap();
}

fn init() {
    // Create the public folder
    create_public_folder();
    // Initialise logger
    env_logger::init();
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    init();
    let address: std::net::SocketAddrV4 = LISTEN_AT.parse().unwrap();

    HttpServer::new(|| {
        let public_folder: PathBuf = PUBLIC_FOLDER.parse::<PathBuf>().unwrap();
        App::new().wrap(middleware::Logger::default()).service(
            // Group routes by API_ROUTE
            web::scope(&API_ROUTE)
                // Image upload
                .service(
                    web::resource(&(UPLOAD_ROUTE.parse::<String>().unwrap()))
                        .route(web::post().to(upload)),
                )
                // Serve images from public folder
                .service(
                    actix_files::Files::new(
                        &(PUBLIC_ROUTE.parse::<String>().unwrap()),
                        public_folder,
                    )
                    .show_files_listing(),
                ),
        )
    })
    .bind(address)?
    .run()
    .await
}

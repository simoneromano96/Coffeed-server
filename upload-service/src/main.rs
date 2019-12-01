use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::ContentDisposition;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{
    future::{err, Either},
    Future, Stream,
};
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

fn save_file(field: Field) -> impl Future<Item = String, Error = Error> {
    let content_disposition: ContentDisposition =
        ContentDisposition::from(field.content_disposition().unwrap());
    let filename: &str = content_disposition.get_filename().unwrap(); // filename.fake.extension
    let splitted: Vec<&str> = filename.split('.').collect(); // [filename, extension]
    let file_extension: &str = splitted.last().unwrap(); // extension
    let uploaded_filename: String = format!("{}.{}", nanoid::simple(), file_extension);
    // Create url
    let url: String = format!(
        "{}{}{}/{}",
        API_GATEWAY_PUBLIC_URL.to_owned(),
        API_ROUTE.to_owned(),
        PUBLIC_ROUTE.to_owned(),
        uploaded_filename
    );
    let file_url: Url = Url::parse(&url).unwrap();

    // Local filepath
    let mut file_path: PathBuf = PUBLIC_FOLDER.parse::<PathBuf>().unwrap();
    file_path.push(uploaded_filename);
    let file = match fs::File::create(file_path) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
    };
    Either::B(
        field
            .fold(file, move |mut file: std::fs::File, bytes| {
                // fs operations are blocking, we have to execute writes
                // on threadpool
                web::block(move || {
                    file.write_all(bytes.as_ref())
                        .map_err(|e: std::io::Error| {
                            println!("file.write_all failed: {:?}", e);
                            error::PayloadError::Io(e)
                        })?;
                    Ok(file)
                })
                .map_err(|e: error::BlockingError<MultipartError>| match e {
                    error::BlockingError::Error(e) => e,
                    error::BlockingError::Canceled => MultipartError::Incomplete,
                })
            })
            .map(|_| file_url.into_string())
            .map_err(error::ErrorInternalServerError),
    )
}

fn upload(multipart: Multipart) -> impl Future<Item = HttpResponse, Error = Error> {
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| save_file(field).into_stream())
        .flatten()
        .collect()
        .map(|filepaths| HttpResponse::Ok().json(filepaths))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
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

fn main() -> std::io::Result<()> {
    init();
    let address: std::net::SocketAddrV4 = LISTEN_AT.parse().unwrap();

    HttpServer::new(|| {
        let public_folder: PathBuf = PUBLIC_FOLDER.parse::<PathBuf>().unwrap();
        App::new().wrap(middleware::Logger::default()).service(
            // Group routes by API_ROUTE
            web::scope(&API_ROUTE)
                // Image upload
                .service(web::resource(&UPLOAD_ROUTE).route(web::post().to_async(upload)))
                // Serve images from public folder
                .service(
                    actix_files::Files::new(&PUBLIC_ROUTE, public_folder).show_files_listing(),
                ),
        )
    })
    .bind(address)?
    .run()
}

use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::ContentDisposition;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::future::{err, Either};
use futures::{Future, Stream};
use nanoid;
use std::fs;
use std::io::Write;

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref PRIVATE_HOSTNAME: String = std::env::var("PRIVATE_HOSTNAME").unwrap();
    pub static ref PUBLIC_HOSTNAME: String = std::env::var("PUBLIC_HOSTNAME").unwrap();
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref STATIC_ROUTE: String = std::env::var("STATIC_ROUTE").unwrap();
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
    pub static ref PUBLIC_FOLDER: String = std::env::var("PUBLIC_FOLDER").unwrap();
}

pub fn save_file(field: Field) -> impl Future<Item = String, Error = Error> {
    let content_disposition: ContentDisposition = field.content_disposition().unwrap();
    let filename: &str = content_disposition.get_filename().unwrap(); // filename.fake.extension
    let splitted: Vec<&str> = filename.split('.').collect(); // [filename, extension]
    let file_extension: &str = splitted.last().unwrap(); // extension
    let uploaded_filename: String = format!("{}.{}", nanoid::simple(), file_extension);
    let url: String = format!(
        "{}/{}/{}",
        PUBLIC_HOSTNAME.to_owned(),
        STATIC_ROUTE.to_owned(),
        uploaded_filename
    );

    let file_path_string = format!("src/public/uploads/{}", uploaded_filename);
    let file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
    };
    Either::B(
        field
            .fold(file, move |mut file, bytes| {
                // fs operations are blocking, we have to execute writes
                // on threadpool
                web::block(move || {
                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        MultipartError::Payload(error::PayloadError::Io(e))
                    })?;
                    // acc += bytes.len() as i64;
                    Ok(file)
                })
                .map_err(|e: error::BlockingError<MultipartError>| match e {
                    error::BlockingError::Error(e) => e,
                    error::BlockingError::Canceled => MultipartError::Incomplete,
                })
            })
            .map(|_| url)
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn upload(multipart: Multipart) -> impl Future<Item = HttpResponse, Error = Error> {
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| save_file(field).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

fn create_public_folder() {
    let path: &str = &PUBLIC_FOLDER;
    // Recursive won't fail if the folders already exist
    fs::DirBuilder::new().recursive(true).create(path).unwrap();
}

fn init() {
    // Create the public folder
    create_public_folder();
    // Initialise logger
    env_logger::init();
}

fn main() -> std::io::Result<()> {
    init();
    let address: std::net::SocketAddrV4 = PRIVATE_HOSTNAME.parse().unwrap();

    HttpServer::new(|| {
        // Get public folder as String
        let public_folder = PUBLIC_FOLDER.parse::<String>().unwrap();
        App::new().wrap(middleware::Logger::default()).service(
            // Group routes by API_ROUTE
            web::scope(&API_ROUTE)
                // Image upload
                .service(web::resource(&UPLOAD_ROUTE).route(web::post().to_async(upload)))
                // Serve images from public folder
                .service(
                    actix_files::Files::new(&STATIC_ROUTE, public_folder).show_files_listing(),
                ),
        )
    })
    .bind(address)?
    .run()
}

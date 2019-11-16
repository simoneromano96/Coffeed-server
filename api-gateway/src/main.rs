// Modules
mod models;

// Crates
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::ContentDisposition;
use actix_web::http::Uri;
use actix_web::{error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use bytes::Bytes;
use env_logger;
use futures::{Future, Stream};
use reqwest;
use reqwest::multipart::Part;
use reqwest::Url;
use reqwest::{Client, Response};
use std::io::Read;
use std::sync::Arc;

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref LISTEN_AT: String = std::env::var("LISTEN_AT").unwrap();
    pub static ref PUBLIC_BASE_URL: String = std::env::var("PUBLIC_BASE_URL").unwrap();
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref PUBLIC_ROUTE: String = std::env::var("PUBLIC_ROUTE").unwrap();
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
}

fn create_bytes(field: Field) -> impl Future<Item = (Bytes, String), Error = Error> {
    let content_disposition: ContentDisposition = field.content_disposition().unwrap();
    // Get filename, ex: file.fake.extension
    let filename: String = String::from(content_disposition.get_filename().unwrap());
    // Get the bytes of the field into bytes
    let bytes: Bytes = Bytes::new();
    field
        .fold(bytes, move |mut last_chunk: Bytes, current_chunk: Bytes| {
            web::block(move || {
                last_chunk.extend(current_chunk);
                Ok(last_chunk)
            })
            .map_err(|e| match e {
                error::BlockingError::Error(e) => e,
                error::BlockingError::Canceled => MultipartError::Incomplete,
            })
        })
        .map(|bytes| (bytes, filename))
        .map_err(error::ErrorInternalServerError)
}

fn upload(
    multipart: Multipart,
    client: web::Data<Arc<reqwest::Client>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let arc_client = client;
    // For each multipart field
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| create_bytes(field).into_stream())
        .flatten()
        .collect()
        .map(|couples: Vec<(Bytes, String)>| {
            // Create url string
            let destination_address_string: String = format!(
                "{}{}{}",
                UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
                API_ROUTE.parse::<String>().unwrap(),
                UPLOAD_ROUTE.parse::<String>().unwrap(),
            );
            // Then Parse it into URL
            let destination_address: Url = destination_address_string.parse().unwrap();
            let mut request = reqwest::multipart::Form::new();

            for field in couples {
                let native_bytes: &[u8] = field.0.as_ref();
                let filename: String = field.1;
                let part: Part = Part::bytes(native_bytes.to_owned()).file_name(filename.clone());
                request = request.part(filename, part);
            }
            let client = arc_client;
            let mut response: Response = client
                .post(destination_address)
                .multipart(request)
                .send()
                .unwrap();
            let photos: Vec<String> = response.json().unwrap();
            let upload_response: models::UploadResponse = models::UploadResponse { data: photos };

            HttpResponse::Ok().json(upload_response)
        })
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

fn public_files(
    request: HttpRequest,
    client: web::Data<Arc<reqwest::Client>>,
) -> Result<HttpResponse, Error> {
    let arc_client = client;
    let full_uri: &Uri = request.uri();
    println!("{:?}", full_uri);
    // Path already includes /api
    let path = full_uri.path();
    println!("{:?}", path);
    // Create url string
    let destination_address_string: String = format!(
        "{}{}",
        UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
        path.parse::<String>().unwrap(),
    );
    println!("{:?}", destination_address_string);
    // Then Parse it into URL
    let destination_address: Url = destination_address_string.parse().unwrap();
    println!("{:?}", destination_address);

    let mut response: Response = arc_client.get(destination_address).send().unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    response
        .read_to_end(&mut buffer)
        .map(|_result| HttpResponse::Ok().body(buffer))
        .map_err(Error::from)
}

fn main() -> std::io::Result<()> {
    let address: std::net::SocketAddrV4 = LISTEN_AT.parse().unwrap();
    //TODO: Custom http client
    let client_builder = Client::builder();
    // client_builder.cookie_store(true);
    // client_builder.use_rustls_tls();
    let http_client = Arc::new(client_builder.build().unwrap());
    env_logger::init();

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(http_client.clone())
            .service(
                web::scope(&API_ROUTE)
                    .service(web::resource(&UPLOAD_ROUTE).route(web::post().to_async(upload)))
                    .service(web::resource(&PUBLIC_ROUTE).route(web::get().to(public_files))),
            )
    })
    .bind(address)?
    .run()
}

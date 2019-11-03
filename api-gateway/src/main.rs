use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::ContentDisposition;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use bytes::Bytes;
use env_logger;
use futures::{Future, Stream};
use reqwest;
use reqwest::multipart::Part;
use reqwest::{Client, Response};
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
    // Get (all?) bytes of the field
    let bytes: Bytes = Bytes::new();
    field
        .fold(bytes, move |bytes: Bytes, field_bytes: Bytes| {
            web::block(move || {
                let new_bytes = Bytes::from([bytes, field_bytes].concat());
                Ok(new_bytes)
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
    // For each multipart field
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| create_bytes(field).into_stream())
        .flatten()
        .collect()
        .map(|couples: Vec<(Bytes, String)>| {
            let destination_address: String = format!(
                "{}{}{}",
                UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
                API_ROUTE.parse::<String>().unwrap(),
                UPLOAD_ROUTE.parse::<String>().unwrap(),
            );
            let mut request = reqwest::multipart::Form::new();

            for field in couples {
                let native_bytes: &[u8] = field.0.as_ref();
                let filename: String = field.1.clone();
                let part: Part = Part::bytes(native_bytes.to_owned());
                request = request.part(filename, part);
            }
            let new_client = Client::new();
            let response: Response = new_client
                .post(&destination_address)
                .multipart(request)
                .send()
                .unwrap();
            println!("{:?}", response);

            HttpResponse::Ok().json("It works! <Evil laugh>")
        })
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

fn main() -> std::io::Result<()> {
    //std::env::set_var("RUST_LOG", "actix_http=trace");
    let address: std::net::SocketAddrV4 = LISTEN_AT.parse().unwrap();
    //TODO: Custom http client
    let http_client = Arc::new(Client::new());
    env_logger::init();

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(http_client.clone())
            .service(
                web::scope(&API_ROUTE)
                    .service(web::resource(&UPLOAD_ROUTE).route(web::post().to_async(upload))),
            )
    })
    .bind(address)?
    .run()
}

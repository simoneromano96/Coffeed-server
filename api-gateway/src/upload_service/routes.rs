// Crates
use crate::{models, AppState};
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{
    error, http::header::ContentDisposition, http::Uri, web, web::Bytes, Error, HttpRequest,
    HttpResponse,
};
use futures::{Future, Stream};
use reqwest::{self, multipart::Form, multipart::Part, Response, Url};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufReader, Write};
use std::{io::Read, sync::Arc};

// Evaluate env vars only once
lazy_static::lazy_static! {
    // Upload service
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
    pub static ref PUBLIC_ROUTE: String = std::env::var("PUBLIC_ROUTE").unwrap();
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
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

pub fn upload(
    multipart: Multipart,
    app_state: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let arc_client = app_state.http_client.clone();
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
                &UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
                &API_ROUTE.parse::<String>().unwrap(),
                &UPLOAD_ROUTE.parse::<String>().unwrap(),
            );
            // Then Parse it into URL
            let destination_address: Url = destination_address_string.parse::<Url>().unwrap();
            // Create form
            let mut form: Form = Form::new();
            // Add form data
            for (bytes, filename) in couples {
                let part = Part::bytes(bytes.to_vec()).file_name(filename.clone());
                form = form.part(filename.clone(), part);
            }

            let http_client = arc_client;

            web::block(move || {
                http_client
                    .post(destination_address)
                    .multipart(form)
                    .send()
                    .map(|res| Ok("test"))
                    .map_err(|e| Err(e.to_string()))
            })
            .map_err(|e| Error::from(e))
            .map(|r| HttpResponse::Ok().json("asd"))
        })
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
        .flatten()
}

pub fn public_files(
    request: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let http_client = &app_state.http_client;
    // let arc_client = client;
    let full_uri: &Uri = request.uri();
    // Path already includes /api
    let path = full_uri.path();
    // Create url string
    let destination_address_string: String = format!(
        "{}{}",
        &UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
        &path.parse::<String>().unwrap(),
    );
    // Then Parse it into URL
    let destination_address: Url = destination_address_string.parse::<Url>().unwrap();

    // let mut response: Response = arc_client.get(destination_address).send().unwrap();
    web::block(move || {
        http_client
            .get(destination_address)
            .send()
            .map(|res| Ok("test"))
            .map_err(|e| Err(e.to_string()))
    })
    .map_err(|e| Error::from(e))
    .map(|r| HttpResponse::Ok().json("asd"))
}

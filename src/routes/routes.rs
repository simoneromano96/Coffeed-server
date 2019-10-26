use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::ContentDisposition;
use actix_web::{error, web, Error, HttpResponse};
use futures::future::{err, Either};
use futures::{Future, Stream};
use nanoid;
use std::format;
use std::fs;
use std::io::Write;

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

pub fn save_file(field: Field) -> impl Future<Item = String, Error = Error> {
    let content_disposition: ContentDisposition = field.content_disposition().unwrap();
    let filename: &str = content_disposition.get_filename().unwrap(); // filename.fake.extension
    let splitted: Vec<&str> = filename.split('.').collect(); // [filename, extension]
    let file_extension: &str = splitted.last().unwrap(); // extension
    let uploaded_filename: String = format!("{}.{}", nanoid::simple(), file_extension);
    let server_address = std::env::var("ADDRESS").unwrap();
    let server_port = std::env::var("PORT").unwrap_or_else(|_| "80".to_string());
    let url: String = format!(
        "{}:{}/{}/{}",
        server_address, server_port, "/public/uploads", uploaded_filename
    );

    let file_path_string = format!("src/public/uploads/{}", uploaded_filename);
    let file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
    };
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, acc), bytes| {
                // fs operations are blocking, we have to execute writes
                // on threadpool
                web::block(move || {
                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        MultipartError::Payload(error::PayloadError::Io(e))
                    })?;
                    // acc += bytes.len() as i64;
                    Ok((file, acc))
                })
                .map_err(|e: error::BlockingError<MultipartError>| match e {
                    error::BlockingError::Error(e) => e,
                    error::BlockingError::Canceled => MultipartError::Incomplete,
                })
            })
            .map(|(_, acc)| url)
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

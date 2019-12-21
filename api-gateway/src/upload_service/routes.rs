use crate::{forward_to, AppState};
use actix_web::{http::Uri, web, Error, HttpRequest, HttpResponse};

// Evaluate env vars only once
lazy_static::lazy_static! {
    // Upload service
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
    pub static ref PUBLIC_ROUTE: String = std::env::var("PUBLIC_ROUTE").unwrap();
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
}

pub async fn upload(
    app_state: web::Data<AppState>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get client
    let client = app_state.http_client.clone();

    // Create url string
    let destination_address: String = format!(
        "{}{}{}",
        &UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &UPLOAD_ROUTE.parse::<String>().unwrap(),
    );

    forward_to(destination_address, client, body, req).await
}

pub async fn public_files(
    app_state: web::Data<AppState>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Path already includes /api
    let full_uri: &Uri = req.uri();
    let path = full_uri.path();
    // Create url string
    let destination_address: String = format!(
        "{}{}",
        &UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
        &path.parse::<String>().unwrap(),
    );

    forward_to(destination_address, client, body, req).await
}

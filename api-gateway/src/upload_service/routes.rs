use crate::{forward_to, AppState};
use actix_web::{http::Uri, web, web::Bytes, Error, HttpRequest, HttpResponse};
use futures::{Future, Stream};

// Evaluate env vars only once
lazy_static::lazy_static! {
    // Upload service
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
    pub static ref PUBLIC_ROUTE: String = std::env::var("PUBLIC_ROUTE").unwrap();
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
}

pub fn upload(
    app_state: web::Data<AppState>,
    body: web::Payload,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Get client
    let client = app_state.http_client.clone();

    // Create url string
    let destination_address: String = format!(
        "{}{}{}",
        &UPLOAD_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &UPLOAD_ROUTE.parse::<String>().unwrap(),
    );

    // forward_to(destination_address, client, body, req)

    // Create a new request
    let forwarded_req = client
        .request_from(destination_address, req.head())
        .no_decompress();
    // Add headers
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req
            .header("x-forwarded-for", format!("{}", addr.ip()))
            .header("forwarded", format!("for={}", addr.ip()))
    } else {
        forwarded_req
    };

    forwarded_req
        .send_stream(body)
        .map_err(Error::from)
        .map(|mut res| {
            let mut client_resp = HttpResponse::build(res.status());
            // Remove `Connection` as per
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
            for (header_name, header_value) in res.headers().iter() {
                client_resp.header(header_name.clone(), header_value.clone());
            }
            res.body()
                .into_stream()
                .concat2()
                .map(move |b| client_resp.body(b))
                .map_err(|e| e.into())
        })
        .flatten()
}

pub fn public_files(
    app_state: web::Data<AppState>,
    body: web::Bytes,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
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
    forward_to(destination_address, client, body, req)
}

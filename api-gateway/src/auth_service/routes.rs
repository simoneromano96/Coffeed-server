// Crates
use crate::AppState;
use actix_session::Session;
use actix_web::client as awc;
use actix_web::http::{HeaderMap, HeaderName, HeaderValue};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use futures::{Future, Stream};
use serde_derive::{Deserialize, Serialize};
use std::env;

// Evaluate env vars only once
lazy_static::lazy_static! {
    // Commons
    pub static ref API_ROUTE: String = env::var("API_ROUTE").unwrap();
    // Auth service
    pub static ref AUTH_SERVICE_PUBLIC_URL: String = env::var("AUTH_SERVICE_PUBLIC_URL").unwrap();
    pub static ref AUTH_SERVICE_URL: String = env::var("AUTH_SERVICE_URL").unwrap();
    pub static ref LOGIN_ROUTE: String = env::var("LOGIN_ROUTE").unwrap();
    pub static ref LOGOUT_ROUTE: String = env::var("LOGOUT_ROUTE").unwrap();
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IndexResponse {
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SessionInfo {
    user_id: String,
    user_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    email: String,
    password: String,
}

fn forward_to(
    destination_address: String,
    client: awc::Client,
    body: web::Bytes,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
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
        .send_body(body)
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

pub fn login(
    app_state: web::Data<AppState>,
    // login_info: web::Json<LoginInfo>,
    body: web::Bytes,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Create url string
    let destination_address: String = format!(
        "{}{}{}",
        &AUTH_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &LOGIN_ROUTE.parse::<String>().unwrap(),
    );
    forward_to(destination_address, client, body, req)
}

pub fn logout(
    app_state: web::Data<AppState>,
    body: web::Bytes,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Create url string
    let destination_address: String = format!(
        "{}{}{}",
        &AUTH_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &LOGOUT_ROUTE.parse::<String>().unwrap(),
    );
    forward_to(destination_address, client, body, req)
}

pub fn get_session(session: Session) -> impl Responder {
    let user_id = session.get::<String>("user_id").unwrap();
    let user_type = session.get::<String>("user_type").unwrap();

    if user_id.is_some() && user_type.is_some() {
        HttpResponse::Ok().json(SessionInfo {
            user_id: user_id.unwrap(),
            user_type: user_type.unwrap(),
        })
    } else {
        HttpResponse::Forbidden().json("Please authenticate")
    }
}

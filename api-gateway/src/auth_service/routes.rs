// Crates
use crate::{forward_to, AppState};
use actix_session::Session;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use futures::Future;
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

// Crates
use crate::{forward_to, AppState};
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
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

pub async fn login(
    app_state: web::Data<AppState>,
    // login_info: web::Json<LoginInfo>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Create url string
    let destination_address: String = format!(
        "{}{}{}",
        &AUTH_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &LOGIN_ROUTE.parse::<String>().unwrap(),
    );

    forward_to(destination_address, client, body, req).await
}

pub async fn logout(
    app_state: web::Data<AppState>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Create url string
    let destination_address: String = format!(
        "{}{}{}",
        &AUTH_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &LOGOUT_ROUTE.parse::<String>().unwrap(),
    );

    forward_to(destination_address, client, body, req).await
}

pub async fn get_session(session: Session) -> Result<HttpResponse, Error> {
    let user_id = session.get::<String>("user_id")?;
    let user_type = session.get::<String>("user_type")?;

    if user_id.is_some() && user_type.is_some() {
        HttpResponse::Ok()
            .json(SessionInfo {
                user_id: user_id.unwrap(),
                user_type: user_type.unwrap(),
            })
            .await
    } else {
        Err(error::ErrorForbidden("Please authenticate"))
    }
}

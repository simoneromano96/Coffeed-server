// Crates
use crate::AppState;
use actix_session::Session;
use actix_web::{cookie::CookieJar, error, web, Error, HttpRequest, HttpResponse, Responder};
use futures::Future;
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue, FORWARDED},
    Url,
};
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
    counter: i32,
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
    login_info: web::Json<LoginInfo>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Create url string
    let destination_address_string: String = format!(
        "{}{}{}",
        &AUTH_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &LOGIN_ROUTE.parse::<String>().unwrap(),
    );
    // Then Parse it into URL
    let destination_address: Url = destination_address_string.parse::<Url>().unwrap();

    // Get request ip
    let from_address = req.head().peer_addr.unwrap();

    // Create headers
    let mut header_map: HeaderMap = HeaderMap::new();
    // Set forwarded header
    header_map.append(
        FORWARDED,
        HeaderValue::from_str(&from_address.to_string()).unwrap(),
    );

    web::block(move || {
        let result: Result<reqwest::Response, reqwest::Error> = client
            .post(destination_address)
            .headers(header_map)
            .json(&(login_info.into_inner()))
            .send();
        match result {
            Ok(mut response) => {
                let mut cookie_jar: CookieJar = CookieJar::new();
                let cookies = response.cookies();
                cookies.for_each(|cookie| {
                    let actix_cookie = actix_web::cookie::Cookie::new(
                        cookie.name().to_owned(),
                        cookie.value().to_owned(),
                    );
                    cookie_jar.add(actix_cookie);
                });
                Ok((response.json::<IndexResponse>().unwrap(), cookie_jar))
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .map(|(data, cookies)| {
        let mut response_builder = HttpResponse::Ok();
        cookies.iter().for_each(|cookie| {
            response_builder.cookie(cookie.to_owned());
        });
        response_builder.json(data)
    })
    .map_err(error::ErrorInternalServerError)
}

pub fn logout(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Get client
    let client = app_state.http_client.clone();
    // Create url string
    let destination_address_string: String = format!(
        "{}{}{}",
        &AUTH_SERVICE_URL.parse::<String>().unwrap(),
        &API_ROUTE.parse::<String>().unwrap(),
        &LOGOUT_ROUTE.parse::<String>().unwrap(),
    );
    // Then Parse it into URL
    let destination_address: Url = destination_address_string.parse::<Url>().unwrap();

    // Get request ip
    let from_address = req.head().peer_addr.unwrap();

    // Create headers
    let mut header_map: HeaderMap = HeaderMap::new();
    // Set forwarded header
    header_map.append(
        FORWARDED,
        HeaderValue::from_str(&from_address.to_string()).unwrap(),
    );

    web::block(move || {
        let result: Result<reqwest::Response, reqwest::Error> =
            client.post(destination_address).headers(header_map).send();

        match result {
            Ok(mut response) => {
                let mut cookie_jar = CookieJar::new();
                let cookies = response.cookies();
                cookies.for_each(|cookie| {
                    let actix_cookie = actix_web::cookie::Cookie::new(
                        cookie.name().to_owned(),
                        cookie.value().to_owned(),
                    );
                    cookie_jar.add(actix_cookie);
                });

                Ok((response.text().unwrap(), cookie_jar))
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .map(|(data, cookies)| {
        let mut response_builder = HttpResponse::Ok();
        cookies.iter().for_each(|cookie| {
            response_builder.cookie(cookie.to_owned());
        });
        response_builder.json(data)
    })
    .map_err(error::ErrorInternalServerError)
}

pub fn get_session(session: Session) -> impl Responder {
    let user_id = session.get::<String>("user_id").unwrap().unwrap();
    let user_type = session.get::<String>("user_type").unwrap().unwrap();

    HttpResponse::Ok().json(SessionInfo { user_id, user_type })
}

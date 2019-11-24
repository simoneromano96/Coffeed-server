//! Example of login and logout using redis-based sessions
//!
//! Every request gets a session, corresponding to a cache entry and cookie.
//! At login, the session key changes and session state in cache re-assigns.
//! At logout, session state in cache is removed and cookie is invalidated.
// Modules
mod graphql;

// Crates
use actix_identity::IdentityService;
use actix_redis::RedisSession;
use actix_session::Session;
use actix_web::{
    middleware,
    middleware::Compress,
    web,
    web::{get, post, resource, scope},
    App, HttpResponse, HttpServer, Result,
};
use serde::{Deserialize, Serialize};
use std::{io, net::SocketAddrV4};

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref LISTEN_AT: String = std::env::var("LISTEN_AT").unwrap();
    pub static ref AUTH_SERVICE_PUBLIC_URL: String = std::env::var("AUTH_SERVICE_PUBLIC_URL").unwrap();
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref LOGIN_ROUTE: String = std::env::var("LOGIN_ROUTE").unwrap();
    pub static ref LOGOUT_ROUTE: String = std::env::var("LOGOUT_ROUTE").unwrap();
    pub static ref REDIS_HOST: String = std::env::var("REDIS_HOST").unwrap();
    pub static ref REDIS_PORT: String = std::env::var("REDIS_PORT").unwrap();
    pub static ref SESSION_SECRET: String = std::env::var("SESSION_SECRET").unwrap();
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IndexResponse {
    user_id: Option<String>,
    counter: i32,
}

#[derive(Deserialize)]
struct Identity {
    user_id: String,
}

/*
fn index(session: Session) -> Result<HttpResponse> {
    let user_id: Option<String> = session.get::<String>("user_id").unwrap();
    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(IndexResponse { user_id, counter }))
}

fn increment(session: Session) -> Result<HttpResponse> {
    let user_id: Option<String> = session.get::<String>("user_id").unwrap();
    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .map_or(1, |inner| inner + 1);
    session.set("counter", counter)?;

    Ok(HttpResponse::Ok().json(IndexResponse { user_id, counter }))
}
*/

// fn signup() {}

fn login(user_id: web::Json<Identity>, session: Session) -> Result<HttpResponse> {
    let id = user_id.into_inner().user_id;
    session.set("user_id", &id)?;
    session.renew();

    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(IndexResponse {
        user_id: Some(id),
        counter,
    }))
}

fn logout(session: Session) -> Result<HttpResponse> {
    let id: Option<String> = session.get("user_id")?;
    if let Some(x) = id {
        session.purge();
        Ok(format!("Logged out: {}", x).into())
    } else {
        Ok("Could not log out anonymous user".into())
    }
}

fn init() -> (SocketAddrV4, String, Vec<u8>) {
    // Create a socket address from listen_at
    let address: SocketAddrV4 = LISTEN_AT.parse::<SocketAddrV4>().unwrap();
    // Session
    let redis_host: String = format!(
        "{}:{}",
        REDIS_HOST.parse::<String>().unwrap(),
        REDIS_PORT.parse::<String>().unwrap()
    );
    let session_secret: Vec<u8> = SESSION_SECRET.parse::<String>().unwrap().into_bytes();
    // Logger utility
    env_logger::init();

    (address, redis_host, session_secret)
}

fn main() -> io::Result<()> {
    let (address, redis_host, session_secret) = init();
    //.wrap(RedisSession::new(redis_host.clone(), &session_secret))

    HttpServer::new(move || {
        App::new()
            .wrap(RedisSession::new(redis_host.clone(), &session_secret))
            .wrap(Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                scope(&API_ROUTE)
                    .service(resource(&LOGIN_ROUTE).route(post().to(login)))
                    .service(resource(&LOGOUT_ROUTE).route(post().to(logout))),
            )
    })
    .bind(address)?
    .run()
}

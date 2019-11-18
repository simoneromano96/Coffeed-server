// Modules
pub mod auth_service;
pub mod models;
pub mod upload_service;

// Crates
use actix_session::Session;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::{middleware, web, App, HttpServer};
use env_logger;
use reqwest::{self, Client, ClientBuilder};
use serde_derive::{Deserialize, Serialize};
use std::{env, io, net::SocketAddrV4, sync::Arc};

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref LISTEN_AT: String = env::var("LISTEN_AT").unwrap();
    pub static ref API_ROUTE: String = env::var("API_ROUTE").unwrap();
    // Gateway
    pub static ref API_GATEWAY_PUBLIC_URL: String = env::var("API_GATEWAY_PUBLIC_URL").unwrap();
    // Upload service
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
    pub static ref UPLOAD_ROUTE: String = env::var("UPLOAD_ROUTE").unwrap();
    pub static ref PUBLIC_ROUTE: String = env::var("PUBLIC_ROUTE").unwrap();
    // Auth service
    pub static ref AUTH_SERVICE_PUBLIC_URL: String = env::var("AUTH_SERVICE_PUBLIC_URL").unwrap();
    pub static ref AUTH_SERVICE_URL: String = env::var("AUTH_SERVICE_URL").unwrap();
    pub static ref LOGIN_ROUTE: String = env::var("LOGIN_ROUTE").unwrap();
    pub static ref LOGOUT_ROUTE: String = env::var("LOGOUT_ROUTE").unwrap();
    // Redis
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

fn login(req: HttpRequest, session: Session, client: web::Data<Arc<reqwest::Client>>) -> HttpResponse {
    let id = String::from("123");
    session.set("user_id", &id).unwrap();
    session.renew();

    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    HttpResponse::Ok().json(IndexResponse {
        user_id: Some(id),
        counter,
    })
}

fn logout(session: Session) -> HttpResponse {
    let id: Option<String> = session.get("user_id").unwrap();
    let message: String;
    if let Some(x) = id {
        session.purge();
        
        message = format!("Logged out: {}", x);
    } else {
        message = String::from("Could not log out anonymous user");
    }
    HttpResponse::Ok().json(message)
}

fn main() -> io::Result<()> {
    let address: SocketAddrV4 = LISTEN_AT.parse::<SocketAddrV4>().unwrap();
    //TODO: Custom http client
    let client_builder: ClientBuilder = Client::builder();
    // client_builder.cookie_store(true);
    // client_builder.use_rustls_tls();
    let http_client = Arc::new(client_builder.build().unwrap());
    env_logger::init();
    // Add a global listening to /public*
    let public_route: String = format!("{}*", PUBLIC_ROUTE.parse::<String>().unwrap());

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(http_client.clone())
            .service(
                web::scope(&API_ROUTE)
                    .service(
                        web::resource(&UPLOAD_ROUTE)
                            .route(web::post().to_async(upload_service::upload)),
                    )
                    .service(
                        web::resource(&public_route)
                            .route(web::get().to(upload_service::public_files)),
                    )
                    .service(
                        web::resource(&LOGIN_ROUTE).route(web::get().to(login))
                    ),
            )
    })
    .bind(address)?
    .run()
}

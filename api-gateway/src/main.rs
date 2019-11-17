// Modules
pub mod models;
pub mod upload_service;
pub mod auth_service;

// Crates
use actix_web::{middleware, web, App, HttpServer};
use env_logger;
use reqwest;
use reqwest::Client;
use std::sync::Arc;

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref LISTEN_AT: String = std::env::var("LISTEN_AT").unwrap();
    pub static ref API_GATEWAY_PUBLIC_URL: String = std::env::var("API_GATEWAY_PUBLIC_URL").unwrap();
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    // Upload service
    pub static ref UPLOAD_ROUTE: String = std::env::var("UPLOAD_ROUTE").unwrap();
    pub static ref PUBLIC_ROUTE: String = std::env::var("PUBLIC_ROUTE").unwrap();
    // Auth service
}

fn main() -> std::io::Result<()> {
    let address: std::net::SocketAddrV4 = LISTEN_AT.parse().unwrap();
    //TODO: Custom http client
    let client_builder = Client::builder();
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
                    ),
            )
    })
    .bind(address)?
    .run()
}

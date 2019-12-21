// Modules
pub mod auth_service;
pub mod models;
pub mod upload_service;

// Crates
use actix_redis::RedisSession;
use actix_web::{client as awc, Error, HttpRequest, HttpResponse};
use actix_web::{middleware, web, App, HttpServer};
use core::time::Duration;
use env_logger;
use std::{env, io, net::SocketAddrV4};

// Evaluate env vars only once
lazy_static::lazy_static! {
    // Actix conf
    pub static ref LISTEN_AT: String = env::var("LISTEN_AT").unwrap();
    // Commons
    pub static ref API_ROUTE: String = env::var("API_ROUTE").unwrap();
    // Gateway
    pub static ref API_GATEWAY_PUBLIC_URL: String = env::var("API_GATEWAY_PUBLIC_URL").unwrap();
    // Upload service
    pub static ref UPLOAD_SERVICE_URL: String = std::env::var("UPLOAD_SERVICE_URL").unwrap();
    pub static ref UPLOAD_ROUTE: String = env::var("UPLOAD_ROUTE").unwrap();
    pub static ref PUBLIC_ROUTE: String = env::var("PUBLIC_ROUTE").unwrap();
    // Auth service
    pub static ref AUTH_SERVICE_URL: String = env::var("AUTH_SERVICE_URL").unwrap();
    pub static ref LOGIN_ROUTE: String = env::var("LOGIN_ROUTE").unwrap();
    pub static ref LOGOUT_ROUTE: String = env::var("LOGOUT_ROUTE").unwrap();
    // Session
    pub static ref REDIS_HOST: String = std::env::var("REDIS_HOST").unwrap();
    pub static ref REDIS_PORT: String = std::env::var("REDIS_PORT").unwrap();
    pub static ref SESSION_SECRET: String = std::env::var("SESSION_SECRET").unwrap();
    pub static ref SESSION_COOKIE_NAME: String = std::env::var("SESSION_COOKIE_NAME").unwrap();
}

pub struct AppState {
    http_client: awc::Client,
}

pub async fn forward_to(
    destination_address: String,
    client: awc::Client,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
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

    // Get response
    let mut res = forwarded_req.send_stream(body).await.map_err(Error::from)?;

    // Create response
    let mut client_resp = HttpResponse::build(res.status());

    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    let body = res.body().limit(usize::max_value()).await?;

    Ok(client_resp.body(body).await?)
}

fn init() -> (SocketAddrV4, String, String, Vec<u8>) {
    // Create a socket address from listen_at
    let address: SocketAddrV4 = LISTEN_AT.parse().unwrap();
    // Add a global listener to /public*
    let public_route: String = format!("{}*", PUBLIC_ROUTE.parse::<String>().unwrap());
    // Session
    let redis_host: String = format!(
        "{}:{}",
        &REDIS_HOST.parse::<String>().unwrap(),
        &REDIS_PORT.parse::<String>().unwrap()
    );
    let session_secret: Vec<u8> = SESSION_SECRET.parse::<String>().unwrap().into_bytes();
    // Logger utility
    env_logger::init();

    (
        address,
        public_route,
        redis_host,
        session_secret,
        // init_client(),
    )
}

fn init_actix_client() -> awc::Client {
    // Client builder
    let client_builder: awc::ClientBuilder = awc::ClientBuilder::default();
    // Params
    // Set timeout to 3 minutes
    let timeout: Duration = Duration::from_secs(180);
    // End params

    client_builder.timeout(timeout).finish()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let (address, public_route, redis_host, session_secret) = init();

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(AppState {
                http_client: init_actix_client(),
            })
            .wrap(
                RedisSession::new(redis_host.clone(), &session_secret)
                    // .cookie_name("session-cookie")
                    .cookie_name(&SESSION_COOKIE_NAME)
                    .cookie_secure(false)
                    .cookie_path("/api"),
            )
            .wrap(middleware::Logger::default())
            .service(
                web::scope(&(API_ROUTE.parse::<String>().unwrap()))
                    // Upload service
                    .service(
                        web::resource(&(UPLOAD_ROUTE.parse::<String>().unwrap()))
                            .route(web::post().to(upload_service::upload)),
                    )
                    .service(
                        web::resource(&public_route)
                            .route(web::get().to(upload_service::public_files)),
                    )
                    // (only for testing purposes)
                    .service(
                        web::resource("get_session")
                            .route(web::get().to(auth_service::get_session)),
                    )
                    .service(
                        web::resource(&(LOGIN_ROUTE.parse::<String>().unwrap()))
                            .route(web::post().to(auth_service::login)),
                    )
                    .service(
                        web::resource(&(LOGOUT_ROUTE.parse::<String>().unwrap()))
                            .route(web::post().to(auth_service::logout)),
                    ),
            )
    })
    .bind(address)?
    .start()
    .await
}

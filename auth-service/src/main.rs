//! Example of login and logout using redis-based sessions
//!
//! Every request gets a session, corresponding to a cache entry and cookie.
//! At login, the session key changes and session state in cache re-assigns.
//! At logout, session state in cache is removed and cookie is invalidated.
//!
use actix_redis::RedisSession;
use actix_session::Session;
use actix_web::client::SendRequestError;
use actix_web::web::HttpRequest;
use actix_web::{
    client::Client,
    middleware, web,
    web::{get, post, resource},
    App, HttpResponse, HttpServer, Result,
};
use futures::future::Future;
use pretty_env_logger;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IndexResponse {
    user_id: Option<String>,
    counter: i32,
}

fn index(session: Session) -> Result<HttpResponse> {
    let user_id: Option<String> = session.get::<String>("user_id").unwrap();
    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(IndexResponse { user_id, counter }))
}

fn do_something(session: Session) -> Result<HttpResponse> {
    let user_id: Option<String> = session.get::<String>("user_id").unwrap();
    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .map_or(1, |inner| inner + 1);
    session.set("counter", counter)?;

    Ok(HttpResponse::Ok().json(IndexResponse { user_id, counter }))
}

#[derive(Deserialize)]
struct Identity {
    user_id: String,
}
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

fn test_api(
    request: HttpRequest,
    session: Session,
) -> impl Future<Item = HttpResponse, Error = SendRequestError> {
    let client = Client::build()
        .bearer_auth("coffeed__auth_service__secret_id")
        .finish();
    client
        .get("http://127.0.0.1:3005")
        .send()
        .and_then(|result| Ok("Yolo".into()))
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,actix_redis=info");
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            // redis session middleware
            .wrap(RedisSession::new("http://127.0.0.1:6379", &[0; 32]))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // .service(resource("/").route(get().to(index)))
            // .service(resource("/do_something").route(post().to(do_something)))
            .service(resource("/login").route(post().to(login)))
            .service(resource("/logout").route(post().to(logout)))
            .service(resource("/*").route(post().to_async(test_api)))
    })
    .bind("127.0.0.1:8080")?
    .run()
}

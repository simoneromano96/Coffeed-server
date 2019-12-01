//! Example of login and logout using redis-based sessions
//!
//! Every request gets a session, corresponding to a cache entry and cookie.
//! At login, the session key changes and session state in cache re-assigns.
//! At logout, session state in cache is removed and cookie is invalidated.
// Modules
// mod graphql;

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
use mongodb::{
    bson, coll::options::IndexOptions, coll::Collection, db::ThreadedDatabase, doc, oid::ObjectId,
    Client, ThreadedClient,
};
use serde::{Deserialize, Serialize};
use std::{io, net::SocketAddrV4};

// Evaluate env vars only once
lazy_static::lazy_static! {
    pub static ref LISTEN_AT: String = std::env::var("LISTEN_AT").unwrap();
    pub static ref AUTH_SERVICE_PUBLIC_URL: String = std::env::var("AUTH_SERVICE_PUBLIC_URL").unwrap();
    // Routes
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref LOGIN_ROUTE: String = std::env::var("LOGIN_ROUTE").unwrap();
    pub static ref LOGOUT_ROUTE: String = std::env::var("LOGOUT_ROUTE").unwrap();
    // Session
    pub static ref REDIS_HOST: String = std::env::var("REDIS_HOST").unwrap();
    pub static ref REDIS_PORT: String = std::env::var("REDIS_PORT").unwrap();
    pub static ref SESSION_SECRET: String = std::env::var("SESSION_SECRET").unwrap();
    // Mongodb
    pub static ref MONGODB_HOST: String = std::env::var("MONGODB_HOST").unwrap();
    pub static ref MONGODB_PORT: String = std::env::var("MONGODB_PORT").unwrap();
    pub static ref MONGODB_AUTH_DB: String = std::env::var("MONGODB_AUTH_DB").unwrap();
    pub static ref MONGODB_AUTH_USERNAME: String = std::env::var("MONGODB_AUTH_USERNAME").unwrap();
    pub static ref MONGODB_AUTH_PASSWORD: String = std::env::var("MONGODB_AUTH_PASSWORD").unwrap();
}

pub struct AppState {
    client: Client,
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

#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id")]
    id: ObjectId,
    username: String,
    email: String,
    password: String,
    #[serde(rename = "userType")]
    user_type: String,
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

fn login(session: Session, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let client = app_state.client.clone();

    // Get the db and collection
    let collection: Collection = client.db("authService").collection("users");

    // Fake up data from client
    let email = "admin@mail.com";
    let password = "password";

    // Find user
    let result_document = collection
        .find_one(Some(doc! { "email":  email, "password": password }), None)
        .unwrap()
        .unwrap();

    // Deserialize the document into a User instance
    let result: User = bson::from_bson(bson::Bson::Document(result_document)).unwrap();

    let id = result.id.to_hex();
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

fn create_db_client(
    host: String,
    port: u16,
    auth_db: String,
    auth_username: String,
    auth_password: String,
) -> Client {
    let client = Client::connect(&host, port).unwrap();
    // Authenticate
    client
        .db(&auth_db)
        .auth(&auth_username, &auth_password)
        .unwrap();

    client
}

fn init_db(client: Client) {
    let client = client.clone();
    // Create indexes
    // UserTypes
    let mut collection: Collection = client.db("authService").collection("users");
    let mut name_index: IndexOptions = IndexOptions::new();
    name_index.unique = Some(true);
    collection
        .create_index(doc! {"name": 1}, Some(name_index))
        .unwrap();
    // Users
    collection = client.db("authService").collection("users");
    let mut email_index: IndexOptions = IndexOptions::new();
    email_index.unique = Some(true);
    let mut username_index: IndexOptions = IndexOptions::new();
    username_index.unique = Some(true);
    collection
        .create_index(doc! {"email": 1}, Some(email_index))
        .unwrap();
    collection
        .create_index(doc! {"username": 1}, Some(username_index))
        .unwrap();
    if collection.count(None, None).unwrap() == 0 {
        let admin = User {
            id: ObjectId::new().unwrap(),
            username: String::from("admin"),
            email: String::from("admin@mail.com"),
            password: String::from("password"),
            user_type: String::from("Admin"),
        };
        let bson = bson::to_bson(&admin).unwrap();
        if let bson::Bson::Document(document) = bson {
            collection.insert_one(document, None).unwrap();
        }
    }
}

fn init() -> (SocketAddrV4, String, Vec<u8>, Client) {
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
    // Connection pool
    let client = create_db_client(
        MONGODB_HOST.parse().unwrap(),
        MONGODB_PORT.parse().unwrap(),
        MONGODB_AUTH_DB.parse().unwrap(),
        MONGODB_AUTH_USERNAME.parse().unwrap(),
        MONGODB_AUTH_PASSWORD.parse().unwrap(),
    );
    // Initialise DB
    init_db(client.clone());

    (address, redis_host, session_secret, client)
}

fn main() -> io::Result<()> {
    let (address, redis_host, session_secret, client) = init();

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                client: client.clone(),
            })
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

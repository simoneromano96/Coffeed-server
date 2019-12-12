//! Example of login and logout using redis-based sessions
//!
//! Every request gets a session, corresponding to a cache entry and cookie.
//! At login, the session key changes and session state in cache re-assigns.
//! At logout, session state in cache is removed and cookie is invalidated.
// Modules
// mod graphql;

// Crates
use actix_redis::RedisSession;
use actix_session::Session;
use actix_web::{
    middleware,
    middleware::Compress,
    web,
    web::{post, resource, scope},
    App, HttpResponse, HttpServer, Result,
};
use argonautica::{Hasher, Verifier};
use mongodb::{
    bson, coll::options::IndexOptions, coll::Collection, db::ThreadedDatabase, doc, Client,
    ThreadedClient,
};
use nanoid;
use serde::{Deserialize, Serialize};
use std::{io, net::SocketAddrV4};

// Evaluate env vars only once
lazy_static::lazy_static! {
    // Actix conf
    pub static ref LISTEN_AT: String = std::env::var("LISTEN_AT").unwrap();
    pub static ref AUTH_SERVICE_PUBLIC_URL: String = std::env::var("AUTH_SERVICE_PUBLIC_URL").unwrap();
    // Routes
    pub static ref API_ROUTE: String = std::env::var("API_ROUTE").unwrap();
    pub static ref LOGIN_ROUTE: String = std::env::var("LOGIN_ROUTE").unwrap();
    pub static ref LOGOUT_ROUTE: String = std::env::var("LOGOUT_ROUTE").unwrap();
    pub static ref SIGNUP_ROUTE: String = std::env::var("SIGNUP_ROUTE").unwrap();
    // Session
    pub static ref REDIS_HOST: String = std::env::var("REDIS_HOST").unwrap();
    pub static ref REDIS_PORT: String = std::env::var("REDIS_PORT").unwrap();
    pub static ref SESSION_SECRET: String = std::env::var("SESSION_SECRET").unwrap();
    pub static ref SESSION_COOKIE_NAME: String = std::env::var("SESSION_COOKIE_NAME").unwrap();
    // Mongodb
    pub static ref MONGODB_HOST: String = std::env::var("MONGODB_HOST").unwrap();
    pub static ref MONGODB_PORT: String = std::env::var("MONGODB_PORT").unwrap();
    pub static ref MONGODB_AUTH_DB: String = std::env::var("MONGODB_AUTH_DB").unwrap();
    pub static ref MONGODB_AUTH_USERNAME: String = std::env::var("MONGODB_AUTH_USERNAME").unwrap();
    pub static ref MONGODB_AUTH_PASSWORD: String = std::env::var("MONGODB_AUTH_PASSWORD").unwrap();
    // NanoID
    pub static ref NANOID_LENGTH: String = std::env::var("NANOID_LENGTH").unwrap();
    // Argon hashing key
    pub static ref ARGON2_HASH_SECRET_KEY: String = std::env::var("ARGON2_HASH_SECRET_KEY").unwrap();
}

pub struct AppState {
    client: Client,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IndexResponse {
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id")]
    id: String,
    username: String,
    email: String,
    password: String,
    #[serde(rename = "userType")]
    user_type: String,
}
#[derive(Serialize, Deserialize)]
struct UserType {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    grants: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct LoginInfo {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct SignupInfo {
    username: String,
    email: String,
    password: String,
    password_confirmation: String,
}

fn hash_password(password: String) -> String {
    let mut hasher = Hasher::default();
    hasher
        .with_password(password)
        .with_secret_key(ARGON2_HASH_SECRET_KEY.parse::<String>().unwrap())
        .hash()
        .unwrap()
}

fn verify_password(password_hash: String, password: String) -> bool {
    let mut verifier = Verifier::default();
    verifier
        .with_hash(password_hash)
        .with_password(password)
        .with_secret_key(ARGON2_HASH_SECRET_KEY.parse::<String>().unwrap())
        .verify()
        .unwrap()
}

fn new_id() -> String {
    nanoid::generate(NANOID_LENGTH.parse::<usize>().unwrap())
}

fn login(
    session: Session,
    app_state: web::Data<AppState>,
    login_info: web::Json<LoginInfo>,
) -> Result<HttpResponse> {
    let client = app_state.client.clone();

    // Get the db and collection
    let collection: Collection = client.db("authService").collection("users");

    let email = &login_info.email;

    // Find user
    let result_document = collection
        .find_one(Some(doc! { "email":  email }), None)
        .unwrap()
        .unwrap();

    // Deserialize the document into a User instance
    let result: User = bson::from_bson(bson::Bson::Document(result_document)).unwrap();

    if verify_password(result.password, login_info.password.clone()) {
        let (id, user_type) = (result.id, result.user_type);
        session.set("user_id", &id)?;
        session.set("user_type", &user_type)?;
        session.renew();

        Ok(HttpResponse::Ok().json(IndexResponse { user_id: Some(id) }))
    } else {
        Ok(HttpResponse::BadRequest().json("User not found or wrong password"))
    }
}

fn signup(
    app_state: web::Data<AppState>,
    signup_info: web::Json<SignupInfo>,
) -> Result<HttpResponse> {
    let client = app_state.client.clone();

    let mut result: Result<HttpResponse> = Ok(HttpResponse::BadRequest().json("Unknown error"));

    if signup_info.password != signup_info.password_confirmation {
        result = Ok(HttpResponse::BadRequest().json("Passwords don't match"));
    }

    let collection: Collection = client.db("authService").collection("users");

    let password: String = hash_password(signup_info.password.clone());

    let id = new_id();
    let user: User = User {
        id: id.clone(),
        username: signup_info.username.clone(),
        email: signup_info.email.clone(),
        password,
        user_type: "".to_string(), // This will be resolved after creation
    };

    // Create user
    let bson = bson::to_bson(&user).unwrap();
    if let bson::Bson::Document(document) = bson {
        let insert_result = collection.insert_one(document, None);
        match insert_result {
            Ok(_result) => {
                result = Ok(HttpResponse::Ok().json(id));
            }
            Err(err) => result = Ok(HttpResponse::BadRequest().json(err.to_string())),
        }
    }

    result
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
    // Create indexes
    // UserTypes
    let mut collection: Collection = client.db("authService").collection("userTypes");
    let mut name_index: IndexOptions = IndexOptions::new();
    name_index.unique = Some(true);
    collection
        .create_index(doc! {"name": 1}, Some(name_index))
        .unwrap();
    if collection.count(None, None).unwrap() == 0 {
        let admin_type = UserType {
            id: new_id(),
            name: String::from("Admin"),
            grants: vec![
                String::from("create"),
                String::from("update"),
                String::from("delete"),
            ],
        };
        let bson = bson::to_bson(&admin_type).unwrap();
        if let bson::Bson::Document(document) = bson {
            collection.insert_one(document, None).unwrap();
        }
    }
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
        let password = hash_password(String::from("password"));

        let user_types = client.db("authService").collection("userTypes");
        let user_type_doc = user_types
            .find_one(Some(doc! {"name": "Admin" }), None)
            .unwrap()
            .unwrap();
        let user_type: UserType = bson::from_bson(bson::Bson::Document(user_type_doc)).unwrap();

        let admin = User {
            id: new_id(),
            username: String::from("admin"),
            email: String::from("admin@mail.com"),
            password,
            user_type: user_type.id,
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
            .wrap(
                RedisSession::new(redis_host.clone(), &session_secret)
                    // .cookie_name("session-cookie")
                    .cookie_name(&SESSION_COOKIE_NAME)
                    .cookie_secure(false)
                    .cookie_path("/api"),
            )
            .wrap(Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                scope(&API_ROUTE)
                    .service(resource(&LOGIN_ROUTE).route(post().to(login)))
                    .service(resource(&LOGOUT_ROUTE).route(post().to(logout)))
                    .service(resource(&SIGNUP_ROUTE).route(post().to(signup))),
            )
    })
    .bind(address)?
    .run()
}

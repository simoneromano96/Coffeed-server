pub mod routes;
pub mod schema;
pub mod utils;
// pub mod models;

use crate::schema::User;
use crate::utils::PasswordHasher;
use actix_cors::Cors;
use actix_files;
use actix_web::{middleware, App, HttpServer};
use mongodb::{
    bson, coll::options::IndexOptions, coll::Collection, db::ThreadedDatabase, doc, oid::ObjectId,
    Client, ThreadedClient,
};
use pretty_env_logger;
use std::net::SocketAddr;

// pub type MongoPool = r2d2::Pool<MongodbConnectionManager>;
// pub type MongoConnection = r2d2::PooledConnection<MongodbConnectionManager>;

fn create_db_client() -> Client {
    let client = Client::connect("167.86.100.118", 27017).expect("Failed to initialize client.");
    // Authenticate
    client
        .db("admin")
        .auth("username", "password")
        .expect("Could not authenticate.");

    client
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("ADDRESS", "127.0.0.1");
    std::env::set_var("PORT", "8082");

    pretty_env_logger::init();

    let port: u16 = 8082;
    let addr: SocketAddr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let db_client = create_db_client();
    let mut password_hasher = PasswordHasher::build(String::from("secret key"));
    // Create indexes
    // Coffees
    let mut collection: Collection = db_client.db("coffeed").collection("coffees");
    let mut name_index: IndexOptions = IndexOptions::new();
    name_index.unique = Some(true);
    collection
        .create_index(doc! {"name": 1}, Some(name_index))
        .expect("Could not create index");
    // Users
    collection = db_client.db("coffeed").collection("users");
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
            password: password_hasher.hash(String::from("password")).unwrap(),
            user_type: String::from("Admin"),
        };
        let bson = bson::to_bson(&admin).unwrap();
        if let bson::Bson::Document(document) = bson {
            collection.insert_one(document, None).unwrap();
        }
    }

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::new())
            .wrap(middleware::Logger::default())
            // Save db_client in Server's state
            .data(db_client.clone())
            .configure(schema::register)
            // Serve images
            .service(actix_files::Files::new("/public", "src/public").show_files_listing())
    })
    .bind(addr)
    .unwrap()
    .run()
    .unwrap();
}

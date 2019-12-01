pub mod schema;
pub mod utils;
use crate::schema::User;
use crate::utils::utils::hash;
use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use mongodb::{
    bson, coll::options::IndexOptions, coll::Collection, db::ThreadedDatabase, doc, oid::ObjectId,
    Client, ThreadedClient,
};
use std::net::SocketAddr;

// pub type MongoPool = r2d2::Pool<MongodbConnectionManager>;
// pub type MongoConnection = r2d2::PooledConnection<MongodbConnectionManager>;

fn create_db_client(
    host: String,
    port: u16,
    auth_db: String,
    auth_username: String,
    auth_password: String,
) -> Client {
    let client = Client::connect(&host, port).expect("Failed to initialize client.");
    // Authenticate
    client
        .db(&auth_db)
        .auth(&auth_username, &auth_password)
        .expect("Could not authenticate.");

    client
}

fn init_db(db_client: Client) {
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
            password: hash(String::from("password")).unwrap(),
            user_type: String::from("Admin"),
        };
        let bson = bson::to_bson(&admin).unwrap();
        if let bson::Bson::Document(document) = bson {
            collection.insert_one(document, None).unwrap();
        }
    }
}

fn main() {
    // TODO: Env file with these values
    // std::env::set_var("RUST_LOG", "actix_web=info,actix_redis=info");
    std::env::set_var("RUST_LOG", "actix_web=info");
    // This server public address
    std::env::set_var("ACTIX_ADDRESS", "127.0.0.1");
    std::env::set_var("ACTIX_PORT", "8082");
    // Argon Hash Key
    std::env::set_var("HASH_SECRET_KEY", "secret_key");
    // MongoDB
    std::env::set_var("MONGODB_HOST", "167.86.100.118");
    std::env::set_var("MONGODB_PORT", "27017");
    std::env::set_var("MONGODB_AUTH_DB", "admin");
    std::env::set_var("MONGODB_AUTH_USERNAME", "username");
    std::env::set_var("MONGODB_AUTH_PASSWORD", "password");
    // JWT
    std::env::set_var("JWT_SECRET_KEY", "secret_key_2");
    std::env::set_var("JWT_ISSUER", "coffeed_inc");
    let expiry_time = 24 * 60 * 60 * 1000; // 1 Day in milliseconds
    std::env::set_var("JWT_EXPIRY", expiry_time.to_string());
    // Redis Sessions
    // std::env::set_var("REDIS_HOST", "167.86.100.118");
    // std::env::set_var("REDIS_PORT", "6379");

    pretty_env_logger::init();

    // Get actix info from env
    let actix_address = std::env::var("ACTIX_ADDRESS").unwrap();
    let actix_port = std::env::var("ACTIX_PORT").unwrap().parse::<u16>().unwrap();

    let address: SocketAddr = (format!("{}:{}", actix_address, actix_port))
        .parse::<SocketAddr>()
        .unwrap();

    // Get DB info from env
    let mongodb_host = std::env::var("MONGODB_HOST").unwrap();
    let mongodb_port = std::env::var("MONGODB_PORT")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let mongodb_auth_db = std::env::var("MONGODB_AUTH_DB").unwrap();
    let mongodb_auth_username = std::env::var("MONGODB_AUTH_USERNAME").unwrap();
    let mongodb_auth_password = std::env::var("MONGODB_AUTH_PASSWORD").unwrap();

    let db_client = create_db_client(
        mongodb_host,
        mongodb_port,
        mongodb_auth_db,
        mongodb_auth_username,
        mongodb_auth_password,
    );

    init_db(db_client.clone());

    // let redis_host = std::env::var("REDIS_HOST").unwrap();
    // let redis_port = std::env::var("REDIS_PORT").unwrap();
    // let redis_uri = format!("{}:{}", redis_host, redis_port);

    // Start http server
    HttpServer::new(move || {
        App::new()
            // CORS
            .wrap(
                Cors::new()
                //    .allowed_origin("ALL")
                //    .allowed_methods(vec!["GET", "POST"])
                //    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                //    .allowed_header(header::CONTENT_TYPE)
                //    .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            // Save db_client in Server's state
            .data(db_client.clone())
            .configure(schema::register)
    })
    .bind(address)
    .unwrap()
    .run()
    .unwrap();
}

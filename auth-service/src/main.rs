// Modules
// mod graphql;
mod migrations;

// Crates
use actix_redis::RedisSession;
use actix_session::Session;
use actix_web::{
    middleware,
    middleware::Compress,
    web,
    web::{post, resource, scope},
    App, Error, HttpResponse, HttpServer, Result,
};
use argonautica::{Hasher, Verifier};
use mysql::OptsBuilder;
use nanoid;
use r2d2::Pool;
use r2d2_mysql::MysqlConnectionManager;
use refinery::Runner;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;

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
    // MySQL
    pub static ref MYSQL_HOST: String = std::env::var("MYSQL_HOST").unwrap();
    pub static ref MYSQL_PORT: String = std::env::var("MYSQL_PORT").unwrap();
    pub static ref MYSQL_DATABASE: String = std::env::var("MYSQL_DATABASE").unwrap();
    pub static ref MYSQL_AUTH_USERNAME: String = std::env::var("MYSQL_AUTH_USERNAME").unwrap();
    pub static ref MYSQL_AUTH_PASSWORD: String = std::env::var("MYSQL_AUTH_PASSWORD").unwrap();
    // NanoID
    pub static ref NANOID_LENGTH: String = std::env::var("NANOID_LENGTH").unwrap();
    // Argon hashing key
    pub static ref ARGON2_HASH_SECRET_KEY: String = std::env::var("ARGON2_HASH_SECRET_KEY").unwrap();
}

pub type MySQLPool = Pool<MysqlConnectionManager>;

pub struct AppState {
    client: MySQLPool,
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

async fn login(
    session: Session,
    app_state: web::Data<AppState>,
    login_info: web::Json<LoginInfo>,
) -> Result<HttpResponse, Error> {
    let client = app_state.client.clone();

    let mut result: Result<HttpResponse> = Ok(HttpResponse::BadRequest().json("Unknown error"));
    // Get the db and collection
    /*
    let collection: Collection = client.database("authService").collection("users");

    let email = &login_info.email;

    // Query the documents in the collection with a filter and an option.
    let filter = doc! { "email": email };
    // Find user
    let result_document = collection.find_one(Some(filter), None).unwrap().unwrap();

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
    */

    result
}

async fn signup(
    app_state: web::Data<AppState>,
    signup_info: web::Json<SignupInfo>,
) -> Result<HttpResponse, Error> {
    let client = app_state.client.clone();

    let mut result: Result<HttpResponse> = Ok(HttpResponse::BadRequest().json("Unknown error"));

    if signup_info.password != signup_info.password_confirmation {
        result = Ok(HttpResponse::BadRequest().json("Passwords don't match"));
    }

    result
    /*
    let collection: Collection = client.database("authService").collection("users");

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
    */
}

async fn logout(session: Session) -> Result<HttpResponse, Error> {
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
    main_database: String,
    auth_username: String,
    auth_password: String,
) -> MySQLPool {
    let mut builder = OptsBuilder::new();

    builder
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .db_name(Some(main_database))
        .user(Some(auth_username))
        .pass(Some(auth_password));

    let manager = MysqlConnectionManager::new(builder);
    r2d2::Pool::builder().build(manager).unwrap()
}

fn init_db(
    host: String,
    port: u16,
    main_database: String,
    auth_username: String,
    auth_password: String,
) {
    let mut builder = mysql::OptsBuilder::new();

    builder
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .db_name(Some(main_database))
        .user(Some(auth_username))
        .pass(Some(auth_password));

    let mut connection = mysql::Conn::new(builder).unwrap();
    migrations::migrations::runner()
        .run(&mut connection)
        .unwrap();

    let user_types_count_query = r#"
        select count(*) as count
        from user_types
    "#;
    match connection.prep_exec(user_types_count_query, ()) {
        Ok(result) => result.map(|row_result| match row_result {
            Ok(row) => {
                let (count) = mysql::from_row(row);
            }
            Err(err) => {
                println!("{}", error.to_string());
            }
        }),
        Err(error) => {
            println!("{}", error.to_string());
        }
    };
    // migrations::runner().run(&mut connection).unwrap();
    // Create indexes
    // UserTypes
    /*
    let mut collection: Collection = client.database("authService").collection("userTypes");
    let name_index: IndexModel = IndexModel::builder()
        .keys(doc! {"name": 1})
        .options(Some(doc! {"unique": true}))
        .build();

    collection.create_indexes(vec![name_index]).unwrap();

    if collection.count_documents(None, None).unwrap() == 0 {
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
    collection = client.database("authService").collection("users");

    let email_index: IndexModel = IndexModel::builder()
        .keys(doc! {"email": 1})
        .options(Some(doc! {"unique": true}))
        .build();

    let username_index: IndexModel = IndexModel::builder()
        .keys(doc! {"username": 1})
        .options(Some(doc! {"unique": true}))
        .build();

    collection
        .create_indexes(vec![email_index, username_index])
        .unwrap();

    if collection.count_documents(None, None).unwrap() == 0 {
        let password = hash_password(String::from("password"));

        let user_types = client.database("authService").collection("userTypes");
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
    */
}

fn init() -> (SocketAddrV4, String, Vec<u8>, MySQLPool) {
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
        MYSQL_HOST.parse().unwrap(),
        MYSQL_PORT.parse().unwrap(),
        MYSQL_DATABASE.parse().unwrap(),
        MYSQL_AUTH_USERNAME.parse().unwrap(),
        MYSQL_AUTH_PASSWORD.parse().unwrap(),
    );
    // Initialise DB
    init_db(
        MYSQL_HOST.parse().unwrap(),
        MYSQL_PORT.parse().unwrap(),
        MYSQL_DATABASE.parse().unwrap(),
        MYSQL_AUTH_USERNAME.parse().unwrap(),
        MYSQL_AUTH_PASSWORD.parse().unwrap(),
    );

    (address, redis_host, session_secret, client)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let (address, redis_host, session_secret, client) = init();

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                client: client.clone(),
            })
            .wrap(
                RedisSession::new(redis_host.clone(), &session_secret)
                    .cookie_name(&SESSION_COOKIE_NAME)
                    .cookie_secure(false)
                    .cookie_path("/api"),
            )
            .wrap(Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                scope(&API_ROUTE)
                    .service(
                        resource(&(LOGIN_ROUTE.parse::<String>().unwrap())).route(post().to(login)),
                    )
                    .service(
                        resource(&(LOGOUT_ROUTE.parse::<String>().unwrap()))
                            .route(post().to(logout)),
                    )
                    .service(
                        resource(&(SIGNUP_ROUTE.parse::<String>().unwrap()))
                            .route(post().to(signup)),
                    ),
            )
    })
    .bind(address)?
    .run()
    .await
}

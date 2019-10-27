use argonautica::{Hasher, Verifier};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde_derive::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("HASH_SECRET_KEY").unwrap();
    pub static ref JWT_SECRET: String = std::env::var("JWT_SECRET_KEY").unwrap();
    pub static ref JWT_ISSUER: String = std::env::var("JWT_ISSUER").unwrap();
    pub static ref JWT_EXPIRY: i64 = std::env::var("JWT_EXPIRY").unwrap().parse::<i64>().unwrap();
}

/// Hash password
pub fn hash(password: String) -> Result<String, String> {
    match Hasher::default()
        .with_secret_key(SECRET_KEY.as_str())
        .with_password(password)
        .hash()
    {
        Ok(hashed) => Ok(hashed),
        Err(error) => Err(error.to_string()),
    }
}
/// Verify hashed password
pub fn verify(hash: String, password: String) -> Result<bool, String> {
    match Verifier::default()
        .with_password(password)
        .with_hash(hash)
        .with_secret_key(SECRET_KEY.as_str())
        .verify()
    {
        Ok(result) => Ok(result),
        Err(error) => Err(error.to_string()),
    }
}

// JWT claims
#[derive(Serialize, Deserialize)]
struct Claims {
    pub iss: String,
    pub sub: String,
    pub company: String,
    pub exp: i64,
}

pub struct SlimUser {
    pub username: String,
    pub email: String,
}

pub fn create_token(username: String, email: String) -> String {
    let headers = Header::default();
    let claims = Claims {
        iss: JWT_ISSUER.to_string(),
        sub: username,
        company: email,
        exp: (chrono::Local::now() + chrono::Duration::milliseconds(*JWT_EXPIRY)).timestamp(),
    };

    encode(&headers, &claims, JWT_SECRET.clone().as_bytes()).unwrap()
}

pub fn decode_token(token: &str) -> SlimUser {
    let data =
        decode::<Claims>(token, JWT_SECRET.clone().as_bytes(), &Validation::default()).unwrap();

    SlimUser {
        username: data.claims.sub,
        email: data.claims.company,
    }
}

use argonautica::{Hasher, Verifier};
use jsonwebtoken::{encode, Header};
use serde_derive::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("HASH_SECRET_KEY").unwrap();
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
    pub exp: i32,
}

pub fn create_token(username: String) -> String {
    let jwt_secret = std::env::var("JWT_SECRET_KEY").unwrap();
    let jwt_issuer = std::env::var("JWT_ISSUER").unwrap();
    let jwt_expiry = std::env::var("JWT_EXPIRY").unwrap().parse::<i32>().unwrap();
    let headers = Header::default();
    let claims = Claims {
        iss: jwt_issuer,
        sub: username,
        exp: jwt_expiry,
    };

    encode(&headers, &claims, jwt_secret.clone().as_bytes()).unwrap()
}

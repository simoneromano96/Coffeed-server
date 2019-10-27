use argonautica::{Hasher, Verifier};
use jsonwebtoken::{encode, Header};
use serde_derive::{Deserialize, Serialize};

pub struct PasswordHasher {
    hasher: Hasher<'static>,
    verifier: Verifier<'static>,
}

impl PasswordHasher {
    /// Build hasher and verifier wrapper
    pub fn build() -> PasswordHasher {
        // Get the secret key
        let secret_key = std::env::var("HASH_SECRET_KEY").unwrap();
        // Hasher
        let mut hasher = Hasher::default();
        hasher.with_secret_key(secret_key.clone());
        // Verifier
        let mut verifier = Verifier::default();
        verifier.with_secret_key(secret_key.clone());

        PasswordHasher { hasher, verifier }
    }
    /// Hash password
    pub fn hash(&mut self, password: String) -> Result<String, String> {
        match self.hasher.with_password(password).hash() {
            Ok(hashed) => Ok(hashed),
            Err(error) => Err(error.to_string()),
        }
    }
    /// Verify hashed password
    pub fn verify(&mut self, hash: String, password: String) -> Result<bool, String> {
        match self
            .verifier
            .with_password(password)
            .with_hash(hash)
            .verify()
        {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

// JWT
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

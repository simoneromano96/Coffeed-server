use argonautica::{Hasher, Verifier};

pub struct PasswordHasher {
    hasher: Hasher<'static>,
    verifier: Verifier<'static>,
}

impl PasswordHasher {
    /// Build hasher and verifier wrapper
    pub fn build(secret_key: String) -> PasswordHasher {
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

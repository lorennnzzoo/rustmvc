use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::Error, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
    pub exp: usize, // Unix timestamp
}

pub struct AuthConfig {
    pub secret: String,
}

impl AuthConfig {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    pub fn generate_token(&self, sub: &str, roles: Vec<String>, expires_in_secs: i64) -> String {
        let exp = Utc::now().timestamp() + expires_in_secs;
        let claims = Claims {
            sub: sub.to_string(),
            roles,
            exp: exp as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .unwrap()
    }

    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>, Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )
    }
}

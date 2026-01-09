use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode,
    encode,
    Algorithm,
    DecodingKey,
    EncodingKey,
    Header,
    Validation
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ForumResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims{
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_token(
    user_id: &Uuid,
    secret: &[u8],
    expires_in_seconds: i64,
) -> ForumResult<String> {
    
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id.to_string(),
        iat,
        exp,
    };

    encode(
        &Header::default(), 
        &claims, 
        &EncodingKey::from_secret(secret)
    ).map_err(|e| e.into())
}

pub fn decode_token<T: Into<String>>(
    token: T,
    secret: &[u8]
) -> ForumResult<String> {
    let decode = decode::<TokenClaims>(
        &token.into(), 
        &DecodingKey::from_secret(secret), 
        &Validation::new(Algorithm::HS256),
    );

    match decode {
        Ok(token) => Ok(token.claims.sub),
        Err(e) => Err(crate::error::ForumError::Token(e.to_string()))
    }
}


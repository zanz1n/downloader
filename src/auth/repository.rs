use std::time::Duration;

use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{
    errors::ErrorKind as JwtErrorKind, Algorithm, DecodingKey, EncodingKey,
    Header, Validation,
};
use uuid::Uuid;

use super::{AuthError, FileToken, Permission, Token, UserToken};

pub struct TokenRepository {
    enc_key: EncodingKey,
    dec_key: DecodingKey,
    header: Header,
    validation: Validation,

    user_token_duration: Duration,
    max_token_duration: Duration,

    srv_secret: Vec<u8>,
}

impl TokenRepository {
    pub fn new(
        algo: Algorithm,
        enc_key: EncodingKey,
        dec_key: DecodingKey,
        user_token_duration: Duration,
        max_token_duration: Duration,
        srv_secret: Vec<u8>,
    ) -> Self {
        Self {
            enc_key,
            dec_key,
            header: Header::new(algo),
            validation: Validation::new(algo),
            user_token_duration,
            max_token_duration,
            srv_secret,
        }
    }
}

impl TokenRepository {
    pub fn generate_user_token(
        &self,
        user_id: Uuid,
        permission: Permission,
        username: String,
    ) -> Result<String, AuthError> {
        let now = Utc::now();

        let claims = UserToken {
            user_id,
            created_at: now,
            expiration: now + self.user_token_duration,
            issuer: "SRV".into(),
            permission,
            username,
        };

        jsonwebtoken::encode(&self.header, &claims, &self.enc_key)
            .map_err(|_| AuthError::GenerateTokenFailed)
    }

    pub fn generate_file_token(
        &self,
        file_id: Uuid,
        expiration: Duration,
        issuer: String,
        permission: Permission,
    ) -> Result<String, AuthError> {
        if expiration > self.max_token_duration {
            return Err(AuthError::TokenExpirationTooLong {
                got: expiration,
                max: self.max_token_duration,
            });
        }

        let now = Utc::now();

        let claims = FileToken {
            file_id,
            created_at: now,
            expiration: now + expiration,
            issuer,
            permission,
        };

        jsonwebtoken::encode(&self.header, &claims, &self.enc_key).map_err(
            |error| {
                tracing::error!(%error, "generate JWT token failed");
                AuthError::GenerateTokenFailed
            },
        )
    }

    pub fn decode_token(&self, token: &str) -> Result<Token, AuthError> {
        jsonwebtoken::decode(token, &self.dec_key, &self.validation)
            .map_err(|error| match error.kind() {
                JwtErrorKind::ExpiredSignature => AuthError::ExpiredToken,
                JwtErrorKind::ImmatureSignature => AuthError::ImatureToken,
                _ => AuthError::InvalidToken,
            })
            .map(|v| v.claims)
    }

    pub fn verify_srv_key(&self, token: &str) -> Result<bool, AuthError> {
        let vec = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| AuthError::InvalidToken)?;

        if vec.len() != self.srv_secret.len() {
            return Err(AuthError::InvalidToken);
        }

        let eq = vec.iter().eq(&self.srv_secret);
        Ok(eq)
    }
}

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

        let claims = Token::User(UserToken {
            user_id,
            created_at: now,
            expiration: now + self.user_token_duration,
            issuer: "SRV".into(),
            permission,
            username,
        });

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

        let claims = Token::File(FileToken {
            file_id,
            created_at: now,
            expiration: now + expiration,
            issuer,
            permission,
        });

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use base64::Engine;
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
    use rand::RngCore;
    use test_log::test;
    use uuid::Uuid;

    use crate::auth::{Permission, Token};

    use super::TokenRepository;

    const USER_TOKEN_DURATION: Duration = Duration::from_secs(1);

    fn rand_vec(size: usize) -> Vec<u8> {
        let mut vec = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut vec);
        vec
    }

    fn rand_string() -> String {
        base64::engine::general_purpose::STANDARD.encode(rand_vec(24))
    }

    fn repository() -> TokenRepository {
        let key = rand_vec(512);
        let srv_secret = rand_vec(128);

        let algo = Algorithm::HS256;
        let enc_key = EncodingKey::from_secret(&key);
        let dec_key = DecodingKey::from_secret(&key);

        let user_token_duration = USER_TOKEN_DURATION;
        let max_token_duration = Duration::from_secs(30 * 24 * 3600);

        TokenRepository::new(
            algo,
            enc_key,
            dec_key,
            user_token_duration,
            max_token_duration,
            srv_secret,
        )
    }

    #[test]
    fn test_create_user_token() {
        let repo = repository();

        let user_id = Uuid::new_v4();
        let permission = Permission::empty()
            .union(Permission::UNPRIVILEGED)
            .union(Permission::WRITE_USERS);
        let username = rand_string();

        let tk = repo
            .generate_user_token(user_id, permission, username.clone())
            .unwrap();

        let data = repo
            .decode_token(&tk)
            .expect("failed to decode generated token");

        let data = match data {
            Token::User(v) => v,
            Token::File(_) => panic!("decoded wrong token type"),
        };

        assert_eq!(data.issuer, "SRV");
        assert_eq!(
            (data.expiration - data.created_at).num_seconds(),
            USER_TOKEN_DURATION.as_secs() as i64
        );
        assert_eq!(data.permission, permission);
        assert_eq!(data.user_id, user_id);
        assert_eq!(data.username, username);
    }

    #[test]
    fn test_create_file_token() {
        let repo = repository();

        let file_id = Uuid::new_v4();
        let expiration = Duration::from_secs(327);
        let issuer = format!("user/{}", Uuid::new_v4());
        let permission = Permission::ADMIN;

        let tk = repo
            .generate_file_token(
                file_id,
                expiration,
                issuer.clone(),
                permission,
            )
            .unwrap();

        let data = repo
            .decode_token(&tk)
            .expect("failed to decode generated token");

        let data = match data {
            Token::User(_) => panic!("decoded wrong token type"),
            Token::File(v) => v,
        };

        assert_eq!(data.issuer, issuer);
        assert_eq!(
            (data.expiration - data.created_at).num_seconds(),
            expiration.as_secs() as i64
        );
        assert_eq!(data.permission, permission);
        assert_eq!(data.file_id, file_id);
    }
}

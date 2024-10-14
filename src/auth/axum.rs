use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{header, request::Parts, StatusCode},
};
use serde::Deserialize;

use crate::{auth::AuthError, errors::DownloaderError};

use super::{repository::TokenRepository, Token};

#[derive(Deserialize)]
struct AuthorizationQuery {
    token: String,
}

pub struct Authorization(pub Token);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Authorization {
    type Rejection = DownloaderError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(header::AUTHORIZATION);

        let (strategy, token) = if let Some(auth_header) = auth_header {
            let s = auth_header
                .to_str()
                .map_err(|_| AuthError::InvalidAuthHeader)?
                .split(' ')
                .collect::<Vec<_>>();

            if s.len() != 2 {
                return Err(AuthError::InvalidAuthHeader.into());
            }

            (s[0], s[1].to_owned())
        } else {
            let token = Query::<AuthorizationQuery>::try_from_uri(&parts.uri)
                .map_err(|_| AuthError::AuthorizationRequired)?
                .0
                .token;

            ("Bearer", token)
        };

        let repo =
            parts.extensions.get::<TokenRepository>().ok_or_else(|| {
                DownloaderError::Other(
                    format!(
                        "Extension of type `{}` was not found. \
                        Perhaps you forgot to add it? See `axum::Extension`.",
                        std::any::type_name::<Arc<TokenRepository>>()
                    ),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        match strategy {
            "Bearer" => repo.decode_token(&token),
            "Secret" => repo.verify_srv_key(&token).and_then(|ok| {
                if ok {
                    Ok(Token::Server)
                } else {
                    Err(AuthError::InvalidToken)
                }
            }),
            s => {
                return Err(AuthError::InvalidAuthStrategy(
                    s.to_owned(),
                    &["Bearer", "Secret"],
                )
                .into())
            }
        }
        .map(Authorization)
        .map_err(DownloaderError::Auth)
    }
}

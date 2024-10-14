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

        let repo = parts.extensions.get::<Arc<TokenRepository>>().ok_or_else(
            || {
                DownloaderError::Other(
                    format!(
                        "Extension of type `{}` was not found. \
                        Perhaps you forgot to add it? See `axum::Extension`.",
                        std::any::type_name::<Arc<TokenRepository>>()
                    ),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            },
        )?;

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        extract::FromRequestParts,
        http::{header, request::Builder, Request},
    };
    use test_log::test;
    use uuid::Uuid;

    use crate::auth::{
        axum::Authorization, repository::tests::repository, Permission, Token,
    };

    async fn test_requests_insertions<F: FnOnce(Builder, String) -> Builder>(
        f: F,
    ) {
        let repo = Arc::new(repository());

        let user_id = Uuid::new_v4();
        let permission = Permission::all();
        let username = Uuid::new_v4().to_string();

        let token = repo
            .generate_user_token(user_id, permission, username.clone())
            .unwrap();

        let mut parts = f(Request::builder().extension(repo.clone()), token)
            .body(())
            .unwrap()
            .into_parts()
            .0;

        let token = Authorization::from_request_parts(&mut parts, &())
            .await
            .expect("Failed to extract created token")
            .0;

        let token = match token {
            Token::User(user_token) => user_token,
            _ => panic!("expected user token, but got {token:?}"),
        };

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.permission, permission);
        assert_eq!(token.username, username);
    }

    #[test(tokio::test)]
    async fn test_header_bearer_token() {
        test_requests_insertions(|builder, token| {
            builder.header(header::AUTHORIZATION, format!("Bearer {token}"))
        })
        .await
    }

    #[test(tokio::test)]
    async fn test_query_bearer_token() {
        test_requests_insertions(|builder, token| {
            builder.uri(format!("https://example.com?token={token}"))
        })
        .await
    }

    #[test(tokio::test)]
    async fn test_header_server_key() {
        let repo = Arc::new(repository());

        let token = repo.get_srv_key();

        let mut parts = Request::builder()
            .extension(repo.clone())
            .header(header::AUTHORIZATION, format!("Secret {token}"))
            .body(())
            .unwrap()
            .into_parts()
            .0;

        let token = Authorization::from_request_parts(&mut parts, &())
            .await
            .expect("Failed to extract created token")
            .0;

        match token {
            Token::Server => {}
            _ => panic!("expected server token, but got {token:?}"),
        }
    }
}

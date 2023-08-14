use std::fmt::Display;

use tracing::Instrument;

use super::{oidc::oidchandler::OidcHandler, token::TokenClaims};
// Create a function by token that will return the user's email
// - Access token
// - Refresh token
// - OIDC token

#[derive(Clone)]
pub struct AuthConfig {
    oidc: Option<OidcHandler>,
}

#[derive(Debug, PartialEq)]
pub enum TokenExtractError {
    InvalidToken(String),
    OidcDisabled,
}

impl Display for TokenExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenExtractError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            TokenExtractError::OidcDisabled => write!(f, "OIDC disabled"),
        }
    }
}

pub enum Token {
    Access(String),
    Refresh(String),
    Oidc(String),
}

impl Token {
    pub async fn get_user_email(&self, auth: AuthConfig) -> Result<String, TokenExtractError> {
        let span = tracing::span!(tracing::Level::INFO, "AUTH::get_user_email");
        async move {
            match self {
                Token::Access(token) => get_user_email_from_token(token.to_string(), false),
                Token::Refresh(token) => get_user_email_from_token(token.to_string(), true),
                Token::Oidc(token) => {
                    get_user_email_from_oidc_token(token.to_string(), auth.clone()).await
                }
            }
        }
        .instrument(span)
        .await
    }
}

fn get_user_email_from_token(token: String, refresh: bool) -> Result<String, TokenExtractError> {
    let span = tracing::span!(tracing::Level::DEBUG, "AUTH::local");
    let _enter = span.enter();
    let claims = match TokenClaims::validate_token(token, refresh) {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Invalid token: {}", err);
            return Err(TokenExtractError::InvalidToken(err.to_string()));
        }
    };
    tracing::debug!("email: {}", claims.email);
    Ok(claims.email)
}

async fn get_user_email_from_oidc_token(
    token: String,
    auth: AuthConfig,
) -> Result<String, TokenExtractError> {
    let span = tracing::span!(tracing::Level::DEBUG, "AUTH::oidc");
    async move {
        let oidc_handler = match auth.oidc {
            Some(oidc) => oidc,
            None => {
                tracing::error!("OIDC disabled");
                return Err(TokenExtractError::OidcDisabled);
            }
        };
        match oidc_handler.validate_token(token).await {
            Ok((ok, value)) => {
                tracing::debug!("ok: {}, value: {}", ok, value);
                if ok {
                    let email = value["email"].to_string().replace('\"', "");
                    tracing::debug!("email: {}", email);
                    return Ok(email);
                }
                Err(TokenExtractError::InvalidToken("Invalid token".to_string()))
            }
            Err(err) => {
                tracing::error!("Invalid token: {}", err);
                Err(TokenExtractError::InvalidToken(err.to_string()))
            }
        }
    }
    .instrument(span)
    .await
}

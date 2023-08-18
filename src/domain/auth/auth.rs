use std::fmt::Display;

use tracing::Instrument;

use crate::{config::Config, domain::token::token::TokenClaims};

use super::super::oidc::oidchandler::OidcHandler;

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
    pub async fn get_user_email(&self, config: Config) -> Result<String, TokenExtractError> {
        let span = tracing::span!(tracing::Level::INFO, "AUTH::get_user_email");
        async move {
            match self {
                Token::Access(token) => get_user_email_from_token(token.to_string(), false, config),
                Token::Refresh(token) => get_user_email_from_token(token.to_string(), true, config),
                Token::Oidc(token) => {
                    get_user_email_from_oidc_token(token.to_string(), config).await
                }
            }
        }
        .instrument(span)
        .await
    }
}

fn get_user_email_from_token(
    token: String,
    refresh: bool,
    config: Config,
) -> Result<String, TokenExtractError> {
    let span = tracing::span!(tracing::Level::DEBUG, "AUTH::local");
    let _enter = span.enter();
    let claims = match TokenClaims::validate_token(token, refresh, config.auth.clone()) {
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
    config: Config,
) -> Result<String, TokenExtractError> {
    let span = tracing::span!(tracing::Level::DEBUG, "AUTH::oidc");
    async move {
        if !config.oidc_enabled {
            return Err(TokenExtractError::OidcDisabled);
        }
        let oidc_handler = config.oidc_back.clone();
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

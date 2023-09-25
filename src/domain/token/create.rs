use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    domain::user::user::User,
    driven::repository::{
        postgres::refresh_token::SearchRefreshToken, repo::Repository, PersistenceConfig,
    },
};
use tracing::Instrument;

use super::refresh_token::RefreshToken;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CreateRefreshTokenError {
    InvalidData(String),
    ServerError(String),
}

impl Display for CreateRefreshTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateRefreshTokenError::InvalidData(msg) => write!(f, "Invalid Data: {}", msg),
            CreateRefreshTokenError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

pub async fn create_refresh_token<
    'a,
    B: PersistenceConfig,
    T: Repository<RefreshToken, SearchRefreshToken, B>,
>(
    repo: T,
    user: &'a User,
    config: Config,
) -> Result<String, CreateRefreshTokenError> {
    let span = tracing::span!(tracing::Level::INFO, "Action::create_refresh_token");
    async move {
        match RefreshToken::create_token(
            user.id.clone(),
            user.email.clone(),
            config.app_name.to_string(),
            config.auth.clone(),
        ) {
            Ok(refresh_token) => {
                tracing::info!(email = user.email.to_string(), "Refresh token inited");
                match repo.create(refresh_token.clone()).await {
                    Ok(_) => {
                        tracing::info!(email = user.email.to_string(), "Refresh token created");
                        return Ok(refresh_token.token);
                    }
                    Err(err) => {
                        tracing::error!(
                            email = user.email.to_string(),
                            err = err.to_string(),
                            "Error will creating refresh token"
                        );
                        return Err(CreateRefreshTokenError::ServerError(
                            "Error will creating refresh token".to_string(),
                        ));
                    }
                }
            }
            Err(err) => {
                tracing::error!(
                    email = user.email.to_string(),
                    "Error creating refresh token: {}",
                    err
                );
                return Err(CreateRefreshTokenError::InvalidData(
                    "Error creating refresh token".to_string(),
                ));
            }
        }
    }
    .instrument(span)
    .await
}

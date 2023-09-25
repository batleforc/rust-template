use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    domain::{token::token::TokenClaims, user::user::User},
    driven::repository::{
        postgres::refresh_token::SearchRefreshToken, repo::Repository, PersistenceConfig,
    },
};
use tracing::Instrument;

use super::refresh_token::RefreshToken;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CreateAccessTokenError {
    InvalidData(String),
    ServerError(String),
}

impl Display for CreateAccessTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateAccessTokenError::InvalidData(msg) => write!(f, "Invalid Data: {}", msg),
            CreateAccessTokenError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

pub async fn create_access_token<
    'a,
    B: PersistenceConfig,
    T: Repository<RefreshToken, SearchRefreshToken, B>,
>(
    repo: T,
    refresh_token: String,
    config: Config,
) -> Result<String, CreateAccessTokenError> {
    let span = tracing::span!(tracing::Level::INFO, "Action::create_access_token");
    async move {
        let mut search = SearchRefreshToken::new();
        search.token = Some(refresh_token.to_string());
        match repo.find_one(search).await {
            Ok(result) => {
                tracing::info!("Refresh token found");
                if result.created_at + chrono::Duration::days(7) < chrono::Utc::now() {
                    tracing::info!("Refresh token expired");
                    return Err(CreateAccessTokenError::InvalidData(
                        "Refresh token expired".to_string(),
                    ));
                }
                let mut claim = match TokenClaims::validate_token(
                    refresh_token.to_string(),
                    true,
                    config.auth.clone(),
                ) {
                    Ok(claim) => {
                        tracing::info!("Refresh token validated");
                        claim
                    }
                    Err(err) => {
                        tracing::error!(
                            err = err.to_string(),
                            "Error will validating refresh token"
                        );
                        return Err(CreateAccessTokenError::InvalidData(
                            "Error will validating refresh token".to_string(),
                        ));
                    }
                };
                claim.to_access_token();
                match claim.sign_token(config.auth.clone()) {
                    Ok(token) => {
                        tracing::info!("Access token created");
                        return Ok(token);
                    }
                    Err(err) => {
                        tracing::error!(err = err.to_string(), "Error will creating access token");
                        return Err(CreateAccessTokenError::ServerError(
                            "Error will creating access token".to_string(),
                        ));
                    }
                }
            }
            Err(err) => {
                tracing::error!(err = err.to_string(), "Error will searching refresh token");
                return Err(CreateAccessTokenError::ServerError(
                    "Error will searching refresh token".to_string(),
                ));
            }
        }
    }
    .instrument(span)
    .await
}

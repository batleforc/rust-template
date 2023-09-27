use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

use crate::{
    config::Config,
    domain::token::refresh_token::RefreshToken,
    driven::repository::{
        postgres::refresh_token::SearchRefreshToken, repo::Repository, repo_error::RepoDeleteError,
        PersistenceConfig,
    },
};

#[derive(Serialize, Deserialize, Clone)]
pub enum LogoutError {
    InvalidRefreshToken(String),
    ServerError(String),
}

pub async fn logout<
    'a,
    B: PersistenceConfig,
    D: Repository<RefreshToken, SearchRefreshToken, B>,
>(
    repo_token: D,
    refresh_token: String,
) -> Result<(), LogoutError> {
    let span = tracing::span!(tracing::Level::INFO, "Action::logout");
    async move {
        match repo_token.delete(refresh_token).await {
            Ok(_) => Ok(()),
            Err(err) => match err {
                RepoDeleteError::NotFound => {
                    tracing::info!("Refresh token not found");
                    Err(LogoutError::InvalidRefreshToken(err.to_string()))
                }
                RepoDeleteError::InvalidData(msg) => {
                    tracing::error!("Invalid data while deleting refresh token");
                    Err(LogoutError::InvalidRefreshToken(msg))
                }
                _ => {
                    tracing::error!("Error while deleting refresh token");
                    Err(LogoutError::ServerError(err.to_string()))
                }
            },
        }
    }
    .instrument(span)
    .await
}

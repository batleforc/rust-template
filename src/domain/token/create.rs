use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    domain::user::user::User,
    driven::repository::{
        postgres::refresh_token::SearchRefreshToken, repo::Repository, PersistenceConfig,
    },
};

use super::{
    refresh_token::RefreshToken,
    token::{self, TokenClaims},
};

#[derive(Serialize, Deserialize, Clone)]
pub enum CreateRefreshTokenError {
    InvalidData(String),
    ServerError(String),
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
    let token_claim = TokenClaims::new(
        user.id,
        user.email.to_string(),
        config.app_name.to_string(),
        true,
    );
    Err(CreateRefreshTokenError::ServerError(
        "Not implemented yet".to_string(),
    ))
}

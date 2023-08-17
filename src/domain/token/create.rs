use serde::{Deserialize, Serialize};

use crate::{
    domain::user::user::User,
    driven::repository::{
        postgres::refresh_token::SearchRefreshToken, repo::Repository, PersistenceConfig,
    },
};

use super::refresh_token::RefreshToken;

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
) -> Result<String, CreateRefreshTokenError> {
    Err(CreateRefreshTokenError::ServerError(
        "Not implemented yet".to_string(),
    ))
}

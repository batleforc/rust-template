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

#[cfg(test)]
mod tests {
    use crate::{
        config::{self, parse_test_config},
        domain::token::refresh_token::RefreshToken,
        driven::repository::{
            postgres::{
                config::ConfigPG, refresh_token::SearchRefreshToken,
                repo_refresh_token::RefreshTokenPGRepo,
            },
            repo::Repository,
        },
    };

    #[actix_web::test]
    async fn should_create_a_sandwich() {
        let main_config = parse_test_config();
        let config = &ConfigPG::new(main_config.persistence);
        let repo: RefreshTokenPGRepo =
            match Repository::<RefreshToken, SearchRefreshToken, ConfigPG>::new(config) {
                Ok(repo) => repo,
                Err(err) => panic!("Error creating repository: {}", err),
            };
        let entity = RefreshToken::new(
            "token".to_string(),
            "email".to_string(),
            uuid::Uuid::new_v4(),
        );
        let res = repo.create(entity).await;
    }

    async fn clean_db(repo: RefreshTokenPGRepo) {
        let _ = repo
            .delete_many(SearchRefreshToken {
                user_id: None,
                token: None,
                email: Some("%".to_string()),
                created_before: None,
            })
            .await;
    }
}

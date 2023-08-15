use crate::{
    domain::token::refresh_token::RefreshToken,
    driven::repository::{
        postgres::refresh_token::RefreshTokenPG,
        repo_error::{
            RepoCreateError, RepoDeleteError, RepoFindAllError, RepoSelectError, RepoUpdateError,
        },
    },
};

use super::{super::repo::Repository, config::ConfigPG, refresh_token::SearchRefreshToken};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::types::ToSql;

#[derive(Debug, Clone)]
pub struct RefreshTokenPGRepo {
    pub pool: Pool,
}

#[async_trait]
impl Repository<RefreshToken, SearchRefreshToken, ConfigPG> for RefreshTokenPGRepo {
    fn new(config: &ConfigPG) -> Result<Self, String>
    where
        Self: Sized,
    {
        let pool = config.pool.clone().unwrap().clone();
        Ok(RefreshTokenPGRepo { pool })
    }
    async fn init(&self) -> Result<(), String> {
        tracing::info!("Initializing refresh token table");
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let extension_stmt = "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";";
        let create_table_stmt = "
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            user_id UUID NOT NULL,
            token VARCHAR NOT NULL,
            PRIMARY KEY (user_id, token)
        );";
        match client.execute(extension_stmt, &[]).await {
            Ok(_) => {
                tracing::info!("Extension uuid-ossp created");
            }
            Err(e) => {
                tracing::error!("Error creating extension uuid-ossp: {}", e);
                drop(client);
                return Err(e.to_string());
            }
        }
        match client.execute(create_table_stmt, &[]).await {
            Ok(_) => {
                tracing::info!("Table refresh token created");
                drop(client);
            }
            Err(e) => {
                tracing::error!("Error creating table refresh token: {}", e);
                drop(client);
                return Err(e.to_string());
            }
        }
        Ok(())
    }

    async fn create(&self, refresh_token: RefreshToken) -> Result<RefreshToken, RepoCreateError> {
        tracing::info!("Creating refresh token");
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let stmt = "
        INSERT INTO refresh_tokens (user_id, token)
        VALUES ($1, $2);";
        let params: [&(dyn ToSql + Sync); 2] = [&refresh_token.user_id, &refresh_token.token];
        match client.execute(stmt, &params).await {
            Ok(_) => {
                tracing::info!("Refresh token created");
                drop(client);
                Ok(refresh_token)
            }
            Err(e) => {
                tracing::error!("Error creating refresh token: {}", e);
                drop(client);
                Err(RepoCreateError::Unknown(e.to_string()))
            }
        }
    }
    async fn find_one(&self, search: SearchRefreshToken) -> Result<RefreshToken, RepoSelectError> {
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let (search_querry, search_param) = match search.turn_into_search() {
            Ok((querry, param)) => (querry, param),
            Err(_) => {
                return Err(RepoSelectError::SelectParamInvalid(
                    "No search param".to_string(),
                ))
            }
        };
        let mut param: Vec<&(dyn ToSql + Sync)> = Vec::new();
        for i in &search_param {
            param.push(i as &(dyn ToSql + Sync));
        }
        match client
            .query_one(
                &format!("SELECT * FROM refresh_tokens WHERE {}", search_querry),
                &param,
            )
            .await
        {
            Ok(row) => {
                tracing::trace!("Got row");
                let refresh = RefreshTokenPG::from_row(&row);
                drop(client);
                Ok(refresh.try_into().unwrap())
            }
            Err(e) => {
                tracing::error!("Error finding refresh: {}", e);
                drop(client);
                Err(RepoSelectError::Unknown(e.to_string()))
            }
        }
    }

    async fn find_all(
        &self,
        search: SearchRefreshToken,
    ) -> Result<Vec<RefreshToken>, RepoFindAllError> {
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        match {
            match search.turn_into_search() {
                Ok((querry, search_param)) => {
                    tracing::trace!(
                        param = ?search_param.clone(),
                        querry = ?querry.clone(),
                        "Got search param"
                    );
                    let mut param: Vec<&(dyn ToSql + Sync)> = Vec::new();
                    for i in &search_param {
                        param.push(i as &(dyn ToSql + Sync));
                    }
                    client
                        .query(
                            &format!("SELECT * FROM refresh_tokens WHERE {}", querry),
                            &param,
                        )
                        .await
                }
                Err(_) => {
                    tracing::trace!("No search param");
                    client.query("SELECT * FROM refresh_tokens", &[]).await
                }
            }
        } {
            Ok(rows) => {
                tracing::trace!("Got rows");
                let mut refreshs = Vec::new();
                for row in rows {
                    let refresh = RefreshTokenPG::from_row(&row);
                    refreshs.push(refresh.try_into().unwrap());
                }
                drop(client);
                Ok(refreshs)
            }
            Err(e) => {
                tracing::error!("Error finding refreshs: {}", e);
                drop(client);
                Err(RepoFindAllError::Unknown(e.to_string()))
            }
        }
    }

    async fn delete(&self, token: String) -> Result<(), RepoDeleteError> {
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let stmt = "DELETE FROM refresh_tokens WHERE token = $1";
        match client.execute(stmt, &[&token]).await {
            Ok(_) => {
                drop(client);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Error deleting refresh: {}", e);
                drop(client);
                Err(RepoDeleteError::Unknown(e.to_string()))
            }
        }
    }

    async fn update(&self, refresh: RefreshToken) -> Result<RefreshToken, RepoUpdateError> {
        tracing::info!("DO NOT USE THE UPDATE FUNCTION for refresh tokens");
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let stmt = "
        UPDATE refresh_tokens
        SET user_id = $1, token = $2
        WHERE user_id = $1 AND token = $2;";
        let params: [&(dyn ToSql + Sync); 2] = [&refresh.user_id, &refresh.token];
        match client.execute(stmt, &params).await {
            Ok(_) => {
                drop(client);
                Ok(refresh)
            }
            Err(e) => {
                tracing::error!("Error updating refresh: {}", e);
                drop(client);
                Err(RepoUpdateError::Unknown(e.to_string()))
            }
        }
    }
}

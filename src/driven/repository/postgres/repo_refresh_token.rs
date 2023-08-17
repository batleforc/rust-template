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
use tracing::Instrument;

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
        let span = tracing::span!(tracing::Level::INFO, "RefreshTokenPGRepo::init",);
        async move {
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let extension_stmt = "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";";
            let create_table_stmt = "
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            user_id UUID NOT NULL,
            token VARCHAR NOT NULL,
            email VARCHAR NOT NULL,
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
        .instrument(span)
        .await
    }

    async fn create(&self, refresh_token: RefreshToken) -> Result<RefreshToken, RepoCreateError> {
        let span = tracing::span!(tracing::Level::INFO, "RefreshTokenPGRepo::create",);
        async move {
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let stmt = "
        INSERT INTO refresh_tokens (user_id, token, email)
        VALUES ($1, $2, $3);";
            let params: [&(dyn ToSql + Sync); 3] = [
                &refresh_token.user_id,
                &refresh_token.token,
                &refresh_token.email,
            ];
            match client.execute(stmt, &params).await {
                Ok(edited) => {
                    drop(client);
                    if edited == 0 {
                        tracing::error!("No data created");
                        return Err(RepoCreateError::InvalidData("No data created".to_string()));
                    }
                    tracing::info!("Refresh token created");
                    Ok(refresh_token)
                }
                Err(e) => {
                    tracing::error!("Error creating refresh token: {}", e);
                    drop(client);
                    Err(RepoCreateError::Unknown(e.to_string()))
                }
            }
        }
        .instrument(span)
        .await
    }
    async fn find_one(&self, search: SearchRefreshToken) -> Result<RefreshToken, RepoSelectError> {
        let span = tracing::span!(tracing::Level::INFO, "RefreshTokenPGRepo::find_one",);
        async move {
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
        .instrument(span)
        .await
    }

    async fn find_all(
        &self,
        search: SearchRefreshToken,
    ) -> Result<Vec<RefreshToken>, RepoFindAllError> {
        let span = tracing::span!(tracing::Level::INFO, "RefreshTokenPGRepo::find_all",);
        async move {
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
                    if refreshs.is_empty() {
                        tracing::error!("No refreshs found");
                        return Err(RepoFindAllError::NotFound);
                    }
                    Ok(refreshs)
                }
                Err(e) => {
                    tracing::error!("Error finding refreshs: {}", e);
                    drop(client);
                    Err(RepoFindAllError::Unknown(e.to_string()))
                }
            }
        }
        .instrument(span)
        .await
    }

    async fn delete(&self, token: String) -> Result<(), RepoDeleteError> {
        let span = tracing::span!(tracing::Level::INFO, "RefreshTokenPGRepo::delete",);
        async move {
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let stmt = "DELETE FROM refresh_tokens WHERE token = $1";
            match client.execute(stmt, &[&token]).await {
                Ok(edited) => {
                    drop(client);
                    if edited == 0 {
                        tracing::error!("No data deleted");
                        return Err(RepoDeleteError::NotFound);
                    }
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Error deleting refresh: {}", e);
                    drop(client);
                    Err(RepoDeleteError::Unknown(e.to_string()))
                }
            }
        }
        .instrument(span)
        .await
    }

    async fn delete_many(&self, search: SearchRefreshToken) -> Result<u64, RepoDeleteError> {
        let span = tracing::span!(tracing::Level::INFO, "RefreshTokenPGRepo::delete_many",);
        async move {
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let (search_querry, search_param) = match search.turn_into_search() {
                Ok((querry, param)) => (querry, param),
                Err(_) => return Err(RepoDeleteError::InvalidData("No search param".to_string())),
            };
            let mut param: Vec<&(dyn ToSql + Sync)> = Vec::new();
            for i in &search_param {
                param.push(i as &(dyn ToSql + Sync));
            }
            match client
                .execute(
                    &format!("DELETE FROM refresh_tokens WHERE {}", search_querry),
                    &param,
                )
                .await
            {
                Ok(edited) => {
                    drop(client);
                    if edited == 0 {
                        tracing::error!("No data deleted");
                        return Err(RepoDeleteError::NotFound);
                    }
                    Ok(edited)
                }
                Err(e) => {
                    tracing::error!("Error deleting refresh: {}", e);
                    drop(client);
                    Err(RepoDeleteError::Unknown(e.to_string()))
                }
            }
        }
        .instrument(span)
        .await
    }

    async fn update(&self, refresh: RefreshToken) -> Result<RefreshToken, RepoUpdateError> {
        let span = tracing::span!(
            tracing::Level::INFO,
            "RefreshTokenPGRepo::update::SHOULD NOT BE USED",
        );
        async move {
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
                Ok(edited) => {
                    drop(client);
                    if edited == 0 {
                        tracing::error!("Error updating refresh: {}", "No refresh found");
                        return Err(RepoUpdateError::Unknown("No refresh found".to_string()));
                    }
                    Ok(refresh)
                }
                Err(e) => {
                    tracing::error!("Error updating refresh: {}", e);
                    drop(client);
                    Err(RepoUpdateError::Unknown(e.to_string()))
                }
            }
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use crate::config::parse_test_config;

    use super::*;

    #[serial(repo_user)]
    #[actix_web::test]
    async fn test_token() {
        let main_config = parse_test_config();
        let config = &ConfigPG::new(main_config.persistence);
        let repo: RefreshTokenPGRepo =
            match Repository::<RefreshToken, SearchRefreshToken, ConfigPG>::new(config) {
                Ok(repo) => repo,
                Err(err) => panic!("Error creating repository: {}", err),
            };
        match repo.init().await {
            Ok(_) => (),
            Err(err) => panic!("Error creating repository: {}", err),
        };
        let token = RefreshToken::new(
            "test".to_string(),
            "test@test.fr".to_string(),
            uuid::Uuid::new_v4(),
        );
        match repo.create(token.clone()).await {
            Ok(_) => (),
            Err(err) => panic!("Error creating token: {:?}", err.to_string()),
        };
        match repo.find_all(SearchRefreshToken::new()).await {
            Ok(users) => {
                assert_eq!(users.len(), 1);
                assert_eq!(users[0].email, token.email);
                assert_eq!(users[0].user_id, token.user_id);
                assert_eq!(users[0].token, token.token);
            }
            Err(err) => panic!("Error finding token: {:#?}", err),
        };
        match repo.delete(token.token).await {
            Ok(_) => (),
            Err(err) => panic!("Error deleting token: {:#?}", err),
        };
        config.pool.clone().unwrap().close();
    }
}

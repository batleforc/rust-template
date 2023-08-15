use crate::{
    domain::user::user::User,
    driven::repository::{
        postgres::user::UserPG,
        repo_error::{
            RepoCreateError, RepoDeleteError, RepoFindAllError, RepoSelectError, RepoUpdateError,
        },
    },
};

use super::{super::repo::Repository, config::ConfigPG, user::SearchUser};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::types::ToSql;

#[derive(Debug, Clone)]
pub struct UserPGRepo {
    pub pool: Pool,
}

#[async_trait]
impl Repository<User, SearchUser, ConfigPG> for UserPGRepo {
    fn new(config: &ConfigPG) -> Result<Self, String>
    where
        Self: Sized,
    {
        let pool = config.pool.clone().unwrap().clone();
        Ok(UserPGRepo { pool })
    }
    async fn init(&self) -> Result<(), String> {
        tracing::info!("Initializing user table");
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let extension_stmt = "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";";
        let create_table_stmt = "
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                nom VARCHAR(255) NOT NULL,
                prenom VARCHAR(255) NOT NULL,
                otp_secret VARCHAR(255),
                otp_url VARCHAR(255),
                otp_enabled BOOLEAN DEFAULT FALSE,
                one_time_token VARCHAR(255),
                is_oauth BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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
                tracing::info!("Table users created");
                drop(client);
            }
            Err(e) => {
                tracing::error!("Error creating table users: {}", e);
                drop(client);
                return Err(e.to_string());
            }
        }
        Ok(())
    }

    async fn create(&self, user: User) -> Result<User, RepoCreateError> {
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let stmt = "
            INSERT INTO users (email, password, name, surname, otp_secret, otp_url, otp_enabled, is_oauth)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,)";
        match client
            .execute(
                stmt,
                &[
                    &user.email,
                    &user.password,
                    &user.name,
                    &user.surname,
                    &user.otp_secret,
                    &user.otp_url,
                    &user.otp_enabled,
                    &user.is_oauth,
                ],
            )
            .await
        {
            Ok(edited) => {
                drop(client);
                if edited == 0 {
                    return Err(RepoCreateError::InvalidData("No user created".to_string()));
                }
                Ok(user)
            }
            Err(e) => {
                drop(client);
                Err(RepoCreateError::Unknown(e.to_string()))
            }
        }
    }

    async fn find_one(&self, search: SearchUser) -> Result<User, RepoSelectError> {
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
                &format!("SELECT * FROM users WHERE {}", search_querry),
                &param,
            )
            .await
        {
            Ok(row) => {
                tracing::trace!("Got row");
                let user = UserPG::from_row(&row);
                drop(client);
                Ok(user.try_into().unwrap())
            }
            Err(e) => {
                tracing::error!("Error finding user: {}", e);
                drop(client);
                Err(RepoSelectError::Unknown(e.to_string()))
            }
        }
    }

    async fn find_all(&self, search: SearchUser) -> Result<Vec<User>, RepoFindAllError> {
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
                        .query(&format!("SELECT * FROM users WHERE {}", querry), &param)
                        .await
                }
                Err(_) => {
                    tracing::trace!("No search param");
                    client.query("SELECT * FROM users", &[]).await
                }
            }
        } {
            Ok(rows) => {
                tracing::trace!("Got rows");
                let mut users = Vec::new();
                for row in rows {
                    let user = UserPG::from_row(&row);
                    users.push(user.try_into().unwrap());
                }
                drop(client);
                if users.is_empty() {
                    tracing::error!("User not found");
                    return Err(RepoFindAllError::NotFound);
                }
                Ok(users)
            }
            Err(e) => {
                tracing::error!("Error finding users: {}", e);
                drop(client);
                Err(RepoFindAllError::Unknown(e.to_string()))
            }
        }
    }

    async fn delete(&self, id: String) -> Result<(), RepoDeleteError> {
        if id.is_empty() {
            return Err(RepoDeleteError::InvalidData("No id".to_string()));
        }
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let stmt = "DELETE FROM users WHERE id = $1";
        match client.execute(stmt, &[&id]).await {
            Ok(edited) => {
                drop(client);
                if edited == 0 {
                    tracing::error!("User not found");
                    return Err(RepoDeleteError::NotFound);
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("Error deleting user: {}", e);
                drop(client);
                Err(RepoDeleteError::Unknown(e.to_string()))
            }
        }
    }

    async fn update(&self, user: User) -> Result<User, RepoUpdateError> {
        tracing::trace!("Getting pool");
        let client = self.pool.get().await.unwrap();
        tracing::trace!("Got pool");
        let stmt = "
            UPDATE users
            SET email = $1, password = $2, name = $3, surname = $4, otp_secret = $5, otp_url = $6, otp_enabled = $7, is_oauth = $8, updated_at = $9
            WHERE id = $10";
        match client
            .execute(
                stmt,
                &[
                    &user.email,
                    &user.password,
                    &user.name,
                    &user.surname,
                    &user.otp_secret,
                    &user.otp_url,
                    &user.otp_enabled,
                    &user.is_oauth,
                    &user.updated_at,
                    &user.id,
                ],
            )
            .await
        {
            Ok(edited) => {
                drop(client);
                if edited == 0 {
                    tracing::error!("User not found");
                    return Err(RepoUpdateError::NotFound);
                }
                Ok(user)
            }
            Err(e) => {
                tracing::error!("Error updating user: {}", e);
                drop(client);
                Err(RepoUpdateError::Unknown(e.to_string()))
            }
        }
    }
}

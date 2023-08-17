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
use tracing::Instrument;
use uuid::Uuid;

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
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::init");
        async move {
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let extension_stmt = "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";";
            let create_table_stmt = "
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                surname VARCHAR(255) NOT NULL,
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
        .instrument(span)
        .await
    }

    async fn create(&self, user: User) -> Result<User, RepoCreateError> {
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::create");
        async move{
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let stmt = "
                INSERT INTO users (id, email, password, name, surname, otp_secret, otp_url, otp_enabled, is_oauth)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
            match client
                .execute(
                    stmt,
                    &[
                        &user.id,
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
        }.instrument(span).await
    }

    async fn find_one(&self, search: SearchUser) -> Result<User, RepoSelectError> {
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::find_one");
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
        .instrument(span)
        .await
    }

    async fn find_all(&self, search: SearchUser) -> Result<Vec<User>, RepoFindAllError> {
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::find_all");
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
        .instrument(span)
        .await
    }

    async fn delete(&self, id: String) -> Result<(), RepoDeleteError> {
        if id.is_empty() {
            return Err(RepoDeleteError::InvalidData("No id".to_string()));
        }
        let parsed_id = match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_) => return Err(RepoDeleteError::InvalidData("Invalid id".to_string())),
        };
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::delete");
        async move {
            tracing::trace!("Getting pool");
            let client = self.pool.get().await.unwrap();
            tracing::trace!("Got pool");
            let stmt = "DELETE FROM users WHERE id = $1";
            match client.execute(stmt, &[&parsed_id]).await {
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
        .instrument(span)
        .await
    }

    async fn delete_many(&self, search: SearchUser) -> Result<u64, RepoDeleteError> {
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::delete_many",);
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
                    &format!("DELETE FROM users WHERE {}", search_querry),
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

    async fn update(&self, user: User) -> Result<User, RepoUpdateError> {
        let span = tracing::span!(tracing::Level::INFO, "UserPGRepo::update");
        async move {
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
                        &chrono::Utc::now(),
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
        }.instrument(span).await
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use crate::config::parse_test_config;

    use super::*;

    #[serial(repo_user)]
    #[actix_web::test]
    async fn test_user() {
        let main_config = parse_test_config();
        let config = &ConfigPG::new(main_config.persistence);
        let repo: UserPGRepo = match Repository::<User, SearchUser, ConfigPG>::new(config) {
            Ok(repo) => repo,
            Err(err) => panic!("Error creating repository: {}", err),
        };
        match repo.init().await {
            Ok(_) => (),
            Err(err) => panic!("Error creating repository: {}", err),
        };
        let mut user = User::new(
            "max@github.com".to_string(),
            "weebo".to_string(),
            "Max".to_string(),
            false,
        );
        match repo.create(user.clone()).await {
            Ok(_) => (),
            Err(err) => panic!("Error creating user: {:?}", err.to_string()),
        };
        match repo.find_all(SearchUser::new()).await {
            Ok(users) => {
                assert_eq!(users.len(), 1);
                assert_eq!(users[0].email, user.email);
                assert_eq!(users[0].password, user.password);
                assert_eq!(users[0].name, user.name);
                assert_eq!(users[0].surname, user.surname);
                assert_eq!(users[0].otp_enabled, user.otp_enabled);
                assert_eq!(users[0].is_oauth, user.is_oauth);
            }
            Err(err) => panic!("Error finding user: {:#?}", err),
        };
        user.name = "Weebz".to_string();
        user.surname = "Mustermann".to_string();
        user.otp_enabled = true;
        match repo.update(user.clone()).await {
            Ok(_) => (),
            Err(err) => panic!("Error updating user: {:#?}", err),
        };
        match repo.find_all(SearchUser::new()).await {
            Ok(users) => {
                assert_eq!(users.len(), 1);
                assert_eq!(users[0].email, user.email);
                assert_eq!(users[0].password, user.password);
                assert_eq!(users[0].name, user.name);
                assert_eq!(users[0].surname, user.surname);
                assert_eq!(users[0].otp_enabled, user.otp_enabled);
                assert_eq!(users[0].is_oauth, user.is_oauth);
            }
            Err(err) => panic!("Error finding user: {:#?}", err),
        };
        match repo.delete(user.id.to_string()).await {
            Ok(_) => (),
            Err(err) => panic!("Error deleting user: {:#?}", err),
        };
        config.pool.clone().unwrap().close();
    }

    #[serial(repo_user)]
    #[actix_web::test]
    async fn test_user_search() {
        let main_config = parse_test_config();
        let config = &ConfigPG::new(main_config.persistence);
        let repo: UserPGRepo = match Repository::<User, SearchUser, ConfigPG>::new(config) {
            Ok(repo) => repo,
            Err(err) => panic!("Error creating repository: {}", err),
        };
        match repo.init().await {
            Ok(_) => (),
            Err(err) => panic!("Error creating repository: {}", err),
        };
        for i in 0..10 {
            let user = User::new(
                format!("max{}@github.com", i).to_string(),
                format!("weeb{}", i).to_string(),
                format!("max{}", i).to_string(),
                false,
            );
            match repo.create(user.clone()).await {
                Ok(_) => (),
                Err(err) => panic!("Error creating user{}: {:#?}", i, err),
            };
        }
        match repo.find_all(SearchUser::new()).await {
            Ok(users) => {
                assert_eq!(users.len(), 10);
            }
            Err(err) => panic!("Error finding user: {:#?}", err),
        };
        let mut search = SearchUser::new();
        search.email = Some("%max1%".to_string());
        match repo.find_one(search.clone()).await {
            Ok(user) => {
                assert_eq!(user.email, "max1@github.com".to_string());
                assert_eq!(user.name, "max1".to_string());
                assert_eq!(user.surname, "weeb1".to_string());
            }
            Err(err) => panic!("Error finding user: {:#?}", err),
        };
        match repo.find_all(search.clone()).await {
            Ok(users) => {
                assert_eq!(users.len(), 1);
                assert_eq!(users[0].email, "max1@github.com".to_string());
                assert_eq!(users[0].name, "max1".to_string());
                assert_eq!(users[0].surname, "weeb1".to_string());
            }
            Err(err) => panic!("Error finding user: {:#?}", err),
        };
        clean_db(repo).await;
        config.pool.clone().unwrap().close();
    }

    async fn clean_db(repo: UserPGRepo) {
        let mut search = SearchUser::new();
        search.email = Some("%".to_string());
        match repo.delete_many(search).await {
            Ok(nbr) => println!("Deleted {} users", nbr),
            Err(err) => panic!("Error deleting users: {:#?}", err),
        }
    }
}

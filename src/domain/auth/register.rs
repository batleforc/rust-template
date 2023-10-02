use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

use crate::{
    config::Config,
    domain::user::user::User,
    driven::repository::{
        postgres::user::SearchUser, repo::Repository, repo_error::RepoSelectError,
        PersistenceConfig,
    },
};

#[derive(Serialize, Deserialize, Clone)]
pub enum RegisterError {
    AlreadyUsedEmail,
    ServerError,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct RegisterUser {
    pub surname: String,
    pub name: String,
    pub password: String,
    pub email: String,
}

pub async fn register<'a, B: PersistenceConfig, T: Repository<User, SearchUser, B>>(
    repo: T,
    register_body: RegisterUser,
) -> Result<bool, RegisterError> {
    let main_span = tracing::span!(tracing::Level::INFO, "Action:Register");
    async move {
        // check if user exist
        let mut search_for_new_user = SearchUser::new();
        search_for_new_user.email = Some(register_body.email.clone());
        match repo.find_one(search_for_new_user).await {
            Ok(_) => {
                tracing::info!(email = register_body.email.clone(), "User already exist");
                return Err(RegisterError::AlreadyUsedEmail);
            }
            Err(err) => match err {
                RepoSelectError::NoRowFound => {
                    tracing::info!(email = register_body.email.clone(), "User found")
                }
                _ => {
                    tracing::error!(
                        email = register_body.email.clone(),
                        err = err.to_string(),
                        "Error while fetching user"
                    );
                    return Err(RegisterError::ServerError);
                }
            },
        }
        // create new user
        let mut user = User::new(
            register_body.email.clone(),
            register_body.surname.clone(),
            register_body.name.clone(),
            false,
        );
        match user.update_password(register_body.password) {
            Ok(_) => {
                tracing::info!("Password set");
            }
            Err(err) => {
                tracing::error!(err = err.to_string(), "Error while setting password");
                return Err(RegisterError::ServerError);
            }
        };
        match repo.create(user).await {
            Ok(_) => {
                tracing::info!("User created");
                return Ok(true);
            }
            Err(err) => {
                tracing::error!(err = err.to_string(), "Error while creating user");
                return Err(RegisterError::ServerError);
            }
        }
    }
    .instrument(main_span)
    .await
}

use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

use crate::{
    config::Config,
    domain::user::{self, user::User},
    driven::repository::{
        postgres::user::SearchUser, repo::Repository, repo_error::RepoSelectError,
        PersistenceConfig,
    },
};

#[derive(Serialize, Deserialize, Clone)]
pub enum LoginOidcError {
    TokenInvalid(String),
    OidcDisabled(String),
    ServerError(String),
}

pub async fn login_oidc<'a, B: PersistenceConfig, T: Repository<User, SearchUser, B>>(
    repo_user: T,
    token: &'a String,
    config: Config,
) -> Result<User, LoginOidcError> {
    let span = tracing::span!(tracing::Level::INFO, "Action::login_oidc");
    async move {
        // get user info from oidc
        let user_info = match config.oidc_back.get_user_info(token.to_string()).await {
            Ok(user_info) => {
                tracing::info!(
                    email = user_info["email"].to_string().replace('\"', ""),
                    "User validated"
                );
                user_info
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
                return Err(LoginOidcError::ServerError(
                    "Error while getting user info".to_string(),
                ));
            }
        };
        // Find user in db
        let (create, user) = match repo_user
            .find_one(SearchUser {
                id: None,
                email: Some(user_info["email"].to_string().replace('\"', "")),
                one_time_token: None,
                surname: None,
                name: None,
            })
            .await
        {
            Ok(user) => {
                tracing::info!(
                    email = user_info["email"].to_string().replace('\"', ""),
                    "User found"
                );
                if !user.is_oauth {
                    tracing::info!(
                        email = user_info["email"].to_string().replace('\"', ""),
                        "User is not oauth"
                    );
                    return Err(LoginOidcError::TokenInvalid(
                        "User is not oauth".to_string(),
                    ));
                }
                // if user is oidc, update user
                (false, Some(user))
            }
            Err(err) => {
                // if no user, create user
                match err {
                    RepoSelectError::NoRowFound => {
                        tracing::info!(
                            email = user_info["email"].to_string().replace('\"', ""),
                            "User not found"
                        );
                        (true, None)
                    }
                    _ => {
                        tracing::error!(
                            email = user_info["email"].to_string().replace('\"', ""),
                            err = err.to_string(),
                            "Error while getting user"
                        );
                        return Err(LoginOidcError::ServerError(err.to_string()));
                    }
                }
            }
        };
        match create {
            true => {
                tracing::info!(
                    email = user_info["email"].to_string().replace('\"', ""),
                    "Creating user"
                );

                let new_user = User::new(
                    user_info["email"].to_string().replace('\"', ""),
                    user_info["family_name"].to_string().replace('\"', ""),
                    user_info["given_name"].to_string().replace('\"', ""),
                    true,
                );
                match repo_user.create(new_user.clone()).await {
                    Ok(user) => {
                        tracing::info!(
                            email = user_info["email"].to_string().replace('\"', ""),
                            "User created"
                        );
                        return Ok(user);
                    }
                    Err(err) => {
                        tracing::error!(
                            email = user_info["email"].to_string().replace('\"', ""),
                            err = err.to_string(),
                            "Error while creating user"
                        );
                        return Err(LoginOidcError::ServerError(err.to_string()));
                    }
                }
            }
            false => {
                tracing::info!(
                    email = user_info["email"].to_string().replace('\"', ""),
                    "Updating user"
                );
                let mut need_update = false;
                let mut in_db_user = user.unwrap();
                let user_given_name = user_info["given_name"].to_string().replace('\"', "");
                if user_given_name != in_db_user.surname {
                    tracing::debug!(name = user_given_name.clone(), "Updating user name");
                    in_db_user.name = user_given_name;
                    need_update = true;
                }
                let user_familly_name = user_info["family_name"].to_string().replace('\"', "");
                if user_familly_name != in_db_user.name {
                    tracing::debug!(surname = user_familly_name.clone(), "Updating user surname");
                    in_db_user.surname = user_familly_name;
                    need_update = true;
                }
                if need_update {
                    tracing::info!("User need update");
                    match repo_user.update(in_db_user).await {
                        Ok(user) => {
                            tracing::info!(
                                email = user_info["email"].to_string().replace('\"', ""),
                                "User updated"
                            );
                            return Ok(user);
                        }
                        Err(err) => {
                            tracing::error!(
                                email = user_info["email"].to_string().replace('\"', ""),
                                err = err.to_string(),
                                "Error while updating user"
                            );
                            return Err(LoginOidcError::ServerError(err.to_string()));
                        }
                    };
                }
                return Ok(in_db_user);
            }
        }
    }
    .instrument(span)
    .await
}

use crate::{
    config::Config,
    domain::{
        token::refresh_token::{self, RefreshToken},
        user::user::User,
    },
    driven::repository::{
        postgres::{refresh_token::SearchRefreshToken, user::SearchUser},
        repo::Repository,
        PersistenceConfig,
    },
};
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone)]
pub enum LoginError {
    EmailPasswordDoesNotMatch,
    UserIsOauth,
    ServerError(String),
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum LoginStatus {
    Otp,
    Refresh,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct LoginUserReturn {
    pub user: Option<User>,
    pub status: LoginStatus,
    pub token: String,
}

pub async fn login<
    'a,
    B: PersistenceConfig,
    T: Repository<User, SearchUser, B>,
    D: Repository<RefreshToken, SearchRefreshToken, B>,
>(
    repo: T,
    repo_token: D,
    email: &'a String,
    password: &'a String,
    config: Config,
) -> Result<LoginUserReturn, LoginError> {
    let main_span = tracing::span!(tracing::Level::INFO, "Action::login");
    async move {
        let user = match repo
            .find_one(SearchUser {
                id: None,
                email: Some(email.to_string()),
                one_time_token: None,
                surname: None,
                name: None,
            })
            .await
        {
            Ok(user) => {
                tracing::info!(email = email.to_string(), "User found");
                user
            }
            Err(err) => {
                tracing::error!(
                    email = email.to_string(),
                    err = err.to_string(),
                    "User not found"
                );
                return Err(LoginError::EmailPasswordDoesNotMatch);
            }
        };
        if user.is_oauth {
            tracing::info!(email = email.to_string(), "User is oauth");
            return Err(LoginError::UserIsOauth);
        }
        match user.validate_password(password.to_string()) {
            Ok(valid) => {
                if !valid {
                    tracing::error!(email = email.to_string(), "Wrong password");
                    return Err(LoginError::EmailPasswordDoesNotMatch);
                }
                tracing::info!(email = email.to_string(), "Password is valid");
            }
            Err(err) => {
                tracing::error!(
                    email = email.to_string(),
                    err = err.to_string(),
                    "Error will validating password"
                );
                return Err(LoginError::ServerError(err.to_string()));
            }
        }
        if user.otp_enabled {
            tracing::warn!(email = email.to_string(), "OTP is not done yet");
            todo!("Generate OTP and save it in the database");
            return Ok(LoginUserReturn {
                user: Some(user),
                status: LoginStatus::Otp,
                token: "".to_string(),
            });
        }
        let token = match RefreshToken::create_token(
            user.id.clone(),
            user.email.clone(),
            config.app_name.to_string(),
            config.auth.clone(),
        ) {
            Ok(refresh_token) => match repo_token.create(refresh_token.clone()).await {
                Ok(_) => {
                    tracing::info!(email = email.to_string(), "Refresh token created");
                    refresh_token.token
                }
                Err(err) => {
                    tracing::error!(
                        email = email.to_string(),
                        err = err.to_string(),
                        "Error will creating refresh token"
                    );
                    return Err(LoginError::ServerError(err.to_string()));
                }
            },
            Err(err) => {
                tracing::error!(
                    email = email.to_string(),
                    err = err.to_string(),
                    "Error will creating refresh token"
                );
                return Err(LoginError::ServerError(err.to_string()));
            }
        };
        Ok(LoginUserReturn {
            user: Some(user),
            status: LoginStatus::Refresh,
            token,
        })
    }
    .instrument(main_span)
    .await
}

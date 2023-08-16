use crate::{
    domain::user::user::User,
    driven::repository::{postgres::user::SearchUser, repo::Repository, PersistenceConfig},
};

use serde::{Deserialize, Serialize};
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

pub async fn login<'a, B: PersistenceConfig, T: Repository<User, SearchUser, B>>(
    repo: T,
    email: &'a String,
    password: &'a String,
) -> Result<LoginUserReturn, LoginError> {
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
        Ok(user) => user,
        Err(_) => return Err(LoginError::EmailPasswordDoesNotMatch),
    };
    if user.is_oauth {
        return Err(LoginError::UserIsOauth);
    }
    match user.validate_password(password.to_string()) {
        Ok(valid) => {
            if !valid {
                return Err(LoginError::EmailPasswordDoesNotMatch);
            }
        }
        Err(err) => return Err(LoginError::ServerError(err.to_string())),
    }
    if user.otp_enabled {
        todo!("Generate OTP and save it in the database");
        return Ok(LoginUserReturn {
            user: Some(user),
            status: LoginStatus::Otp,
            token: "".to_string(),
        });
    }

    todo!("Generate Refresh and save it in the database");
    Ok(LoginUserReturn {
        user: Some(user),
        status: LoginStatus::Refresh,
        token: "".to_string(),
    })
}

use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

use crate::model::{
    token::{self, RefreshToken, TokenClaims},
    user::User,
};

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum LoginStatus {
    OtpStep,
    RefreshStep,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginUserReturn {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub user: Option<User>,
    pub status: LoginStatus,
    pub token: Option<String>,
}

/// Login user
#[utoipa::path(
  tag = "Auth",
  request_body = LoginUser,
  operation_id = "login",
  path = "/api/auth/login",
  responses(
      (status = 200, description = "Login user", body = LoginUserReturn)
  )
)]
#[post("/login")]
pub async fn login(login_body: web::Json<LoginUser>, db_pool: web::Data<Pool>) -> impl Responder {
    let body = login_body.into_inner();
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    let check_user_span = tracing::info_span!("Check if user exist");
    let user = match {
        let pool_swap = pool.clone();
        let body_swap = body.clone();
        async move {
            match User::get_one_by_mail(pool_swap.clone(), body_swap.email.clone()).await {
                Ok(user) => {
                    tracing::debug!(user = ?body_swap.email.clone() ,"User found");
                    Ok(user)
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.email.clone() ,"Error while getting user");
                    return Err(HttpResponse::Unauthorized().finish());
                }
            }
        }.instrument(check_user_span)
    }.await {
        Ok(user) => user,
        Err(err) => return err,
    };
    {
        let valid_password_span = tracing::info_span!("Check if password is valid");
        if let Err(err_response)=valid_password_span.in_scope(|| -> Result<_,HttpResponse> {
            match user.compare_password(body.password.clone()) {
                Ok(valid) => {
                    if !valid {
                        tracing::error!(user = ?body.email.clone() ,"Invalid password");
                        return Err(HttpResponse::Unauthorized().finish());
                    }
                    tracing::debug!(user = ?body.email.clone() ,"Password validated");
                    return Ok(());
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while validating password");
                    return Err(HttpResponse::Unauthorized().finish());
                }
            }
        }){
            return err_response;
        }
    }

    if user.otp_enabled {
        tracing::debug!(user = body.email, "User has otp enabled, sending otp");
        let mut user = user;
        tracing::debug!(user = body.email, "One time token generated");
        user.gen_one_time_token();
        let token = user.one_time_token.clone();

        {
            let pool_swap = pool.clone();
            let body_swap = body.clone();
            let save_new_one_time_token_span = tracing::info_span!("Save new one time token");
            if let Err(err_response) = async move{
                match user.update_otp_secret_url_token_enabled(pool_swap).await {
                    Ok(_) => {
                        tracing::debug!(user = ?body_swap.email.clone() ,"Successfuly saved one time token");
                        Ok(())
                    }
                    Err(err) => {
                        tracing::error!(error = ?err,user = ?body_swap.email.clone() ,"Error while saving one time token");
                        Err(HttpResponse::InternalServerError().finish())
                    }
                }
            }.instrument(save_new_one_time_token_span).await{
                return err_response;
            }
        }

        return HttpResponse::Ok().json(LoginUserReturn {
            status: LoginStatus::OtpStep,
            user: None,
            token,
        });
    }

    tracing::debug!(
        user = body.email,
        "User logged in, generating refresh_token"
    );

    let refresh_token = match TokenClaims::new_tokens(user.id, true) {
        Ok(token) => token,
        Err(err) => {
            tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while generating token");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let refresh_token_db = RefreshToken {
        created_at: chrono::Utc::now(),
        user_id: user.id,
        token: refresh_token.clone(),
    };

    {
        let pool_swap = pool.clone();
        let body_swap = body.clone();
        let delete_old_token_span = tracing::info_span!("Delete old refresh token");
        if let Err(err_response) = async move{
            match token::RefreshToken::keep_only_four_token(pool_swap, user.id).await {
                Ok(_) => {
                    tracing::debug!(user = ?body_swap.email.clone() ,"Successfully deleted old refresh token");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.email.clone() ,"Error while deleting old token");
                    Err(HttpResponse::InternalServerError().finish())
                }
            }
        }.instrument(delete_old_token_span).await{
            return err_response;
        }
    }

    {
        let pool_swap = pool.clone();
        let body_swap = body.clone();
        let insert_new_token_span = tracing::info_span!("Insert new refresh token");
        if let Err(err_response) = async move{
            match refresh_token_db.create(pool_swap.clone()).await {
                Ok(_) => {
                    tracing::debug!(user = ?body_swap.email.clone() ,"Refresh token saved to db");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.email.clone() ,"Error while saving refresh token to db");
                    return Err(HttpResponse::InternalServerError().finish());
                }
            }
        }.instrument(insert_new_token_span).await{
            return err_response;
        }
    }

    HttpResponse::Ok().json(LoginUserReturn {
        user: Some(user),
        status: LoginStatus::RefreshStep,
        token: Some(refresh_token),
    })
}

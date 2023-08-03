use crate::model::token::{self, RefreshToken, TokenClaims};
use crate::model::user::User;
use crate::route::auth::login::{LoginStatus, LoginUserReturn};
use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ValidateOtp {
    pub otp_code: String,
    pub one_time_token: String,
}

/// End the auth process
#[utoipa::path(
  tag = "Auth>Otp",
  request_body = ValidateOtp,
  operation_id = "validate",
  path = "/api/auth/otp/validate",
  responses(
      (status = 200, description = "Success", body = LoginUserReturn),
      (status = 400, description = "Bad request"),
      (status = 500, description = "Internal server error"),
  )
)]
#[post("/validate")]
pub async fn validate_otp(
    db_pool: web::Data<Pool>,
    activate_otp: web::Json<ValidateOtp>,
) -> impl Responder {
    let body = activate_otp.into_inner();
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    tracing::debug!(code = ?body.one_time_token ,"Starting to search user");
    let find_user_span = tracing::info_span!("Find user");
    let mut user = match {
        let pool_swap = pool.clone();
        let body_swap = body.clone();
        async move {
            match User::get_one_by_one_time_token(pool_swap.clone(), body_swap.one_time_token.clone()).await {
                Ok(user) => {
                    tracing::debug!(user = ?user.email.clone() ,"User found");
                    Ok(user)
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.one_time_token.clone() ,"Error while getting user");
                    Err(HttpResponse::Unauthorized().finish())
                }
            }
        }.instrument(find_user_span)
    }.await {
        Ok(user) => user,
        Err(err) => return err,
    };

    if !user.otp_enabled {
        tracing::debug!(user = ?user.email ,"Otp not enabled");
        return HttpResponse::BadRequest().finish();
    }
    if user.is_oauth {
        tracing::debug!(user = ?user.email ,"User is oauth");
        return HttpResponse::BadRequest().finish();
    }

    match user.validate_otp(body.otp_code.clone()) {
        Ok(status) => {
            if !status {
                tracing::debug!(user = ?user.email ,"User otp code is invalid");
                return HttpResponse::BadRequest().finish();
            }
        }
        Err(err) => {
            tracing::error!(error = ?err,user = ?user.email ,"Error while validating otp");
            return HttpResponse::InternalServerError().finish();
        }
    }

    user.one_time_token = None;
    let update_otp_span = tracing::info_span!("Update user otp");
    match {
        let pool_swap = pool.clone();
        let user_swap = user.clone();
        async move {
            match user_swap.update_otp_secret_url_token_enabled(pool_swap.clone()).await {
                Ok(_) => {
                    tracing::debug!(user = ?user_swap.email ,"User otp updated");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?user_swap.email ,"Error while updating user otp");
                    Err(HttpResponse::InternalServerError().finish())
                }
            }
        }.instrument(update_otp_span)
    }.await{
        Ok(_) => (),
        Err(err) => return err,
    }

    tracing::debug!(
        user = user.email.clone(),
        "User otp logged in, generating refresh_token"
    );

    let refresh_token = match TokenClaims::new_tokens(user.id, user.email.clone(), true) {
        Ok(token) => token,
        Err(err) => {
            tracing::error!(error = ?err,user = user.email.clone() ,"Error while generating token");
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
                    tracing::debug!(user = ?body_swap.one_time_token.clone() ,"Successfully deleted old refresh token");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.one_time_token.clone() ,"Error while deleting old token");
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
                    tracing::debug!(user = ?body_swap.one_time_token.clone() ,"Refresh token saved to db");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.one_time_token.clone() ,"Error while saving refresh token to db");
                    Err(HttpResponse::InternalServerError().finish())
                }
            }
        }.instrument(insert_new_token_span).await{
            return err_response;
        }
    }

    HttpResponse::Ok().json(LoginUserReturn {
        user: Some(user),
        token: Some(refresh_token),
        status: LoginStatus::RefreshStep,
    })
}

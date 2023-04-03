use crate::model::user::User;
use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ActivateOtp {
    pub otp_code: String,
}

/// End the totp activate process
#[utoipa::path(
  tag = "Auth>Otp",
  request_body = ActivateOtp,
  operation_id = "activate",
  path = "/api/auth/otp/activate",
  responses(
      (status = 200, description = "Success"),
      (status = 400, description = "Bad request"),
      (status = 500, description = "Internal server error"),
  ),
  security(
    ("access_token" = [])
  )
)]
#[post("/activate")]
pub async fn activate_otp(
    mut user: User,
    db_pool: web::Data<Pool>,
    activate_otp: web::Json<ActivateOtp>,
) -> impl Responder {
    tracing::debug!(user = ?user.email ,"User found, starting otp final activation");
    if user.otp_enabled {
        tracing::debug!(user = ?user.email ,"User already has otp enabled");
        return HttpResponse::BadRequest().finish();
    }
    let body = activate_otp.into_inner();
    match user.validate_otp(body.otp_code) {
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

    user.otp_enabled = true;
    let update_otp_span = tracing::info_span!("Update user otp");
    match {
        let pool_swap = db_pool.into_inner().as_ref().clone();
        let user_swap = user.clone();
        async move {
            match user_swap.update_otp_secret_url_enabled(pool_swap.clone()).await {
                Ok(_) => {
                    tracing::debug!(user = ?user_swap.email ,"User otp updated");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?user_swap.email ,"Error while updating user otp");
                    return Err(HttpResponse::InternalServerError().finish());
                }
            }
        }.instrument(update_otp_span)
    }.await{
        Ok(_) => (),
        Err(err) => return err,
    }

    HttpResponse::Ok().body("")
}

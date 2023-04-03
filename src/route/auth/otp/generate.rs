use crate::model::user::User;
use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct GenOtp {
    pub url: String,
    pub qr_code: String,
}

/// Start the totp activate process
#[utoipa::path(
  tag = "Auth>Otp",
  operation_id = "generate",
  path = "/api/auth/otp/activate",
  responses(
      (status = 200, description = "QrCode", body = GenOtp),
      (status = 400, description = "Bad request"),
      (status = 500, description = "Internal server error"),
  ),
  security(
    ("access_token" = [])
  )
)]
#[get("/activate")]
pub async fn generate_otp(mut user: User, db_pool: web::Data<Pool>) -> impl Responder {
    tracing::debug!(user = ?user.email ,"User found, starting otp generation");
    if user.otp_enabled {
        tracing::debug!(user = ?user.email ,"User already has otp enabled");
        return HttpResponse::BadRequest().finish();
    }
    user.gen_otp_secret();
    let otp_object = match user.get_totp_obj() {
        Ok(otp) => otp,
        Err(err) => {
            tracing::error!(error = ?err,user = ?user.email ,"Error while getting totp user");
            return HttpResponse::InternalServerError().finish();
        }
    };
    user.otp_url = Some(otp_object.get_url());
    let update_otp_span = tracing::info_span!("Update user otp");
    match {
        let pool_swap = db_pool.into_inner().as_ref().clone();
        let user_swap = user.clone();
        async move {
            match user_swap.update_otp_secret_url_token_enabled(pool_swap.clone()).await {
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

    HttpResponse::Ok().json(GenOtp {
        url: otp_object.get_url(),
        qr_code: otp_object.get_qr().unwrap(),
    })
}

use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::{
    token::{RefreshToken, TokenClaims},
    user::User,
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginUserReturn {
    pub user: User,
    pub refresh_token: String,
}

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
    let user = match User::get_one_by_mail(pool.clone(), body.email.clone()).await {
        Ok(user) => {
            tracing::debug!(user = ?body.email.clone() ,"User found");
            user
        }
        Err(err) => {
            tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while getting user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    match user.compare_password(body.password.clone()) {
        Ok(valid) => {
            if !valid {
                tracing::error!(user = ?body.email.clone() ,"Invalid password");
                return HttpResponse::Unauthorized().finish();
            }
            tracing::debug!(user = ?body.email.clone() ,"Password validated");
        }
        Err(err) => {
            tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while validating password");
            return HttpResponse::Unauthorized().finish();
        }
    }
    tracing::debug!(
        user = body.email,
        "User logged in, generating refresh_token"
    );

    let refresh_token = match TokenClaims::new_tokens(user.id, true, None) {
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

    match refresh_token_db.create(pool.clone()).await {
        Ok(_) => {
            tracing::debug!(user = ?body.email.clone() ,"Refresh token saved to db");
        }
        Err(err) => {
            tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while saving refresh token to db");
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().json(LoginUserReturn {
        user,
        refresh_token: refresh_token,
    })
}

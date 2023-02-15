use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::user::User;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginUserReturn {
    pub user: User,
    pub token: String,
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
    let user = match User::get_one_by_mail(pool, body.email.clone()).await {
        Ok(user) => user,
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
        }
        Err(err) => {
            tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while validating password");
            return HttpResponse::Unauthorized().finish();
        }
    }
    tracing::info!(user = body.email, "User logged in");

    HttpResponse::Ok().finish()
}

use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use utoipa::ToSchema;

use crate::model::user::User;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterUserReturn {
    pub created: bool,
    pub email: String,
    pub nom: String,
    pub prenom: String,
}

/// Get current user
///
/// Get current user based on the token
#[utoipa::path(
  tag = "User",
  operation_id = "getuser",
  path = "/api/user",
  responses(
      (status = 200, description = "User", body = User),
      (status = 400, description = "Error message"),
      (status = 500, description = "Internal server error"),
  ),
  security(
    ("access_token" = [])
  )
)]
#[get("")]
pub async fn get_current_user(user: User) -> impl Responder {
    tracing::debug!(user = ?user.email ,"User found");
    HttpResponse::Ok().json(user)
}

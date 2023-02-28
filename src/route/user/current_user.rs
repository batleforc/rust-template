use crate::model::user::User;
use actix_web::{get, web, HttpResponse, Responder};

/// Get current user
///
/// Get current user based on the token
#[utoipa::path(
  tag = "User",
  operation_id = "getuser",
  path = "/api/user/{id}",
  responses(
      (status = 200, description = "User", body = User),
      (status = 400, description = "Error message"),
      (status = 500, description = "Internal server error"),
  ),
  params(
    ("id" = uuid, Path, description = "Id de l'utilisateur"),
  ),
  security(
    ("access_token" = [])
  )
)]
#[get("/{id}")]
pub async fn get_current_user(user: User, uid_user: web::Path<uuid::Uuid>) -> impl Responder {
    tracing::debug!(user = ?user.email ,"User found");
    HttpResponse::Ok().json(user)
}

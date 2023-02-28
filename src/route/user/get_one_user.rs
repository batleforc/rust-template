use crate::model::user::{PublicUser, User};
use actix_web::{get, web, HttpResponse, Responder};

/// Get current user
///
/// Get current user based on the token
#[utoipa::path(
  tag = "User",
  operation_id = "getoneuser",
  path = "/api/user/{id}",
  responses(
      (status = 200, description = "User", body = PublicUser),
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
pub async fn get_one_user(user: User, uid_user: web::Path<uuid::Uuid>) -> impl Responder {
    let target_user_id = uid_user.into_inner();
    if user.id == target_user_id {
        tracing::debug!(user = ?user.email ,"User found, return public version of the current user");
        return HttpResponse::Ok().json(user.to_public_user());
    }
    tracing::debug!(user = ?user.email, uid = ?target_user_id ,"Searching for user");
    // @todo : Search for user in database following the tracing pattern
    HttpResponse::Ok().json(user.to_public_user())
}

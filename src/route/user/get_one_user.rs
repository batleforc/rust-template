use crate::model::user::User;
use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use tracing::Instrument;

/// Get one by uid user
///
/// Get one user by id
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
pub async fn get_one_user(
    user: User,
    uid_user: web::Path<uuid::Uuid>,
    db_pool: web::Data<Pool>,
) -> impl Responder {
    let target_user_id = uid_user.into_inner();
    if user.id == target_user_id {
        tracing::debug!(user = ?user.email ,"User found, return public version of the current user");
        return HttpResponse::Ok().json(user.to_public_user());
    }
    tracing::debug!(user = ?user.email, uid = ?target_user_id ,"Searching for user");
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    let find_user_span = tracing::info_span!("Find user");
    let user_find = match {
        async move {
            match User::get_one(pool.clone(), target_user_id).await {
                Ok(user) => {
                    tracing::debug!(user = ?user.email ,"User found");
                    Ok(user)
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?user.email ,"Error while getting user");
                    return Err(HttpResponse::NotFound().finish());
                }
            }
        }
        .instrument(find_user_span)
    }
    .await
    {
        Ok(selected_user) => selected_user,
        Err(err) => return err,
    };
    HttpResponse::Ok().json(user_find.to_public_user())
}

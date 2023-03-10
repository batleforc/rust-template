use crate::model::user::User;
use actix_web::{delete, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use tracing::Instrument;

/// Delete current user
///
/// Delete current user based on the token
#[utoipa::path(
  tag = "User",
  operation_id = "deleteuser",
  path = "/api/user",
  responses(
      (status = 200, description = "success"),
      (status = 400, description = "Error message"),
      (status = 500, description = "Internal server error"),
  ),
  security(
    ("access_token" = [])
  )
)]
#[delete("")]
pub async fn delete_user(user: User, db_pool: web::Data<Pool>) -> impl Responder {
    tracing::debug!(user = ?user.email, "Suprression de l'uttilisateur courant");
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    let delete_user_span = tracing::info_span!("Delete user and token");
    match {
        async move {
            let user_copy = user.clone();
            match user.delete(pool.clone()).await {
                Ok(edit) => {
                    tracing::debug!(user = ?user_copy.email,nbr_edit= ?edit ,"User found");
                    Ok(edit)
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?user_copy.email ,"Error while deleting user");
                    return Err(HttpResponse::NotFound().finish());
                }
            }
        }
        .instrument(delete_user_span)
    }
    .await
    {
        Ok(_) => (),
        Err(err) => return err,
    };
    HttpResponse::Ok().finish()
}

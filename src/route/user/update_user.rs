use crate::model::user::{User, UserUpdate};
use actix_web::{put, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use tracing::Instrument;

/// Update current user
///
/// Update current user based on the token
#[utoipa::path(
  tag = "User",
  operation_id = "updateuser",
  request_body = UserUpdate,
  path = "/api/user",
  responses(
      (status = 200, description = "success", body = PublicUser),
      (status = 400, description = "Error message"),
      (status = 500, description = "Internal server error"),
  ),
  security(
    ("access_token" = [])
  )
)]
#[put("")]
pub async fn update_user(
    mut user: User,
    db_pool: web::Data<Pool>,
    body: web::Json<UserUpdate>,
) -> impl Responder {
    tracing::debug!(user = ?user.email, "Update de l'uttilisateur courant");
    let value_to_update = body.into_inner();
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    if let Some(nom) = value_to_update.nom {
        user.nom = nom;
    }
    if let Some(prenom) = value_to_update.prenom {
        user.prenom = prenom;
    }
    let delete_user_span = tracing::info_span!("Update user");
    let usr = match {
        async move {
            let user_copy = user.clone();
            match user.update_name_surname(pool.clone()).await {
                Ok(edit) => {
                    tracing::debug!(user = ?user_copy.email,nbr_edit= ?edit ,"User updated");
                    Ok(user_copy)
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?user_copy.email ,"Error while updating user");
                    return Err(HttpResponse::NotFound().finish());
                }
            }
        }
        .instrument(delete_user_span)
    }
    .await
    {
        Ok(usr) => usr,
        Err(err) => return err,
    };
    HttpResponse::Ok().json(usr.to_public_user())
}

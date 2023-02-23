use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::Pool;
use tracing::Instrument;

use crate::{
    helper::header,
    model::token::{self, RefreshToken, TokenClaims},
};

/// Logout
///
/// This endpoint is used to disconnect the user with the refresh token
#[utoipa::path(
  tag = "Auth",
  operation_id = "logout",
  path = "/api/auth/logout",
  responses(
    (status = 200, description = "Logout", body = String)
  ),
  security(
    ("refresh_token" = [])
  )
)]
#[get("/logout")]
pub async fn logout(req: HttpRequest, db_pool: web::Data<Pool>) -> impl Responder {
    let get_token_span = tracing::info_span!("Get Token in header");
    let token = match get_token_span
        .in_scope(|| -> Result<&str, HttpResponse> { header::extract_authorization_header(&req) })
    {
        Ok(token) => token,
        Err(err) => return err,
    };
    drop(get_token_span);
    let check_token_span = tracing::info_span!("Check if token is valid");
    match check_token_span.in_scope(|| -> Result<TokenClaims, HttpResponse> {
        match token::TokenClaims::validate_token(token.to_string(), true) {
            Ok(claim) => Ok(claim),
            Err(err) => {
                tracing::error!(error = ?err, "Error while checking token");
                return Err(HttpResponse::Unauthorized()
                    .content_type(ContentType::plaintext())
                    .body("Invalid token"));
            }
        }
    }) {
        Ok(claim) => claim,
        Err(err) => return err,
    };
    drop(check_token_span);
    let pool: Pool = db_pool.clone().into_inner().as_ref().clone();
    {
        let check_refresh_token_span = tracing::info_span!("Check if refresh token exist");
        match async move {
            match RefreshToken::get_one_by_token(pool.clone(), token.to_string()).await {
                Ok(found_token) => {
                    tracing::debug!(token = ?found_token.token ,"Refresh token found");
                    Ok(found_token)
                }
                Err(err) => {
                    tracing::error!(error = ?err,token = ?token ,"Error while getting refresh token");
                    return Err(HttpResponse::Unauthorized().finish());
                }
            }
        }
        .instrument(check_refresh_token_span)
        .await
        {
            Ok(_) => {}
            Err(err) => return err,
        };
    }
    let delete_refresh_span = tracing::info_span!("Delete refresh token");
    let pool_delete: Pool = db_pool.into_inner().as_ref().clone();
    {
        match async move {
            match RefreshToken::delete_token(pool_delete.clone(), token.to_string()).await {
                Ok(_) => {
                    tracing::debug!(token = ?token ,"Refresh token deleted");
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(error = ?err,token = ?token ,"Error while deleting refresh token");
                    return Err(HttpResponse::InternalServerError().finish());
                }
            }
        }
        .instrument(delete_refresh_span.clone())
        .await
        {
            Ok(_) => {}
            Err(err) => return err,
        };
    }
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("User Disconnected")
}

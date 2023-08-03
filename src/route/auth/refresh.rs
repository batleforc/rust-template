use super::info::AuthType;
use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

use crate::{
    helper::header,
    model::token::{self, RefreshToken, TokenClaims},
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenReturn {
    pub access_token: String,
}

/// Refresh token
///
/// This endpoint is used to get access token from refresh token
#[utoipa::path(
  tag = "Auth",
  operation_id = "refresh",
  path = "/api/auth/refresh",
  responses(
    (status = 200, description = "Token body", body = RefreshTokenReturn)
  ),
  params(
    ("Authorization-type" = AuthType, Header, description = "Type de token (oidc ou buildin)")
  ),
  security(
    ("refresh_token" = [])
  )
)]
#[get("/refresh")]
pub async fn refresh(req: HttpRequest, db_pool: web::Data<Pool>) -> impl Responder {
    let get_token_span = tracing::info_span!("Get Token in header");
    let (token, auth_type) =
        match get_token_span.in_scope(|| -> Result<(&str, AuthType), HttpResponse> {
            header::extract_authorization_type_header(&req)
        }) {
            Ok(token) => token,
            Err(err) => return err,
        };
    drop(get_token_span);
    let check_token_span = tracing::info_span!("Check if token is valid");
    let mut claims = match check_token_span.in_scope(|| -> Result<TokenClaims, HttpResponse> {
        if auth_type == AuthType::Oidc {
            tracing::error!(token_type = ?auth_type.to_string(),"Invalid token type");
            return Err(HttpResponse::Unauthorized()
                .content_type(ContentType::plaintext())
                .body("Invalid token type"));
        }
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
    let pool: Pool = db_pool.into_inner().as_ref().clone();
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
                    Err(HttpResponse::Unauthorized().finish())
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

    let sign_token_span = tracing::info_span!("Sign access token");
    let new_token = match sign_token_span.in_scope(|| -> Result<String, HttpResponse> {
        claims.access_token();
        match claims.sign_token() {
            Ok(token) => Ok(token),
            Err(err) => {
                tracing::error!(error = ?err, "Error while signing token");
                Err(HttpResponse::InternalServerError().finish())
            }
        }
    }) {
        Ok(token) => token,
        Err(err) => return err,
    };
    drop(sign_token_span);

    HttpResponse::Ok().json(RefreshTokenReturn {
        access_token: new_token,
    })
}

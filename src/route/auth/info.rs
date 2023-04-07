use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct AuthStatus {
    pub enabled_protocol: Vec<AuthProtocol>,
    pub can_register: bool,
}

#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct AuthProtocol {
    pub type_auth: AuthType,
    pub name: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum AuthType {
    Oidc,
    BuildIn,
}

/// Return the auth status (and in the future include the oidc enabled and if main auth enabled)
#[utoipa::path(
  tag = "Auth",
  operation_id = "status",
  path = "/api/auth",
  responses(
      (status = 200, description = "Status", body = AuthStatus),
      (status = 400, description = "Bad request"),
      (status = 500, description = "Internal server error"),
  )
)]
#[get("")]
pub async fn auth_status() -> impl Responder {
    tracing::debug!("Asking for api auth status");
    HttpResponse::Ok().json(AuthStatus {
        can_register: true,
        enabled_protocol: vec![AuthProtocol {
            type_auth: AuthType::BuildIn,
            name: "Main".to_string(),
            icon: "".to_string(),
        }],
    })
}

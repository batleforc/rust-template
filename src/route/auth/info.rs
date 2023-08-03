use crate::model::oidc::{FrontOidc, Oidc};
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fmt;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_param: Option<FrontOidc>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, PartialEq, Debug)]
pub enum AuthType {
    Oidc,
    BuildIn,
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthType::Oidc => write!(f, "oidc"),
            AuthType::BuildIn => write!(f, "buildin"),
        }
    }
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
pub async fn auth_status(oidc_handler: web::Data<Oidc>) -> impl Responder {
    tracing::debug!("Asking for api auth status");
    let mut auth_possible = vec![AuthProtocol {
        type_auth: AuthType::BuildIn,
        name: "Main".to_string(),
        icon: "".to_string(),
        oidc_param: None,
    }];
    if !oidc_handler.oidc_disabled {
        auth_possible.push(AuthProtocol {
            type_auth: AuthType::Oidc,
            name: "Oidc".to_string(),
            icon: "".to_string(),
            oidc_param: oidc_handler.front.clone(),
        });
    }
    HttpResponse::Ok().json(AuthStatus {
        can_register: true,
        enabled_protocol: auth_possible,
    })
}

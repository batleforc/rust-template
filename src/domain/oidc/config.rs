use std::env::VarError;

use super::{front::FrontOidc, oidc::OidcHandler};

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub back: Option<OidcHandler>,
    pub front: Option<FrontOidc>,
    pub oidc_disabled: bool,
}

impl OidcConfig {
    pub fn new() -> Result<OidcConfig, VarError> {
        let span = tracing::span!(tracing::Level::DEBUG, "Oidc::new");
        let _enter = span.enter();
        let back = OidcConfig::new_back()?;
        let front = OidcConfig::new_front()?;
        let oidc = OidcConfig {
            back: Some(back),
            front: Some(front),
            oidc_disabled: false,
        };
        tracing::info!("OIDC is enabled");
        Ok(oidc)
    }
    pub fn new_disable() -> OidcConfig {
        let span = tracing::span!(tracing::Level::DEBUG, "Oidc::new_disable");
        let _enter = span.enter();
        tracing::warn!("OIDC is disabled");
        OidcConfig {
            back: None,
            front: None,
            oidc_disabled: true,
        }
    }
    pub fn new_back() -> Result<OidcHandler, VarError> {
        let span = tracing::span!(tracing::Level::DEBUG, "OidcBackend::new");
        let _enter = span.enter();
        let client_id = std::env::var("OIDC_CLIENT_ID").unwrap();
        let client_secret = std::env::var("OIDC_CLIENT_SECRET").unwrap();
        let issuer = std::env::var("OIDC_ISSUER").unwrap();
        let redirect_uri = std::env::var("OIDC_REDIRECT_URI").unwrap();
        let scopes = std::env::var("OIDC_SCOPES").unwrap();
        let userinfo_url = std::env::var("OIDC_USERINFO_URL").unwrap();
        let introspection_url = std::env::var("OIDC_INTROSPECTION_URL").unwrap();
        let key_id = std::env::var("OIDC_KEY_ID").unwrap();
        let client_assertion_type =
            std::env::var("OIDC_CLIENT_ASSERTION_TYPE").unwrap_or_else(|_| {
                "urn:ietf:params:oauth:client-assertion-type:jwt-bearer".to_string()
            });
        tracing::trace!("OidcBackend created");
        Ok(OidcHandler {
            client_id,
            client_secret,
            issuer,
            redirect_uri,
            scopes,
            userinfo_url,
            introspection_url,
            key_id,
            client_assertion_type,
        })
    }

    pub fn new_front() -> Result<FrontOidc, VarError> {
        let span = tracing::span!(tracing::Level::DEBUG, "OidcFrontend::new");
        let _enter = span.enter();
        let client_id = std::env::var("OIDC_FRONT_CLIENT_ID").unwrap();
        let token_url = std::env::var("OIDC_FRONT_TOKEN_URL").unwrap();
        let auth_url = std::env::var("OIDC_FRONT_AUTH_URL").unwrap();
        let issuer = std::env::var("OIDC_FRONT_ISSUER").unwrap();
        let scopes = std::env::var("OIDC_FRONT_SCOPES").unwrap();
        let redirect_uri = std::env::var("OIDC_REDIRECT_URI").unwrap();
        tracing::trace!("OidcFrontend created");
        Ok(FrontOidc {
            client_id,
            token_url,
            auth_url,
            issuer,
            scopes,
            redirect_uri,
        })
    }
}

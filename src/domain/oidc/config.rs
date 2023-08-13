use std::env::VarError;

use super::{front::FrontOidc, oidc::OidcHandler};

#[derive(Clone, Debug)]
pub struct Oidc {
    pub back: Option<OidcHandler>,
    pub front: Option<FrontOidc>,
    pub oidc_disabled: bool,
}

impl Oidc {
    pub fn new() -> Result<Oidc, VarError> {
        let back = Oidc::new_back()?;
        let front = Oidc::new_front()?;
        let oidc = Oidc {
            back: Some(back),
            front: Some(front),
            oidc_disabled: false,
        };
        Ok(oidc)
    }
    pub fn new_disable() -> Oidc {
        Oidc {
            back: None,
            front: None,
            oidc_disabled: true,
        }
    }
    pub fn new_back() -> Result<OidcHandler, VarError> {
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
        let client_id = std::env::var("OIDC_FRONT_CLIENT_ID").unwrap();
        let token_url = std::env::var("OIDC_FRONT_TOKEN_URL").unwrap();
        let auth_url = std::env::var("OIDC_FRONT_AUTH_URL").unwrap();
        let issuer = std::env::var("OIDC_FRONT_ISSUER").unwrap();
        let scopes = std::env::var("OIDC_FRONT_SCOPES").unwrap();
        let redirect_uri = std::env::var("OIDC_REDIRECT_URI").unwrap();
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

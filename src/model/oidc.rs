use super::oidc_token::OidcTokenClaim;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env::VarError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BackOidc {
    pub client_id: String,
    pub client_secret: String,
    pub issuer: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub userinfo_url: String,
    pub introspection_url: String,
    pub key_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FrontOidc {
    pub client_id: String,
    pub token_url: String,
    pub auth_url: String,
    pub issuer: String,
    pub scopes: String,
    pub redirect_uri: String,
}

#[derive(Clone, Debug)]
pub struct Oidc {
    pub back: Option<BackOidc>,
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
    pub fn new_back() -> Result<BackOidc, VarError> {
        let client_id = std::env::var("OIDC_CLIENT_ID").unwrap();
        let client_secret = std::env::var("OIDC_CLIENT_SECRET").unwrap();
        let issuer = std::env::var("OIDC_ISSUER").unwrap();
        let redirect_uri = std::env::var("OIDC_REDIRECT_URI").unwrap();
        let scopes = std::env::var("OIDC_SCOPES").unwrap();
        let userinfo_url = std::env::var("OIDC_USERINFO_URL").unwrap();
        let introspection_url = std::env::var("OIDC_INTROSPECTION_URL").unwrap();
        let key_id = std::env::var("OIDC_KEY_ID").unwrap();
        Ok(BackOidc {
            client_id,
            client_secret,
            issuer,
            redirect_uri,
            scopes,
            userinfo_url,
            introspection_url,
            key_id,
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

impl BackOidc {
    pub async fn validate_token(self, token: String) -> Result<bool, reqwest::Error> {
        let client = Client::new();
        let mut oidc_token = OidcTokenClaim::new(self.client_id.clone(), self.issuer.clone());
        let token_oidc =
            match oidc_token.sign_token(self.key_id.clone(), self.client_secret.clone()) {
                Ok(token) => token,
                Err(e) => {
                    println!("Error: {}", e);
                    return Ok(false);
                }
            };
        let res = client
            .post(&self.introspection_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("token", token),
                (
                    "client_assertion_type",
                    "urn:ietf:params:oauth:client-assertion-type:jwt-bearer".to_string(),
                ),
                ("client_assertion", token_oidc),
            ])
            .send()
            .await?;
        let status = res.status();
        let body = res.text().await?;
        println!("body: {}", body);
        if status != 200 {
            println!("Error: {}", status);
            return Ok(false);
        }

        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let active = json["active"].as_bool().unwrap();
        Ok(active)
    }
}

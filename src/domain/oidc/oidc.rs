use reqwest::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::jwt::OidcTokenClaim;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OidcHandler {
    pub client_id: String,
    pub client_secret: String,
    pub issuer: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub userinfo_url: String,
    pub introspection_url: String,
    pub key_id: String,
    pub client_assertion_type: String,
}

impl OidcHandler {
    pub async fn validate_token(
        self,
        token: String,
    ) -> Result<(bool, serde_json::Value), reqwest::Error> {
        let client = Client::new();
        let mut oidc_token = OidcTokenClaim::new(self.client_id.clone(), self.issuer.clone());
        let token_oidc =
            match oidc_token.sign_token(self.key_id.clone(), self.client_secret.clone()) {
                Ok(token) => token,
                Err(e) => {
                    println!("Error: {}", e);
                    return Ok((false, serde_json::Value::Null));
                }
            };
        let res = client
            .post(&self.introspection_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("token", token),
                ("client_assertion_type", self.client_assertion_type.clone()),
                ("client_assertion", token_oidc),
            ])
            .send()
            .await?;
        let status = res.status();
        if status != 200 {
            return Ok((false, serde_json::Value::Null));
        }
        let body = res.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let active = json["active"].as_bool().unwrap();
        Ok((active, json))
    }

    pub async fn get_user_info(self, token: String) -> Result<serde_json::Value, reqwest::Error> {
        let client = Client::new();
        let res = client
            .get(&self.userinfo_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        let status = res.status();
        if status != 200 {
            return Ok(serde_json::Value::Null);
        }
        let body = res.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        Ok(json)
    }
}

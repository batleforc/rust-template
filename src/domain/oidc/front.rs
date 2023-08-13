use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct FrontOidc {
    pub client_id: String,
    pub token_url: String,
    pub auth_url: String,
    pub issuer: String,
    pub scopes: String,
    pub redirect_uri: String,
}

impl FrontOidc {
    pub fn get_scope(&self) -> Vec<String> {
        self.scopes.split(' ').map(|s| s.to_string()).collect()
    }
}

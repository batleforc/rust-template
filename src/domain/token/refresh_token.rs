use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::Auth, domain::Entity};

use super::token::{self, TokenClaims, TokenError};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RefreshToken {
    pub token: String,
    pub email: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Entity for RefreshToken {}

impl RefreshToken {
    pub fn new(token: String, email: String, user_id: Uuid) -> Self {
        Self {
            token,
            email,
            user_id,
            created_at: Utc::now(),
        }
    }
    pub fn create_token(
        uid: Uuid,
        email: String,
        app_name: String,
        auth_config: Auth,
    ) -> Result<RefreshToken, TokenError> {
        let mut token_claim = TokenClaims::new(uid, email.clone(), app_name, true);
        match token_claim.sign_token(auth_config) {
            Ok(token) => Ok(RefreshToken::new(token, email, uid)),
            Err(e) => Err(e),
        }
    }
}

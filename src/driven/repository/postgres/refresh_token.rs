use super::super::SearchEntity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::refresh_token::RefreshToken;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchRefreshToken {
    pub token: Option<String>,
    pub email: Option<String>,
    pub created_before: Option<DateTime<Utc>>,
}

impl SearchEntity for SearchRefreshToken {}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPG {
    pub token: String,
    pub email: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<RefreshToken> for RefreshTokenPG {
    fn from(value: RefreshToken) -> Self {
        RefreshTokenPG {
            token: value.token,
            email: value.email,
            user_id: value.user_id,
            created_at: value.created_at,
        }
    }
}

impl TryInto<RefreshToken> for RefreshTokenPG {
    type Error = String;

    fn try_into(self) -> Result<RefreshToken, Self::Error> {
        Ok(RefreshToken {
            token: self.token,
            email: self.email,
            user_id: self.user_id,
            created_at: self.created_at,
        })
    }
}

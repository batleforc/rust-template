use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Entity;

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
}

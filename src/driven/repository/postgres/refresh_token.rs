use super::super::SearchEntity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::token::refresh_token::RefreshToken;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchRefreshToken {
    pub user_id: Option<Uuid>,
    pub token: Option<String>,
    pub email: Option<String>,
    pub created_before: Option<DateTime<Utc>>,
}

impl SearchEntity for SearchRefreshToken {}

impl SearchRefreshToken {
    pub fn turn_into_search(&self) -> Result<(String, Vec<String>), ()> {
        let mut query = String::new();
        let mut params: Vec<String> = Vec::new();
        let mut index = 1;
        if let Some(user_id) = &self.user_id {
            query.push_str("user_id LIKE $");
            query.push_str(&index.to_string());
            params.push(user_id.to_string());
            index += 1;
        }
        if let Some(token) = &self.token {
            if !query.is_empty() {
                query.push_str(" AND ");
            }
            query.push_str("token LIKE $");
            query.push_str(&index.to_string());
            params.push(token.to_string());
            index += 1;
        }
        if let Some(email) = &self.email {
            if !query.is_empty() {
                query.push_str(" AND ");
            }
            query.push_str("email LIKE $");
            query.push_str(&index.to_string());
            params.push(email.to_string());
            index += 1;
        }
        if let Some(created_before) = &self.created_before {
            if !query.is_empty() {
                query.push_str(" AND ");
            }
            query.push_str("created_at > $");
            query.push_str(&index.to_string());
            params.push(created_before.to_string());
        }
        if query.is_empty() {
            return Err(());
        }
        Ok((query, params))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPG {
    pub token: String,
    pub email: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl RefreshTokenPG {
    pub fn from_row(row: &tokio_postgres::Row) -> Self {
        RefreshTokenPG {
            token: row.get("token"),
            email: row.get("email"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
        }
    }
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

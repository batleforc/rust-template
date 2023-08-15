use super::super::SearchEntity;
use crate::domain::user::user::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUser {
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub one_time_token: Option<String>,
    pub surname: Option<String>,
    pub name: Option<String>,
}

impl SearchEntity for SearchUser {}

impl SearchUser {
    pub fn turn_into_search(&self) -> Result<(String, Vec<String>), ()> {
        let mut query = String::new();
        let mut params: Vec<String> = Vec::new();
        let mut index = 1;
        if let Some(id) = &self.id {
            query.push_str("id LIKE $");
            query.push_str(&index.to_string());
            params.push(id.to_string());
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
        if let Some(one_time_token) = &self.one_time_token {
            if !query.is_empty() {
                query.push_str(" AND ");
            }
            query.push_str("one_time_token LIKE $");
            query.push_str(&index.to_string());
            params.push(one_time_token.to_string());
            index += 1;
        }
        if let Some(surname) = &self.surname {
            if !query.is_empty() {
                query.push_str(" AND ");
            }
            query.push_str("surname LIKE $");
            query.push_str(&index.to_string());
            params.push(surname.to_string());
            index += 1;
        }
        if let Some(name) = &self.name {
            if !query.is_empty() {
                query.push_str(" AND ");
            }
            query.push_str("name LIKE $");
            query.push_str(&index.to_string());
            params.push(name.to_string());
        }
        if query.is_empty() {
            return Err(());
        }
        Ok((query, params))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPG {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub surname: String,
    pub name: String,
    pub otp_secret: Option<String>,
    pub otp_url: Option<String>,
    pub otp_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub one_time_token: Option<String>,
    pub is_oauth: bool,
}

impl From<User> for UserPG {
    fn from(value: User) -> Self {
        UserPG {
            id: value.id,
            email: value.email,
            password: value.password,
            surname: value.surname,
            name: value.name,
            otp_secret: value.otp_secret,
            otp_url: value.otp_url,
            otp_enabled: value.otp_enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
            is_oauth: value.is_oauth,
            one_time_token: value.one_time_token,
        }
    }
}

impl UserPG {
    pub fn from_row(row: &tokio_postgres::Row) -> Self {
        UserPG {
            id: row.get("id"),
            email: row.get("email"),
            password: row.get("password"),
            surname: row.get("surname"),
            name: row.get("name"),
            otp_secret: row.get("otp_secret"),
            otp_url: row.get("otp_url"),
            otp_enabled: row.get("otp_enabled"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_oauth: row.get("is_oauth"),
            one_time_token: row.get("one_time_token"),
        }
    }
}

impl TryInto<User> for UserPG {
    type Error = String;

    fn try_into(self) -> Result<User, Self::Error> {
        Ok(User {
            id: self.id,
            email: self.email,
            password: self.password,
            surname: self.surname,
            name: self.name,
            otp_secret: self.otp_secret,
            otp_url: self.otp_url,
            otp_enabled: self.otp_enabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_oauth: self.is_oauth,
            one_time_token: self.one_time_token,
        })
    }
}

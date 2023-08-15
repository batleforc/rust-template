use super::{
    super::Entity,
    otp::{Totp, TotpError},
    password::{Password, PasswordError},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub surname: String,
    pub name: String,
}

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub surname: String,
    pub name: String,
    pub otp_secret: Option<String>,
    pub otp_url: Option<String>,
    pub otp_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub one_time_token: Option<String>,
    pub is_oauth: bool,
}

impl Entity for User {}

impl User {
    pub fn new(email: String, nom: String, prenom: String, is_oauth: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password: "".to_string(),
            surname: nom,
            name: prenom,
            otp_secret: None,
            otp_url: None,
            otp_enabled: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            one_time_token: None,
            is_oauth,
        }
    }
    pub fn get_surname(&self) -> String {
        self.surname.clone()
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_email(&self) -> String {
        self.email.clone()
    }
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    pub fn get_otp_secret(&self) -> Option<String> {
        self.otp_secret.clone()
    }
    pub fn get_otp_url(&self) -> Option<String> {
        self.otp_url.clone()
    }
    pub fn get_otp_enabled(&self) -> bool {
        self.otp_enabled
    }
    pub fn get_created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    pub fn get_updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
    pub fn get_one_time_token(&self) -> Option<String> {
        self.one_time_token.clone()
    }
    pub fn get_is_oauth(&self) -> bool {
        self.is_oauth
    }
    pub fn validate_password(&self, password: String) -> Result<bool, PasswordError> {
        Password::verify(password, self.password.clone())
    }
    pub fn update_password(&mut self, password: String) -> Result<(), PasswordError> {
        match Password::hash(password) {
            Ok(h) => {
                self.password = h;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    pub fn gen_otp_secret(&mut self) -> Result<(), TotpError> {
        match Totp::gen_otp_secret() {
            Ok(secret) => {
                self.otp_secret = Some(secret);
                self.set_otp_enabled(true);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    pub fn create_otp_url(&mut self, app_name: String) -> Result<(), TotpError> {
        if let Some(otp_secret) = self.get_otp_secret() {
            return match Totp::get_otp_url(self.get_email(), otp_secret.clone(), app_name) {
                Ok(url) => {
                    self.otp_url = Some(url);
                    Ok(())
                }
                Err(err) => Err(TotpError::InvalidSecret(err.to_string())),
            };
        }
        Err(TotpError::InvalidSecret("No secret defined".to_string()))
    }
    pub fn set_otp_enabled(&mut self, otp_enabled: bool) {
        self.otp_enabled = otp_enabled;
    }
    pub fn set_updated_at(&mut self, updated_at: chrono::DateTime<chrono::Utc>) {
        self.updated_at = updated_at;
    }
    pub fn set_one_time_token(&mut self, one_time_token: String) {
        self.one_time_token = Some(one_time_token);
    }
    pub fn set_is_oauth(&mut self, is_oauth: bool) {
        self.is_oauth = is_oauth;
    }
    pub fn set_nom(&mut self, nom: String) {
        self.surname = nom;
    }
    pub fn set_prenom(&mut self, prenom: String) {
        self.name = prenom;
    }
}

impl TryInto<PublicUser> for User {
    type Error = ();

    fn try_into(self) -> Result<PublicUser, Self::Error> {
        Ok(PublicUser {
            id: self.id,
            email: self.email,
            surname: self.surname,
            name: self.name,
        })
    }
}
#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;

    fn create_user() -> (User, Uuid, chrono::DateTime<chrono::Utc>) {
        let now = chrono::Utc::now();
        let id = Uuid::new_v4();
        let user = User {
            id,
            email: "john@doe.net".to_string(),
            password: "password".to_string(),
            name: "John".to_string(),
            surname: "Doe".to_string(),
            otp_secret: None,
            otp_url: None,
            otp_enabled: false,
            created_at: now,
            updated_at: now,
            one_time_token: None,
            is_oauth: false,
        };
        (user, id, now)
    }

    #[test]
    fn test_user_creation() {
        let (user, id, now) = create_user();
        assert_eq!(user.get_name(), "John");
        assert_eq!(user.get_surname(), "Doe");
        assert_eq!(user.get_email(), "john@doe.net");
        assert_eq!(user.get_id(), id);
        assert_eq!(user.get_otp_secret(), None);
        assert_eq!(user.get_otp_url(), None);
        assert_eq!(user.get_otp_enabled(), false);
        assert_eq!(user.get_created_at(), now);
        assert_eq!(user.get_updated_at(), now);
        assert_eq!(user.get_one_time_token(), None);
        assert_eq!(user.get_is_oauth(), false);
    }

    #[test]
    fn test_user_password_validation() {
        let (mut user, _, _) = create_user();
        assert_eq!(
            user.validate_password("password".to_string()),
            Err(PasswordError::HashEngineError(
                "Invalid hash: password".to_string()
            ))
        );
        user.update_password("XJEa1dUVLh6".to_string()).unwrap();
        assert_eq!(user.validate_password("XJEa1dUVLh6".to_string()), Ok(true));
        assert_eq!(user.validate_password("wrong".to_string()), Ok(false));
    }

    #[test]
    fn test_assign() {
        let (mut user, _, _) = create_user();
        let now_2 = chrono::Utc::now().add(chrono::Duration::days(1));
        user.set_nom("Doe".to_string());
        user.set_prenom("John".to_string());
        user.set_updated_at(now_2);
        user.set_one_time_token("token".to_string());
        user.set_is_oauth(true);
        user.set_otp_enabled(true);

        assert_eq!(user.get_surname(), "Doe");
        assert_eq!(user.get_name(), "John");
        assert_eq!(user.get_updated_at().timestamp(), now_2.timestamp());
        assert_eq!(user.get_one_time_token(), Some("token".to_string()));
        assert_eq!(user.get_is_oauth(), true);
        assert_eq!(user.get_otp_enabled(), true);
    }
}

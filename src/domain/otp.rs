use std::{default, fmt::Display};

use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Debug, PartialEq)]
pub enum TotpError {
    InvalidSecret(String),
    ValidateSecret(String),
}

impl Display for TotpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TotpError::InvalidSecret(msg) => write!(f, "Invalid secret: {}", msg),
            TotpError::ValidateSecret(msg) => write!(f, "Validate secret: {}", msg),
            _ => write!(f, "Totp error"),
        }
    }
}

pub struct Totp {}

impl Totp {
    pub fn get_totp_obj(
        email: String,
        secret: String,
        app_name: String,
    ) -> Result<TOTP, TotpError> {
        let secret = Secret::Encoded(secret);
        let secret_bytes = match secret.to_bytes() {
            Ok(s) => s,
            Err(err) => return Err(TotpError::InvalidSecret(err.to_string())),
        };
        match TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some(app_name),
            email,
        ) {
            Ok(t) => Ok(t),
            Err(err) => Err(TotpError::InvalidSecret(err.to_string())),
        }
    }
    pub fn gen_otp_secret() -> Result<String, TotpError> {
        let secret = Secret::generate_secret();
        Ok(secret.to_encoded().to_string())
    }
    pub fn validate_otp(
        otp: String,
        email: String,
        app_name: String,
        secret: String,
    ) -> Result<bool, TotpError> {
        let totp = match Totp::get_totp_obj(email, secret, app_name) {
            Ok(t) => t,
            Err(err) => return Err(TotpError::ValidateSecret(err.to_string())),
        };
        match totp.check_current(&otp) {
            Ok(v) => Ok(v),
            Err(err) => Err(TotpError::ValidateSecret(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn test_validate_otp() {
        let email = String::from("test@test.mdx");
        let app_name = String::from("rust-template");
        let secret = String::from("IQSTDLOJVO3DDMFM4E5XSONQTQMSH2JI");
        let otp = Totp::get_totp_obj(email.clone(), secret.clone(), app_name.clone()).unwrap();
        let time: u64 = Utc
            .with_ymd_and_hms(2023, 8, 3, 23, 09, 11)
            .unwrap()
            .timestamp()
            .try_into()
            .unwrap();
        let otp_current = otp.generate_current().unwrap();
        let mut validate = Totp::validate_otp(
            otp_current.clone(),
            email.clone(),
            app_name.clone(),
            secret.clone(),
        );
        assert_eq!(validate, Ok(true));
        let otp_code = otp.generate(time);
        assert_eq!(otp_code, "349445".to_string());

        let result = otp.check(&otp_code, time);
        assert_eq!(result, true);

        validate = Totp::validate_otp(
            "349446".to_string(),
            email.clone(),
            app_name.clone(),
            secret.clone(),
        );
        assert_eq!(validate, Ok(false));
    }
}

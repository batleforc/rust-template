use std::fmt::Display;

use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug, PartialEq)]
pub enum PasswordError {
    HashEngineError(String),
}

impl Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::HashEngineError(msg) => write!(f, "Hash engine error: {}", msg),
        }
    }
}

pub struct Password {}

impl Password {
    pub fn hash(password: String) -> Result<String, PasswordError> {
        let span = tracing::span!(tracing::Level::DEBUG, "PASSWORD::hash");
        let _enter = span.enter();
        match hash(password, DEFAULT_COST) {
            Ok(h) => Ok(h),
            Err(err) => {
                tracing::error!("Hash engine error: {}", err);
                Err(PasswordError::HashEngineError(err.to_string()))
            }
        }
    }
    pub fn verify(password: String, hash: String) -> Result<bool, PasswordError> {
        let span = tracing::span!(tracing::Level::DEBUG, "PASSWORD::verify");
        let _enter = span.enter();
        match verify(password, &hash) {
            Ok(h) => Ok(h),
            Err(err) => {
                tracing::error!("Hash engine error: {}", err);
                Err(PasswordError::HashEngineError(err.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let password = String::from("password");
        let hashed_password = Password::hash(password.clone()).unwrap();
        assert_ne!(password, hashed_password);
    }

    #[test]
    fn test_verify() {
        let password = String::from("password");
        let hashed_password = Password::hash(password.clone()).unwrap();
        assert!(Password::verify(password, hashed_password).unwrap());
    }
}

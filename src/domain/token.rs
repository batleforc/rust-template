use std::env;

use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum TokenError {
    InvalidSignToken(String),
    InvalidToken(String),
    WrongTokenType(String),
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::InvalidSignToken(msg) => write!(f, "Invalid sign token: {}", msg),
            TokenError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            TokenError::WrongTokenType(msg) => write!(f, "Wrong token type: {}", msg),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
pub struct TokenClaims {
    pub sub: Uuid,     // subject
    pub email: String, // email
    pub exp: usize,    // expiration
    pub iat: usize,    // issued at
    pub iss: String,   // issuer
    pub refresh: bool, // is refresh token
}

impl TokenClaims {
    pub fn new(sub: Uuid, email: String, iss: String, refresh: bool) -> Self {
        let iat = chrono::Utc::now();
        let exp = match refresh {
            true => iat + chrono::Duration::hours(1),
            false => iat + chrono::Duration::days(7),
        };
        Self {
            sub,
            email,
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
            iss,
            refresh,
        }
    }
    fn gen_header(refresh: bool) -> Header {
        let kid = match refresh {
            true => "refresh_token",
            false => "access_token",
        };
        Header {
            alg: Algorithm::HS512,
            kid: Some(kid.to_string()),
            ..Default::default()
        }
    }
    pub fn get_key(refresh: bool) -> String {
        if refresh {
            match env::var("REFRESH_TOKEN_SIGN") {
                Ok(val) => val,
                Err(_) => "lambda_refresh_token_sign".to_string(),
            }
        } else {
            match env::var("ACCESS_TOKEN_SIGN") {
                Ok(val) => val,
                Err(_) => "lambda_token_sign".to_string(),
            }
        }
    }
    pub fn sign_token(&mut self) -> Result<String, TokenError> {
        let header = Self::gen_header(self.refresh);
        let key_string = Self::get_key(self.refresh);
        let key = key_string.as_bytes();
        match encode(&header, self, &EncodingKey::from_secret(key)) {
            Ok(token) => Ok(token),
            Err(err) => Err(TokenError::InvalidSignToken(err.to_string())),
        }
    }

    pub fn validate_token(token: String, refresh: bool) -> Result<Self, TokenError> {
        let key_string = Self::get_key(refresh);
        let key = key_string.as_bytes();
        match jsonwebtoken::decode::<TokenClaims>(
            &token,
            &DecodingKey::from_secret(key),
            &Validation::new(Algorithm::HS512),
        ) {
            Ok(token_data) => {
                if token_data.claims.refresh != refresh {
                    return Err(TokenError::WrongTokenType(
                        "Token type does not match".to_string(),
                    ));
                }

                Ok(token_data.claims)
            }
            Err(err) => Err(TokenError::InvalidToken(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_default_token_claims(
        sub: Uuid,
        email: String,
        iss: String,
        refresh: bool,
    ) -> (TokenClaims, usize, usize) {
        let token = TokenClaims::new(sub, email, iss, refresh);
        (token.clone(), token.iat, token.exp)
    }

    #[test]
    fn test_token_validate_and_content() {
        let sub = Uuid::new_v4();
        let email = "joseph@joestar.com".to_string();
        let iss = "lambda".to_string();

        for refresh in vec![true, false].iter() {
            let (mut token_claims, _, _) =
                init_default_token_claims(sub, email.clone(), iss.clone(), *refresh);
            let token = match token_claims.sign_token() {
                Ok(token) => {
                    assert!(true);
                    token
                }
                Err(_) => {
                    panic!("Failed to sign token")
                }
            };
            let token_claims = match TokenClaims::validate_token(token, *refresh) {
                Ok(token_claims) => {
                    assert!(true);
                    token_claims
                }
                Err(_) => {
                    panic!("Failed to validate token")
                }
            };
            assert_eq!(token_claims.sub, sub);
            assert_eq!(token_claims.email, email);
            assert_eq!(token_claims.iss, iss);
            assert_eq!(token_claims.refresh, *refresh);
            match *refresh {
                true => {
                    assert_eq!(token_claims.exp, token_claims.iat + 3600)
                }
                false => {
                    assert_eq!(token_claims.exp, token_claims.iat + 3600 * 24 * 7)
                }
            }
        }
    }
}

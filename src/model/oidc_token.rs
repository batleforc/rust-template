use jsonwebtoken::{Algorithm, Header};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct OidcTokenClaim {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

impl OidcTokenClaim {
    pub fn new(client_id: String, issuer: String) -> OidcTokenClaim {
        let exp = chrono::Utc::now() + chrono::Duration::hours(1);
        let iat = chrono::Utc::now();
        OidcTokenClaim {
            sub: client_id.clone(),
            iss: client_id,
            aud: issuer,
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
        }
    }

    pub fn new_header(key_id: String) -> Header {
        Header {
            alg: Algorithm::RS256,
            kid: Some(key_id),
            ..Default::default()
        }
    }

    pub fn sign_token(&mut self, key_id: String, private_key: String) -> Result<String, String> {
        let header = OidcTokenClaim::new_header(key_id);
        match jsonwebtoken::encode(
            &header,
            self,
            &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key.as_bytes()).unwrap(),
        ) {
            Ok(token) => Ok(token),
            Err(e) => Err(e.to_string()),
        }
    }
}

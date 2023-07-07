use std::env;

use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: uuid::Uuid, // subject
    pub email: String,   // email
    pub exp: usize,      // expiration
    pub iat: usize,      // issued at
    pub iss: String,     // issuer
    pub refresh: bool,   // is refresh token
}

impl TokenClaims {
    pub fn new_token_claims(user_id: uuid::Uuid, email: String, refresh: bool) -> TokenClaims {
        let mut exp = chrono::Utc::now() + chrono::Duration::hours(1);
        if refresh {
            exp = chrono::Utc::now() + chrono::Duration::days(7);
        }
        TokenClaims {
            sub: user_id,
            email,
            exp: exp.timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            iss: "Rust_api".to_string(),
            refresh,
        }
    }
    pub fn access_token(&mut self) {
        self.exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;
        self.iat = chrono::Utc::now().timestamp() as usize;
        self.refresh = false;
    }
    fn new_header(refresh: bool) -> Header {
        let kid_key = if refresh {
            "refresh_token"
        } else {
            "access_token"
        };
        Header {
            alg: Algorithm::HS512,
            kid: Some(kid_key.to_string()),
            ..Default::default()
        }
    }
    fn get_key(refresh: bool) -> String {
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
    pub fn new_tokens(user_id: uuid::Uuid, email: String, refresh: bool) -> Result<String, String> {
        let mut claims = TokenClaims::new_token_claims(user_id, email, refresh);
        claims.sign_token()
    }
    pub fn sign_token(&mut self) -> Result<String, String> {
        let header = TokenClaims::new_header(self.refresh);
        let key_string = TokenClaims::get_key(self.refresh);
        let key = key_string.as_bytes();
        match encode(&header, &self, &EncodingKey::from_secret(key)) {
            Ok(token) => Ok(token),
            Err(_) => Err("Error while creating token".to_string()),
        }
    }
    pub fn validate_token(token: String, refresh: bool) -> Result<TokenClaims, String> {
        let key_string = TokenClaims::get_key(refresh);
        let key = key_string.as_bytes();
        match jsonwebtoken::decode::<TokenClaims>(
            &token,
            &DecodingKey::from_secret(key),
            &Validation::new(Algorithm::HS512),
        ) {
            Ok(token_data) => {
                if token_data.claims.refresh != refresh {
                    return Err("Token is not valid".to_string());
                }

                Ok(token_data.claims)
            }
            Err(err) => Err(format!("Error while validating token {}", err)),
        }
    }
}

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_id: uuid::Uuid,
    pub token: String,
}

impl RefreshToken {
    pub async fn create_table(pool: deadpool_postgres::Pool) -> Result<u64, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let create_table = "
      CREATE TABLE IF NOT EXISTS refresh_tokens (
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        user_id UUID NOT NULL,
        token VARCHAR NOT NULL,
        PRIMARY KEY (user_id, token)
      );";
        client.execute(create_table, &[]).await
    }

    pub async fn create(self, pool: deadpool_postgres::Pool) -> Result<u64, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let create = "
        INSERT INTO refresh_tokens (created_at, user_id, token)
        VALUES ($1, $2, $3)";
        client
            .execute(create, &[&self.created_at, &self.user_id, &self.token])
            .await
    }

    pub async fn get_one_by_token(
        pool: deadpool_postgres::Pool,
        token: String,
    ) -> Result<RefreshToken, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
        SELECT created_at, user_id, token
        FROM refresh_tokens
        WHERE token = $1";
        let row = client.query_one(get_one, &[&token]).await?;
        Ok(RefreshToken {
            created_at: row.get(0),
            user_id: row.get(1),
            token: row.get(2),
        })
    }

    pub async fn keep_only_four_token(
        pool: deadpool_postgres::Pool,
        user_id: uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let delete = "
        DELETE FROM refresh_tokens
        WHERE user_id = $1
        AND created_at < (
            SELECT created_at
            FROM refresh_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 1 OFFSET 3
        )";
        client.execute(delete, &[&user_id]).await
    }

    pub async fn delete_token(
        pool: deadpool_postgres::Pool,
        token: String,
    ) -> Result<u64, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let delete = "
        DELETE FROM refresh_tokens
        WHERE token = $1";
        client.execute(delete, &[&token]).await
    }
}

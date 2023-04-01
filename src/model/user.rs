use std::env::var;

use actix_web::{error::ErrorUnauthorized, web, FromRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::Error;
use totp_rs::TotpUrlError;
use tracing::Instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{helper::header, model::token::TokenClaims};

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub nom: String,
    pub prenom: String,
}

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct UserUpdate {
    pub nom: Option<String>,
    pub prenom: Option<String>,
}

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub nom: String,
    pub prenom: String,
    #[serde(skip)]
    pub otp_secret: Option<String>,
    #[serde(skip)]
    pub otp_url: Option<String>,
    pub otp_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn create_table(pool: deadpool_postgres::Pool) -> Result<u64, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let create_table = "
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                nom VARCHAR(255) NOT NULL,
                prenom VARCHAR(255) NOT NULL,
                otp_secret VARCHAR(255),
                otp_url VARCHAR(255),
                otp_enabled BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPZ NOT NULL DEFAULT NOW()
            );";
        client.execute(create_table, &[]).await
    }
    pub async fn get_one(
        pool: deadpool_postgres::Pool,
        id: Uuid,
    ) -> Result<User, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
            SELECT id, email, password, nom, prenom, otp_secret, otp_url, otp_enabled, created_at, updated_at
            FROM users
            WHERE id = $1";
        let row = client.query_one(get_one, &[&id]).await?;
        Ok(User {
            id: row.get(0),
            email: row.get(1),
            password: row.get(2),
            nom: row.get(3),
            prenom: row.get(4),
            otp_secret: row.get(5),
            otp_url: row.get(6),
            otp_enabled: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        })
    }

    // check if user exists
    pub async fn exists(
        pool: deadpool_postgres::Pool,
        email: String,
    ) -> Result<bool, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
            SELECT id
            FROM users
            WHERE email = $1";
        let row = client.query(get_one, &[&email]).await?;
        Ok(row.len() > 0)
    }

    pub async fn get_one_by_mail(
        pool: deadpool_postgres::Pool,
        email: String,
    ) -> Result<User, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
            SELECT id, email, password, nom, prenom, otp_secret, otp_url, otp_enabled, created_at, updated_at
            FROM users
            WHERE email = $1";
        let row = client.query_one(get_one, &[&email]).await?;

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            password: row.get(2),
            nom: row.get(3),
            prenom: row.get(4),
            otp_secret: row.get(5),
            otp_url: row.get(6),
            otp_enabled: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        })
    }

    pub async fn create(self, pool: deadpool_postgres::Pool) -> Result<u64, Error> {
        let client = pool.get().await.unwrap();

        let create = "
            INSERT INTO users (email, password, nom, prenom, otp_secret, otp_url, otp_enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
        client
            .execute(
                create,
                &[
                    &self.email,
                    &self.password,
                    &self.nom,
                    &self.prenom,
                    &self.otp_secret,
                    &self.otp_url,
                    &self.otp_enabled,
                    &self.created_at,
                    &self.updated_at,
                ],
            )
            .await
    }
    pub async fn delete(self, pool: deadpool_postgres::Pool) -> Result<u64, Error> {
        let client = pool.get().await.unwrap();

        let delete_user = "DELETE FROM users where id = $1";
        let delete_token = "DELETE FROM refresh_tokens where user_id = $1";
        if let Err(err) = client.execute(delete_user, &[&self.id]).await {
            return Err(err);
        }
        client.execute(delete_token, &[&self.id]).await
    }

    pub async fn update_name_surname(self, pool: deadpool_postgres::Pool) -> Result<u64, Error> {
        let client = pool.get().await.unwrap();
        let update = "
            UPDATE users
            SET nom = $1, prenom = $2, updated_at = $3
            WHERE id = $4";
        client
            .execute(
                update,
                &[&self.nom, &self.prenom, &chrono::Utc::now(), &self.id],
            )
            .await
    }

    pub async fn update_otp_secret_url_enabled(
        &self,
        pool: deadpool_postgres::Pool,
    ) -> Result<u64, Error> {
        let client = pool.get().await.unwrap();
        let update = "
            UPDATE users
            SET otp_secret = $1, otp_url = $2, otp_enabled = $3, updated_at = $4
            WHERE id = $5";
        client
            .execute(
                update,
                &[
                    &self.otp_secret,
                    &self.otp_url,
                    &self.otp_enabled,
                    &chrono::Utc::now(),
                    &self.id,
                ],
            )
            .await
    }
}

impl User {
    pub fn compare_password(&self, password: String) -> Result<bool, bcrypt::BcryptError> {
        verify(password, &self.password)
    }
    pub fn hash_password(&mut self, password: String) -> Option<bcrypt::BcryptError> {
        match hash(password, DEFAULT_COST) {
            Ok(hash) => {
                self.password = hash;
                None
            }
            Err(e) => Some(e),
        }
    }
    pub fn to_public_user(&self) -> PublicUser {
        PublicUser {
            id: self.id,
            nom: self.nom.clone(),
            prenom: self.prenom.clone(),
        }
    }

    pub fn gen_otp_secret(&mut self) {
        let secret = totp_rs::Secret::generate_secret();
        let secret_byte = match secret.to_bytes() {
            Ok(s) => s,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
        let totp_object = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret_byte,
            Some(var("APP_NAME").unwrap()),
            self.email.clone(),
        )
        .unwrap();
        self.otp_secret = Some(totp_object.get_secret_base32());
    }

    pub fn get_totp_obj(&self) -> Result<totp_rs::TOTP, TotpUrlError> {
        let secret = totp_rs::Secret::Encoded(self.otp_secret.clone().unwrap());
        let secret_byte = match secret.to_bytes() {
            Ok(s) => s,
            Err(err) => {
                return Err(TotpUrlError::Secret(format!("{:?}", err)));
            }
        };
        totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret_byte,
            Some(var("APP_NAME").unwrap()),
            self.email.clone(),
        )
    }

    pub fn validate_otp(&self, otp: String) -> bool {
        let totp = self.get_totp_obj().unwrap();
        match totp.check_current(&otp) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl FromRequest for User {
    type Error = actix_web::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        tracing::info!("Start auth middleware");
        let get_token_span = tracing::info_span!("Auth: Get Token in header");
        let token = match get_token_span.in_scope(|| -> Result<&str, HttpResponse> {
            header::extract_authorization_header(&req)
        }) {
            Ok(token) => token,
            Err(_) => {
                tracing::error!("Error while getting token");
                return Box::pin(async {
                    Err(ErrorUnauthorized("Error lors de la récupération du token"))
                });
            }
        };
        drop(get_token_span);
        let validate_token_span = tracing::info_span!("Auth: Validate Token");
        let claims = match validate_token_span.in_scope(|| -> Result<TokenClaims, String> {
            match TokenClaims::validate_token(token.to_string(), false) {
                Ok(claim) => Ok(claim),
                Err(err) => {
                    tracing::error!(error = ?err, "Error while checking token");
                    return Err("Invalid token".to_string());
                }
            }
        }) {
            Ok(claim) => claim,
            Err(err) => return Box::pin(async { Err(ErrorUnauthorized(err)) }),
        };
        drop(validate_token_span);

        Box::pin(async move {
            let check_user_span = tracing::info_span!("Auth: Check if user exists");
            let user = match async move {
                let pool = req.app_data::<web::Data<Pool>>().unwrap();
                match User::get_one(pool.get_ref().clone(), claims.sub).await {
                    Ok(user) => Ok(user),
                    Err(err) => {
                        tracing::error!(error = ?err, "Error while getting user");
                        return Err(ErrorUnauthorized("Invalid token"));
                    }
                }
            }
            .instrument(check_user_span)
            .await
            {
                Ok(user) => user,
                Err(err) => {
                    tracing::error!(error = ?err, "Error while getting user");
                    return Err(err);
                }
            };
            tracing::debug!(user = ?user.email.clone(),"User authenticated");
            Ok(user)
        })
    }
}

use std::{env::var, time::SystemTimeError};

use super::super::route::auth::info::AuthType;
use super::oidc::Oidc;
use actix_web::{error::ErrorUnauthorized, web, FromRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::Error;
use totp_rs::TotpUrlError;
use tracing::Instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    helper::{self, header},
    model::token::TokenClaims,
};

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
    #[serde(skip)]
    pub one_time_token: Option<String>,
    pub is_oauth: bool,
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
                one_time_token VARCHAR(255),
                is_oauth BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );";
        client.execute(create_table, &[]).await
    }
    pub async fn get_one(
        pool: deadpool_postgres::Pool,
        id: Uuid,
    ) -> Result<User, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
            SELECT id, email, password, nom, prenom, otp_secret, otp_url, otp_enabled, one_time_token, is_oauth, created_at, updated_at
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
            one_time_token: row.get(8),
            is_oauth: row.get(9),
            created_at: row.get(10),
            updated_at: row.get(11),
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
    ) -> Result<Option<User>, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
            SELECT id, email, password, nom, prenom, otp_secret, otp_url, otp_enabled,one_time_token, is_oauth, created_at, updated_at
            FROM users
            WHERE email = $1";
        let row = client.query_opt(get_one, &[&email]).await?;
        match row {
            Some(row_content) => Ok(Some(User {
                id: row_content.get(0),
                email: row_content.get(1),
                password: row_content.get(2),
                nom: row_content.get(3),
                prenom: row_content.get(4),
                otp_secret: row_content.get(5),
                otp_url: row_content.get(6),
                otp_enabled: row_content.get(7),
                one_time_token: row_content.get(8),
                is_oauth: row_content.get(9),
                created_at: row_content.get(10),
                updated_at: row_content.get(11),
            })),
            None => Ok(None),
        }
    }

    pub async fn get_one_by_one_time_token(
        pool: deadpool_postgres::Pool,
        token: String,
    ) -> Result<User, tokio_postgres::Error> {
        let client = pool.get().await.unwrap();

        let get_one = "
            SELECT id, email, password, nom, prenom, otp_secret, otp_url, otp_enabled,one_time_token, is_oauth, created_at, updated_at
            FROM users
            WHERE one_time_token = $1";
        let row = client.query_one(get_one, &[&token]).await?;

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            password: row.get(2),
            nom: row.get(3),
            prenom: row.get(4),
            otp_secret: row.get(5),
            otp_url: row.get(6),
            otp_enabled: row.get(7),
            one_time_token: row.get(8),
            is_oauth: row.get(9),
            created_at: row.get(10),
            updated_at: row.get(11),
        })
    }

    pub async fn create(self, pool: deadpool_postgres::Pool) -> Result<u64, Error> {
        let client = pool.get().await.unwrap();

        let create = "
            INSERT INTO users (email, password, nom, prenom, otp_secret, otp_url, otp_enabled, is_oauth, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)";
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
                    &self.is_oauth,
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

    pub async fn update_otp_secret_url_token_enabled(
        &self,
        pool: deadpool_postgres::Pool,
    ) -> Result<u64, Error> {
        let client = pool.get().await.unwrap();
        let update = "
            UPDATE users
            SET otp_secret = $1, otp_url = $2, otp_enabled = $3, updated_at = $4, one_time_token = $5
            WHERE id = $6";
        client
            .execute(
                update,
                &[
                    &self.otp_secret,
                    &self.otp_url,
                    &self.otp_enabled,
                    &chrono::Utc::now(),
                    &self.one_time_token,
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
    pub fn gen_one_time_token(&mut self) {
        let token = format!("{}_{}", helper::string::generate_random_string(6), self.id);
        self.one_time_token = Some(token);
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

    pub fn validate_otp(&self, otp: String) -> Result<bool, SystemTimeError> {
        let totp = self.get_totp_obj().unwrap();
        return totp.check_current(&otp);
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
        Box::pin(async move {
            let req = req.clone();
            let get_token_span = tracing::info_span!("Auth: Get Token in header");
            let (token, auth_type) =
                match get_token_span.in_scope(|| -> Result<(&str, AuthType), HttpResponse> {
                    header::extract_authorization_type_header(&req)
                }) {
                    Ok(token) => token,
                    Err(_) => {
                        tracing::error!("Error while getting token");
                        return Err(ErrorUnauthorized("Error lors de la récupération du token"));
                    }
                };
            drop(get_token_span);
            tracing::debug!("Token of type {:?} found", auth_type.to_string());
            let email_wrap = match auth_type {
                AuthType::Oidc => {
                    let oidc_handler = match req.app_data::<web::Data<Oidc>>() {
                        Some(handler) => handler,
                        None => {
                            tracing::error!("Error while getting oidc handler");
                            return Err(ErrorUnauthorized("Error avec la configuration OIDC"));
                        }
                    };
                    if oidc_handler.oidc_disabled {
                        tracing::error!("OIDC is disabled");
                        return Err(ErrorUnauthorized("OIDC est désactivé sur ce serveur"));
                    }
                    tracing::debug!("OIDC config loaded");
                    let validate_token_span = tracing::info_span!("Auth: Validate Token (oidc)");
                    async move {
                        match oidc_handler
                            .back
                            .clone()
                            .unwrap()
                            .validate_token(token.to_string())
                            .await {
                                Ok((valide, value)) => {
                                    if valide {
                                        let email = value["email"].to_string().replace("\"", "");
                                        tracing::debug!(email=?email, "Token valide returning email");
                                        Ok(email)
                                    } else {
                                        tracing::error!("Token invalide");
                                        return Err(ErrorUnauthorized(
                                            "Error lors de la récupération du token",
                                        ));
                                    }
                                }
                                Err(err) => {
                                    tracing::error!(error = ?err, "Error while checking token with oidc");
                                    return Err(ErrorUnauthorized(
                                        "Error lors de la récupération du token",
                                    ));
                                }
                            }
                    }
                    .instrument(validate_token_span)
                    .await
                }
                AuthType::BuildIn => {
                    let validate_token_span = tracing::info_span!("Auth: Validate Token");
                    let claims =
                        match validate_token_span.in_scope(|| -> Result<TokenClaims, String> {
                            match TokenClaims::validate_token(token.to_string(), false) {
                                Ok(claim) => Ok(claim),
                                Err(err) => {
                                    tracing::error!(error = ?err, "Error while checking token");
                                    return Err("Invalid token".to_string());
                                }
                            }
                        }) {
                            Ok(claim) => claim,
                            Err(err) => return Err(ErrorUnauthorized(err)),
                        };
                    drop(validate_token_span);
                    Ok(claims.sub.to_string())
                }
            };
            let email = match email_wrap {
                Ok(email) => email,
                Err(err) => return Err(err),
            };
            let check_user_span = tracing::info_span!("Auth: Check if user exists");
            let user = match async move {
                let pool = req.app_data::<web::Data<Pool>>().unwrap();
                match User::get_one_by_mail(pool.get_ref().clone(), email.clone()).await {
                    Ok(user) => match user {
                        Some(user) => Ok(user),
                        None => {
                            tracing::error!("User not found");
                            return Err(ErrorUnauthorized("Invalid token"));
                        }
                    },
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
                    tracing::error!(error = ?err,auth_type = ?auth_type, "Error while getting user");
                    return Err(err);
                }
            };
            tracing::debug!(user = ?user.email.clone(),"User authenticated");
            Ok(user)
        })
    }
}

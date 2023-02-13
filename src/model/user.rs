use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use tokio_postgres::Error;
use utoipa::ToSchema;
use uuid::Uuid;

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
                created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP NOT NULL DEFAULT NOW()
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
    pub fn compare_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, &self.password)
    }
    pub fn hash_password(&mut self, password: &str) -> Option<bcrypt::BcryptError> {
        match hash(password, DEFAULT_COST) {
            Ok(hash) => {
                self.password = hash;
                None
            }
            Err(e) => Some(e),
        }
    }
}

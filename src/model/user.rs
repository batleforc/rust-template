use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub nom: String,
    pub prenom: String,
    pub otp_secret: Option<String>,
    pub otp_url: Option<String>,
    pub otp_enabled: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
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
}

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
}

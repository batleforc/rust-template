use actix_web::{web, Scope};

use super::auth::init::init_auth;

pub fn init_api() -> Scope {
    web::scope("/api").service(init_auth())
}

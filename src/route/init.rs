use actix_web::{web, Scope};

use super::auth::init::init_auth;
use super::user::init::init_user;

pub fn init_api() -> Scope {
    web::scope("/api").service(init_auth()).service(init_user())
}

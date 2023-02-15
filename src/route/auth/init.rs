use actix_web::{web, Scope};

use super::login;

pub fn init_auth() -> Scope {
    web::scope("/auth").service(login::login)
}

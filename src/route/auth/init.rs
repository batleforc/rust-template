use actix_web::{web, Scope};

use super::login;
use super::register;
pub fn init_auth() -> Scope {
    web::scope("/auth")
        .service(login::login)
        .service(register::register)
}

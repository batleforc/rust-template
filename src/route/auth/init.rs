use actix_web::{web, Scope};

use super::login;
use super::refresh;
use super::register;
pub fn init_auth() -> Scope {
    web::scope("/auth")
        .service(login::login)
        .service(register::register)
        .service(refresh::refresh)
}

use actix_web::{web, Scope};

use super::login;
use super::logout;
use super::otp;
use super::refresh;
use super::register;
pub fn init_auth() -> Scope {
    web::scope("/auth")
        .service(login::login)
        .service(register::register)
        .service(refresh::refresh)
        .service(logout::logout)
        .service(otp::init::init_otp())
}

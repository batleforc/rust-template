use actix_web::{web, Scope};

use super::info;
use super::login;
use super::logout;
use super::otp;
use super::refresh;
use super::register;
use super::register_oidc;
pub fn init_auth() -> Scope {
    web::scope("/auth")
        .service(login::login)
        .service(register::register)
        .service(refresh::refresh)
        .service(logout::logout)
        .service(otp::init::init_otp())
        .service(info::auth_status)
        .service(register_oidc::register_oidc)
}

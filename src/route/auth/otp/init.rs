use actix_web::{web, Scope};

use super::generate;

pub fn init_otp() -> Scope {
    web::scope("/otp").service(generate::generate_otp)
}

use actix_web::{web, Scope};

use super::{activate, generate, validate};

pub fn init_otp() -> Scope {
    web::scope("/otp")
        .service(generate::generate_otp)
        .service(activate::activate_otp)
        .service(validate::validate_otp)
}

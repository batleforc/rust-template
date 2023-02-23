use actix_web::{web, Scope};

pub fn init_otp() -> Scope {
    web::scope("/otp")
}

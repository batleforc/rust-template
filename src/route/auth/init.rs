use actix_web::{web, Scope};

pub fn init_couple() -> Scope {
    web::scope("/auth")
}

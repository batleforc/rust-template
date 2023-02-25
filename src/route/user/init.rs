use actix_web::{web, Scope};

use super::current_user;

pub fn init_user() -> Scope {
    web::scope("/user").service(current_user::get_current_user)
}

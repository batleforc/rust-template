use actix_web::{web, Scope};

use super::{current_user, get_one_user};

pub fn init_user() -> Scope {
    web::scope("/user")
        .service(current_user::get_current_user)
        .service(get_one_user::get_one_user)
}

// @todo Add delete user
// @todo Add update user
// @todo Get user by id (public version)

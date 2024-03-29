use actix_web::{web, Scope};

use super::{current_user, delete_user, get_one_user, update_user};

pub fn init_user() -> Scope {
    web::scope("/user")
        .service(current_user::get_current_user)
        .service(get_one_user::get_one_user)
        .service(delete_user::delete_user)
        .service(update_user::update_user)
}

use utoipa::OpenApi;

use super::super::model::oidc;
use super::auth::{
    info, login, logout,
    otp::{activate, generate, validate},
    refresh, register, register_oidc,
};
use super::health;
use super::security::SecurityAddon;
use super::user::{current_user, delete_user, get_one_user, update_user};
use crate::model;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust API",
        version = "0.1.0",
        description = "This is the template for the Rust API",
        contact(
            name = "Batleforc",
            url = "https://weebo.fr",
            email = "maxleriche.60@gmail.com"
        ),
    ),
    tags(
        (name = "Auth", description = "Authentification"),
        (name = "Auth>Otp", description = "Authentification>Otp"),
        (name = "Health", description = "Health check"),
        (name = "User", description = "User management")
    ),
    paths(
        health::health,
        health::hello,
        login::login,
        register::register,
        refresh::refresh,
        logout::logout,
        info::auth_status,
        current_user::get_current_user,
        get_one_user::get_one_user,
        delete_user::delete_user,
        update_user::update_user,
        generate::generate_otp,
        activate::activate_otp,
        validate::validate_otp,
        register_oidc::register_oidc,
    ),
    components(
        schemas(
            model::user::User,
            model::user::PublicUser,
            model::user::UserUpdate,
            generate::GenOtp,
            activate::ActivateOtp,
            validate::ValidateOtp,
            login::LoginUser,
            login::LoginUserReturn,
            login::LoginStatus,
            register::RegisterUser,
            register::RegisterUserReturn,
            refresh::RefreshTokenReturn,
            info::AuthStatus,
            info::AuthProtocol,
            info::AuthType,
            oidc::FrontOidc,
        )
    ),
    modifiers(&SecurityAddon)
)
    ]
pub struct ApiDoc;

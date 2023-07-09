// Endpoint who aim to register/update oidc user when they login

use crate::{
    helper::header,
    model::{oidc::Oidc, user::User},
    route::auth::info::AuthType,
};
use actix_web::{http::header::ContentType, post, web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::Pool;
use tracing::Instrument;

#[utoipa::path(
    tag = "Auth",
    operation_id = "register_oidc",
    path = "/api/auth/register_oidc",
    responses(
        (status = 200, description = "Register oidc user", body = User),
        (status = 500, description = "Possible internal server error", body = String),
        (status = 401, description = "Access denied", body = String)
    ),
    security(
        ("oidc" = [])
    ),
    params(
        ("Authorization-type" = String, Header, description = "Type de token (oidc ou buildin)")
    ),
)]
#[post("/register_oidc")]
pub async fn register_oidc(
    req: HttpRequest,
    db_pool: web::Data<Pool>,
    oidc_handler: web::Data<Oidc>,
) -> impl Responder {
    tracing::info!("Start auth validation");
    let get_token_span = tracing::info_span!("Auth: Get Token in header");
    let (token, auth_type) =
        match get_token_span.in_scope(|| -> Result<(&str, AuthType), HttpResponse> {
            header::extract_authorization_type_header(&req)
        }) {
            Ok((token, auth_type)) => {
                if auth_type != AuthType::Oidc {
                    tracing::error!("Invalid token type");
                    return HttpResponse::Unauthorized()
                        .content_type(ContentType::plaintext())
                        .body("Invalid token type");
                }
                (token, auth_type)
            }
            Err(err) => {
                tracing::error!("Error while getting token");
                return err;
            }
        };
    drop(get_token_span);
    tracing::debug!("Token of type {:?} found", auth_type.to_string());
    let check_token_span = tracing::info_span!("Auth: Check if token is valid and user info");
    let user_info = match {
        if oidc_handler.oidc_disabled {
            tracing::error!("OIDC is disabled");
            return HttpResponse::Unauthorized()
                .content_type(ContentType::plaintext())
                .body("OIDC est désactivé sur ce serveur");
        }
        async move {
            match oidc_handler
                .back
                .clone()
                .unwrap()
                .get_user_info(token.to_string())
                .await
            {
                Ok(user_info) => {
                    tracing::debug!("User info found");
                    Ok(user_info)
                }
                Err(err) => {
                    tracing::error!("Error while getting user info {:?}", err);
                    return Err(HttpResponse::Unauthorized()
                        .content_type(ContentType::plaintext())
                        .body("Invalid token"));
                }
            }
        }
        .instrument(check_token_span)
    }
    .await
    {
        Ok(user_info) => user_info,
        Err(err) => return err,
    };
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    let check_get_user_from_db = tracing::info_span!("Check if user is in database then get it");

    let user_from_db = match {
        let pool = pool.clone();
        let user_info = user_info.clone();
        async move {
            match User::get_one_by_mail(pool, user_info["email"].to_string().replace("\"", ""))
                .await
            {
                Ok(user) => Ok(user),
                Err(err) => {
                    tracing::error!("Error while getting user from database {:?}", err);
                    Err(HttpResponse::InternalServerError()
                        .content_type(ContentType::plaintext())
                        .body("Error while getting user from database"))
                }
            }
        }
        .instrument(check_get_user_from_db)
    }
    .await
    {
        Ok(user) => user,
        Err(err) => return err,
    };
    match user_from_db {
        Some(mut user) => {
            tracing::debug!("User found in database");
            let span_update_user = tracing::info_span!("Update user");
            return async move {
                if !user.is_oauth {
                    tracing::error!("User is not oauth");
                    return HttpResponse::Unauthorized()
                        .content_type(ContentType::plaintext())
                        .body("User is not oauth");
                }
                user.nom = user_info["family_name"].to_string().replace("\"", "");
                user.prenom = user_info["given_name"].to_string().replace("\"", "");
                match user.clone().update_name_surname(pool.clone()).await {
                    Ok(nbr) => {
                        if nbr > 0 {
                            tracing::debug!("User updated");
                        } else {
                            tracing::debug!("Nothing to update");
                        }
                        tracing::debug!("User updated");
                        return HttpResponse::Ok().json(user);
                    }
                    Err(err) => {
                        tracing::error!("Error while updating user {:?}", err);
                        return HttpResponse::InternalServerError()
                            .content_type(ContentType::plaintext())
                            .body("Error while updating user");
                    }
                }
            }
            .instrument(span_update_user)
            .await;
        }
        None => {
            tracing::debug!("User not found in database, proceed to create it");
            let span_create_user = tracing::info_span!("Create user");

            return async move {
                let pool = pool.clone();
                let user = User {
                    id: uuid::Uuid::new_v4(),
                    email: user_info["email"].to_string().replace("\"", ""),
                    password: "".to_string(),
                    is_oauth: true,
                    nom: user_info["family_name"].to_string().replace("\"", ""),
                    prenom: user_info["given_name"].to_string().replace("\"", ""),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    otp_enabled: false,
                    otp_secret: None,
                    otp_url: None,
                    one_time_token: None,
                };

                match user.clone().create(pool).await {
                    Ok(_) => {
                        tracing::debug!("User created");
                        return HttpResponse::Ok().json(user);
                    }
                    Err(err) => {
                        tracing::error!("Error while creating user {:?}", err);
                        return HttpResponse::InternalServerError()
                            .content_type(ContentType::plaintext())
                            .body("Error while creating user");
                    }
                }
            }
            .instrument(span_create_user)
            .await;
        }
    }
}

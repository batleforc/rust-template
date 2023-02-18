use actix_web::{http::header::ContentType, post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use utoipa::ToSchema;

use crate::{helper, model::user::User};

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
    pub nom: String,
    pub prenom: String,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterUserReturn {
    pub created: bool,
    pub email: String,
    pub nom: String,
    pub prenom: String,
}

/// Register user
///
/// Password must be between 3 and 20 characters long and contain at least one number, one lowercase and one uppercase letter.
/// Email must be a valid email.
/// Name and surname must be at least 2 characters long.
#[utoipa::path(
  tag = "Auth",
  request_body = RegisterUser,
  operation_id = "register",
  path = "/api/auth/register",
  responses(
      (status = 200, description = "Register user", body = RegisterUserReturn),
      (status = 400, description = "Error message"),
      (status = 500, description = "Internal server error"),
  )
)]
#[post("/register")]
pub async fn register(
    register_body: web::Json<RegisterUser>,
    db_pool: web::Data<Pool>,
) -> impl Responder {
    let body = register_body.into_inner();
    let pool: Pool = db_pool.into_inner().as_ref().clone();
    let check_user_span = tracing::info_span!("Check if user exist");
    {
        let pool_swap = pool.clone();
        let body_swap = body.clone();
        if let Err(return_content) = async move{
            match User::exists(pool_swap.clone(), body_swap.email.clone()).await {
                Ok(exist) => {
                    if exist {
                        tracing::error!(user = ?body_swap.email.clone() ,"User already exist");
                        return Err(HttpResponse::BadRequest()
                            .content_type(ContentType::plaintext())
                        .body("User already exist"));
                    }
                    tracing::debug!(user = ?body_swap.email.clone() ,"User does not exist");
                    return Ok(());
                }
                Err(err) => {
                    tracing::error!(error = ?err,user = ?body_swap.email.clone(), code= ?err.as_db_error() ,"Error while getting user");
                    return Err(HttpResponse::BadRequest().finish());
                }
            }
        }.instrument(check_user_span).await{
            return return_content;
        }
    }

    {
        let validate_input_span = tracing::info_span!("Validate input");
        if let Err(err_response) =
            validate_input_span.in_scope(|| -> Result<_, HttpResponse> {
                if !helper::string_rule::validate_email(body.email.clone()) {
                    tracing::debug!(user = ?body.email.clone() ,"Email not valid");
                    return Err(HttpResponse::BadRequest()
                .content_type(ContentType::plaintext())
                .body("Email not valid(>=3,<=20, >= 1 number, >= 1 lowercase, >= 1 uppercase)"));
                }

                if !helper::string_rule::validate_password(body.password.clone()) {
                    tracing::debug!(user = ?body.email.clone() ,"Password not valid");
                    return Err(HttpResponse::BadRequest()
                        .content_type(ContentType::plaintext())
                        .body("Password not valid"));
                }

                if !helper::string_rule::validate_name(body.nom.clone()) {
                    tracing::debug!(user = ?body.email.clone() ,"Nom not valid");
                    return Err(HttpResponse::BadRequest()
                        .content_type(ContentType::plaintext())
                        .body("Nom not valid"));
                }

                if !helper::string_rule::validate_name(body.prenom.clone()) {
                    tracing::debug!(user = ?body.email.clone() ,"Prenom not valid");
                    return Err(HttpResponse::BadRequest()
                        .content_type(ContentType::plaintext())
                        .body("Prenom not valid"));
                }
                Ok(())
            })
        {
            return err_response;
        }
    }
    tracing::debug!(user = ?body.email.clone() ,"Input valid");

    let mut user = User {
        id: uuid::Uuid::new_v4(),
        email: body.email.clone(),
        password: body.password.clone(),
        nom: body.nom.clone(),
        prenom: body.prenom.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        otp_enabled: false,
        otp_secret: None,
        otp_url: None,
    };

    let id = user.id.clone();

    {
        let span =
            tracing::info_span!("Hash password (is meant to be slow  to prevent bruteforce)");
        if let Err(res_hash) =span.in_scope(|| -> Result<_, HttpResponse> {
            if let Some(err) = user.hash_password(body.password.clone()) {
            tracing::error!(error = ?err,user = ?body.email.clone() ,"Error while hashing password");
            return Err(HttpResponse::BadRequest()
            .content_type(ContentType::plaintext())
            .body("Error while hashing password"));
        }
        return Ok(());
        }) {
            return res_hash;
        }
    }

    {
        let insert_user_span = tracing::info_span!("Insert user");
        let body_swap = body.clone();
        if let Err(err) = async move {
        if let Err(err) = user.create(pool.clone()).await {
            tracing::error!(error = ?err,user = ?body_swap.email.clone() ,"Error while creating user");
            return Err(HttpResponse::InternalServerError()
                .content_type(ContentType::plaintext())
                .body("Error while creating user"));
        }
        return Ok(());
        }
        .instrument(insert_user_span)
        .await
        {
            return err;
        }
    }

    tracing::debug!(user = ?body.email.clone(), uid = ?id ,"User created");

    HttpResponse::Ok().json(RegisterUserReturn {
        created: true,
        email: body.email.clone(),
        nom: body.nom.clone(),
        prenom: body.prenom.clone(),
    })
}

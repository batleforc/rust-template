use super::super::route::auth::info::AuthType;
use actix_web::{http::header::ContentType, HttpRequest, HttpResponse};

pub fn extract_authorization_type_header(
    req: &HttpRequest,
) -> Result<(&str, AuthType), HttpResponse> {
    let header = req.headers();
    let token_type = match header.get("Authorization-type") {
        Some(token_type) => match token_type.to_str() {
            Ok(token_type) => {
                if token_type.eq("oidc") {
                    AuthType::Oidc
                } else if token_type.eq("buildin") {
                    AuthType::BuildIn
                } else {
                    return Err(HttpResponse::Unauthorized()
                        .content_type(ContentType::plaintext())
                        .body("Invalid token type"));
                }
            }
            Err(err) => {
                return Err(HttpResponse::Unauthorized()
                    .content_type(ContentType::plaintext())
                    .body(format!("Invalid token type: {}", err)))
            }
        },
        None => {
            return Err(HttpResponse::Unauthorized()
                .content_type(ContentType::plaintext())
                .body("No token type provided"))
        }
    };

    let token = match header.get("Authorization") {
        Some(token) => match token.to_str() {
            Ok(token) => {
                if token.starts_with("Bearer ") {
                    &token[7..]
                } else {
                    return Err(HttpResponse::Unauthorized()
                        .content_type(ContentType::plaintext())
                        .body("Invalid token"));
                }
            }
            Err(err) => {
                return Err(HttpResponse::Unauthorized()
                    .content_type(ContentType::plaintext())
                    .body(format!("Invalid token: {}", err)))
            }
        },
        None => {
            return Err(HttpResponse::Unauthorized()
                .content_type(ContentType::plaintext())
                .body("No token provided"))
        }
    };
    Ok((token, token_type))
}

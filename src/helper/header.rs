use actix_web::{http::header::ContentType, HttpRequest, HttpResponse};

pub fn extract_authorization_header(req: &HttpRequest) -> Result<&str, HttpResponse> {
    let token = match req.headers().get("Authorization") {
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
    Ok(token)
}

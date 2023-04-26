use actix_web::{get, HttpResponse};

#[utoipa::path(
    tag = "Health",
    responses(
        (status = 200, description = "Static health ready endpoint", body = String)
    )
)]
#[get("/")]
pub async fn hello() -> &'static str {
    "Hello RustApi!"
}

#[utoipa::path(
    tag = "Health",
    responses(
        (status = 200, description = "Prometheus log", body = String)
    ))]
#[get("/metrics")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

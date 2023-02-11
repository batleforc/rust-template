use crate::helper::tracing::init_telemetry;
use actix_cors::Cors;
use actix_web::{get, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenvy::dotenv;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod helper;
mod model;

/// This is the base endpoint for the API
///
/// This endpoint is used to check if the API is up and running
#[utoipa::path(tag = "health")]
#[get("/")]
async fn hello() -> &'static str {
    "Hello RustApi!"
}

/// This is the health endpoint for the API
///
/// This endpoint is used to provide a prometheus endpoint for the API, and output metrics
#[utoipa::path(tag = "health")]
#[get("/metrics")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Auth", description = "Authentification"),
        (name = "Health", description = "Health check"),
        (name = "User", description = "User management")
    ),
    paths(
        health,
        hello,
    ),
    components(schemas(model::user::User,)))
    ]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    }
    init_telemetry("ApiRust");
    let openapi = ApiDoc::openapi();
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "5437".to_string())
        .parse()
        .unwrap();
    let prometheus = PrometheusMetricsBuilder::new("api_rust")
        .endpoint("/metrics")
        .build()
        .unwrap();
    println!("Starting server on port {}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .wrap(prometheus.clone())
            .service(health)
            .service(hello)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/docs/docs.json", openapi.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

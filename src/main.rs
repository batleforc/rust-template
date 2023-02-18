use crate::helper::tracing::init_telemetry;
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenvy::dotenv;
use tokio_postgres::NoTls;
use tracing_actix_web::TracingLogger;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod helper;
mod model;
mod route;

/// This is the base endpoint for the API
///
/// This endpoint is used to check if the API is up and running
#[utoipa::path(tag = "Health")]
#[get("/")]
async fn hello() -> &'static str {
    "Hello RustApi!"
}

/// This is the health endpoint for the API
///
/// This endpoint is used to provide a prometheus endpoint for the API, and output metrics
#[utoipa::path(tag = "Health")]
#[get("/metrics")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "access_token",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        components.add_security_scheme(
            "refresh_token",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust API",
        version = "0.1.0",
        description = "This is the API for the Rust API",
        contact(
            name = "Batleforc",
            url = "https://weebo.fr",
            email = "maxleriche.60@gmail.com"
        ),
    ),
    tags(
        (name = "Auth", description = "Authentification"),
        (name = "Health", description = "Health check"),
        (name = "User", description = "User management")
    ),
    paths(
        health,
        hello,
        route::auth::login::login,
        route::auth::register::register,
    ),
    components(
        schemas(
            model::user::User,
            route::auth::login::LoginUser,
            route::auth::login::LoginUserReturn,
            route::auth::register::RegisterUser,
            route::auth::register::RegisterUserReturn,
        )
    ),
    modifiers(&SecurityAddon)
)
    ]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    }
    init_telemetry("ApiRust");
    let db_config = model::db::DbConfig::new();
    let dbpool = db_config.pg.create_pool(None, NoTls).unwrap();

    model::db::on_database_init(dbpool.clone()).await;

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
            .app_data(web::Data::new(dbpool.clone()))
            .wrap(cors)
            .wrap(TracingLogger::default())
            .wrap(prometheus.clone())
            .service(health)
            .service(hello)
            .service(route::init::init_api())
            .service(SwaggerUi::new("/docs/{_:.*}").url("/docs/docs.json", openapi.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

use crate::helper::tracing::init_telemetry;
use crate::route::apidoc::ApiDoc;
use crate::route::health::{health, hello};
use actix_cors::Cors;
use actix_web::dev::Service as _;
use actix_web::http::header;
use actix_web::{web, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenvy::dotenv;
use tokio_postgres::NoTls;
use tracing_actix_web::{RequestId, TracingLogger};
use utoipa::OpenApi;
use utoipa_swagger_ui::{oauth, SwaggerUi};

mod helper;
mod model;
mod route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server");
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    }
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "production".to_string());
    let app_name = format!("ApiRust-{}", rust_env);
    std::env::set_var("APP_NAME", app_name.clone());
    println!("Initializing telemetry");
    init_telemetry(app_name.as_str());
    println!("Initializing database");
    let db_config = model::db::DbConfig::new();
    let dbpool = match model::db::DbConfig::get_tls_connector() {
        Some(connector) => db_config
            .pg
            .create_pool(None, connector)
            .expect("Failed to create pool"),
        None => db_config
            .pg
            .create_pool(None, NoTls)
            .expect("Failed to create pool without tls"),
    };
    println!("Initializing database schema");
    model::db::on_database_init(dbpool.clone()).await;

    println!("Initializing OpenApi");
    let openapi = ApiDoc::openapi();
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "5437".to_string())
        .parse()
        .expect("PORT must be a number");
    println!("Initializing Prometheus");
    let prometheus = PrometheusMetricsBuilder::new("api_rust")
        .endpoint("/metrics")
        .build()
        .unwrap();
    let oidc_handler = match model::oidc::Oidc::new() {
        Ok(oidc) => oidc,
        Err(e) => {
            println!("Oidc eror: {}", e);
            model::oidc::Oidc::new_disable()
        }
    };

    println!("Starting server on port {}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        let swagger_ui = match oidc_handler.front.clone() {
            Some(front) => SwaggerUi::new("/docs/{_:.*}")
                .url("/docs/docs.json", openapi.clone())
                .oauth(
                    oauth::Config::new()
                        .client_id(&front.client_id)
                        .scopes(front.get_scope())
                        .use_pkce_with_authorization_code_grant(true),
                ),
            None => SwaggerUi::new("/docs/{_:.*}").url("/docs/docs.json", openapi.clone()),
        };
        App::new()
            .app_data(web::Data::new(dbpool.clone()))
            .app_data(web::Data::new(oidc_handler.clone()))
            .wrap_fn(|mut req, srv| {
                let request_id_asc = req.extract::<RequestId>();
                let fut = srv.call(req);
                async move {
                    let mut res = fut.await?;
                    let request_id: RequestId = request_id_asc.await.unwrap();
                    let request_id_str = format!("{}", request_id);
                    let headers = res.headers_mut();
                    headers.insert(
                        header::HeaderName::from_static("x-request-id"),
                        header::HeaderValue::from_str(request_id_str.as_str()).unwrap(),
                    );
                    Ok(res)
                }
            })
            .wrap(cors)
            .wrap(TracingLogger::default())
            .wrap(prometheus.clone())
            .service(health)
            .service(hello)
            .service(route::init::init_api())
            .service(swagger_ui)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

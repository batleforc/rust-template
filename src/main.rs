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
use utoipa_swagger_ui::SwaggerUi;

mod helper;
mod model;
mod route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    }
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "production".to_string());
    let app_name = format!("ApiRust-{}", rust_env);
    std::env::set_var("APP_NAME", app_name.clone());
    init_telemetry(app_name.as_str());
    let db_config = model::db::DbConfig::new();
    let dbpool = match model::db::DbConfig::get_tls_connector() {
        Some(connector) => db_config.pg.create_pool(None, connector).unwrap(),
        None => db_config.pg.create_pool(None, NoTls).unwrap(),
    };

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
            .wrap_fn(|mut req, srv| {
                let request_id_asc = req.extract::<RequestId>();
                let fut = srv.call(req);
                async move {
                    let mut res = fut.await?;
                    let request_id = request_id_asc.await.unwrap();
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
            .service(SwaggerUi::new("/docs/{_:.*}").url("/docs/docs.json", openapi.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

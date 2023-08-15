use deadpool_postgres::{Config, Pool};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use tokio_postgres::NoTls;

use crate::{config, driven::repository::PersistenceConfig};

#[derive(Debug, Clone)]
pub struct ConfigPG {
    pub pool: Option<Pool>,
    pub pg: Config,
}

impl ConfigPG {
    pub fn new(config: config::PersistenceConfig) -> Self {
        let span = tracing::info_span!("ConfigPG::new");
        let _enter = span.enter();
        let mut config_pg = ConfigPG {
            pool: None,
            pg: ConfigPG::create_pg_config_from_config(config.clone()),
        };
        config_pg.pool = Some(config_pg.create_pool(config.clone()));
        config_pg
    }
    pub fn create_pg_config_from_config(config: config::PersistenceConfig) -> Config {
        Config {
            host: Some(config.host.clone()),
            port: config.port.clone(),
            user: Some(config.user.clone()),
            password: Some(config.password.clone()),
            dbname: Some(config.database.clone()),
            ssl_mode: Some(deadpool_postgres::SslMode::Prefer),
            ..Default::default()
        }
    }
    pub fn get_tls_connector(
        config: config::PersistenceConfig,
    ) -> Option<postgres_openssl::MakeTlsConnector> {
        if config.tls.is_none() || !config.tls.unwrap() {
            return None;
        }
        let mut builder = SslConnector::builder(SslMethod::tls()).expect("Cannot create builder");
        if config.tls_insecure.is_none() || !config.tls_insecure.unwrap() {
            tracing::info!("Setting tls verify mode to none");
            builder.set_verify(SslVerifyMode::NONE);
        }
        let connector = postgres_openssl::MakeTlsConnector::new(builder.build());
        Some(connector)
    }

    pub fn create_pool(&self, config: config::PersistenceConfig) -> Pool {
        let connector = ConfigPG::get_tls_connector(config.clone());
        match connector {
            Some(connector) => {
                tracing::info!("Creating pool with tls");
                self.pg
                    .create_pool(None, connector)
                    .expect("Failed to create pool")
            }
            None => {
                tracing::info!("Creating pool without tls");
                self.pg
                    .create_pool(None, NoTls)
                    .expect("Failed to create pool")
            }
        }
    }
}

impl PersistenceConfig for ConfigPG {}

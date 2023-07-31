use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
//use tokio_postgres::NoTls;

pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub pg: deadpool_postgres::Config,
}

impl DbConfig {
    pub fn new() -> Self {
        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .expect("DB_PORT must be a number");
        let user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
        let database = std::env::var("DB_DATABASE").unwrap_or_else(|_| "postgres".to_string());

        let pg = deadpool_postgres::Config {
            host: Some(host.clone()),
            port: Some(port),
            user: Some(user.clone()),
            password: Some(password.clone()),
            dbname: Some(database.clone()),
            ssl_mode: Some(deadpool_postgres::SslMode::Prefer),
            ..Default::default()
        };

        Self {
            host,
            port,
            user,
            password,
            database,
            pg,
        }
    }

    pub fn get_tls_connector() -> Option<postgres_openssl::MakeTlsConnector> {
        println!("Getting tls connector");
        let tls_mode = std::env::var("DB_TLS")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .expect("DB_TLS must be a boolean");
        if !tls_mode {
            return None;
        }
        println!("Check if verify cert");
        let verify_cert = std::env::var("DB_VERIFY_CERT")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .expect("DB_VERIFY_CERT must be a boolean");
        println!("Creating builder");
        let mut builder = SslConnector::builder(SslMethod::tls()).expect("Cannot create builder");
        if !verify_cert {
            builder.set_verify(SslVerifyMode::NONE);
        }
        println!("Creating connector");
        let connector = postgres_openssl::MakeTlsConnector::new(builder.build());
        Some(connector)
    }
}

pub async fn on_database_init(pool: deadpool_postgres::Pool) {
    let client = pool.get().await.unwrap();

    let install_addon = "
        CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";
      ";
    match client.execute(install_addon, &[]).await {
        Ok(_) => println!("Extension uuid-ossp installed"),
        Err(e) => println!("Error installing extension uuid-ossp: {}", e),
    };

    match super::user::User::create_table(pool.clone()).await {
        Ok(_) => println!("Table users created"),
        Err(e) => {
            panic!("Error creating table users: {}", e);
        }
    }
    match super::token::RefreshToken::create_table(pool.clone()).await {
        Ok(_) => println!("Table refresh_tokens created"),
        Err(e) => {
            panic!("Error creating table refresh_tokens: {}", e);
        }
    }

    println!("Database initialized")
}

// pub async fn on_database_wipe(pool: deadpool_postgres::Pool) {
//     let client = pool.get().await.unwrap();

//     match client.execute("DROP TABLE IF EXISTS users", &[]).await {
//         Ok(_) => println!("Table users dropped"),
//         Err(e) => println!("Error dropping table users: {}", e),
//     };
//     match client
//         .execute("DROP TABLE IF EXISTS refresh_tokens", &[])
//         .await
//     {
//         Ok(_) => println!("Table refresh_tokens dropped"),
//         Err(e) => println!("Error dropping table refresh_tokens: {}", e),
//     };
// }

// pub async fn get_test_database() -> deadpool_postgres::Pool {
//     let host = std::env::var("DB_PORT_TEST").unwrap_or_else(|_| "5433".to_string());
//     std::env::set_var("DB_PORT", host);
//     let config = DbConfig::new();
//     let dbpool = match DbConfig::get_tls_connector() {
//         Some(connector) => config.pg.create_pool(None, connector).unwrap(),
//         None => config.pg.create_pool(None, NoTls).unwrap(),
//     };
//     on_database_wipe(dbpool.clone()).await;
//     on_database_init(dbpool.clone()).await;
//     dbpool
// }

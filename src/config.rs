use serde::Deserialize;
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

use crate::domain::oidc::front::OidcFront;
use crate::domain::oidc::oidchandler::OidcHandler;

const PERSISTENCE_HOST: &str = "PERSISTENCE_HOST";
const PERSISTENCE_PORT: &str = "PERSISTENCE_PORT";
const PERSISTENCE_USER: &str = "PERSISTENCE_USER";
const PERSISTENCE_PWD: &str = "PERSISTENCE_PWD";
const PERSISTENCE_DB: &str = "PERSISTENCE_DB";
const PERSISTENCE_SCHEMA_COLLECTION: &str = "PERSISTENCE_SCHEMA";
const PERSISTENCE_TLS: &str = "PERSISTENCE_TLS";
const PERSISTENCE_TLS_INSECURE: &str = "PERSISTENCE_TLS_INSECURE";
const APP_NAME: &str = "APP_NAME";

const OIDC_ENABLED: &str = "OIDC_ENABLED";
const OIDC_CLIENT_ID: &str = "OIDC_CLIENT_ID";
const OIDC_CLIENT_SECRET: &str = "OIDC_CLIENT_SECRET";
const OIDC_ISSUER: &str = "OIDC_ISSUER";
const OIDC_REDIRECT_URI: &str = "OIDC_REDIRECT_URI";
const OIDC_SCOPES: &str = "OIDC_SCOPES";
const OIDC_USERINFO_URL: &str = "OIDC_USERINFO_URL";
const OIDC_INTROSPECTION_URL: &str = "OIDC_INTROSPECTION_URL";
const OIDC_KEY_ID: &str = "OIDC_KEY_ID";
const OIDC_CLIENT_ASSERTION_TYPE: &str = "OIDC_CLIENT_ASSERTION_TYPE";

const OIDC_FRONT_CLIENT_ID: &str = "OIDC_FRONT_CLIENT_ID";
const OIDC_FRONT_TOKEN_URL: &str = "OIDC_FRONT_TOKEN_URL";
const OIDC_FRONT_AUTH_URL: &str = "OIDC_FRONT_AUTH_URL";
const OIDC_FRONT_ISSUER: &str = "OIDC_FRONT_ISSUER";
const OIDC_FRONT_SCOPES: &str = "OIDC_FRONT_SCOPES";
const OIDC_FRONT_REDIRECT_URL: &str = "OIDC_FRONT_REDIRECT_URL";

const AUTH_REFRESH_TOKEN_SIGN: &str = "AUTH_REFRESH_TOKEN_SIGN";
const AUTH_ACCESS_TOKEN_SIGN: &str = "AUTH_ACCESS_TOKEN_SIGN";

#[derive(Deserialize)]
pub struct Config {
    pub app_name: String,
    pub oidc_enabled: bool,
    pub persistence: PersistenceConfig,
    pub oidc_front: OidcFront,
    pub oidc_back: OidcHandler,
    pub auth: Auth,
}

#[derive(Deserialize, Clone)]
pub struct Auth {
    pub refresh_token_sign: String,
    pub access_token_sign: String,
}

impl Auth {
    pub fn get_key(&self, refresh: bool) -> String {
        match refresh {
            true => self.refresh_token_sign.clone(),
            false => self.access_token_sign.clone(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct PersistenceConfig {
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: String,
    pub database: String,
    pub tls: Option<bool>,
    pub tls_insecure: Option<bool>,
}

impl PersistenceConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.host.is_empty() {
            return Err("host is empty".to_string());
        }
        if self.user.is_empty() {
            return Err("user is empty".to_string());
        }
        if self.password.is_empty() {
            return Err("password is empty".to_string());
        }
        if self.database.is_empty() {
            return Err("database is empty".to_string());
        }
        Ok(())
    }
}

pub fn parse_local_config() -> Config {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/config.toml");
    parse_config(d)
}

pub fn parse_test_config() -> Config {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test_config.toml");
    parse_config(d)
}

pub fn parse_config(path_buf: PathBuf) -> Config {
    let config: Config = parse_config_from_file(path_buf);
    override_config_with_env_vars(config)
}

fn parse_config_from_file(path_buf: PathBuf) -> Config {
    let config_file = path_buf.into_os_string().into_string().unwrap();
    toml::from_str(read_to_string(config_file).unwrap().as_str()).unwrap()
}

fn override_config_with_env_vars(config: Config) -> Config {
    let pers = config.persistence;
    let oidc_back = config.oidc_back;
    let oidc_front = config.oidc_front;

    Config {
        app_name: env::var(APP_NAME).unwrap_or(config.app_name),
        oidc_enabled: env::var(OIDC_ENABLED)
            .map(|p| {
                p.parse::<bool>()
                    .expect("Cannot parse the received oidc_enabled")
            })
            .ok()
            .or(Some(config.oidc_enabled))
            .unwrap(),
        persistence: PersistenceConfig {
            host: env::var(PERSISTENCE_HOST).unwrap_or(pers.host),
            port: env::var(PERSISTENCE_PORT)
                .map(|p| {
                    p.parse::<u16>()
                        .expect("Cannot parse the received persistence port")
                })
                .ok()
                .or(pers.port),
            user: env::var(PERSISTENCE_USER).unwrap_or(pers.user),
            password: env::var(PERSISTENCE_PWD).unwrap_or(pers.password),
            database: env::var(PERSISTENCE_DB).unwrap_or(pers.database),
            tls: env::var(PERSISTENCE_TLS)
                .map(|p| {
                    p.parse::<bool>()
                        .expect("Cannot parse the received persistence tls")
                })
                .ok()
                .or(pers.tls),
            tls_insecure: env::var(PERSISTENCE_TLS_INSECURE)
                .map(|p| {
                    p.parse::<bool>()
                        .expect("Cannot parse the received persistence tls")
                })
                .ok()
                .or(pers.tls_insecure),
        },
        oidc_back: OidcHandler {
            client_id: env::var(OIDC_CLIENT_ID).unwrap_or(oidc_back.client_id),
            client_secret: env::var(OIDC_CLIENT_SECRET).unwrap_or(oidc_back.client_secret),
            issuer: env::var(OIDC_ISSUER).unwrap_or(oidc_back.issuer),
            redirect_url: env::var(OIDC_REDIRECT_URI).unwrap_or(oidc_back.redirect_url),
            scopes: env::var(OIDC_SCOPES).unwrap_or(oidc_back.scopes),
            userinfo_url: env::var(OIDC_USERINFO_URL).unwrap_or(oidc_back.userinfo_url),
            introspection_url: env::var(OIDC_INTROSPECTION_URL)
                .unwrap_or(oidc_back.introspection_url),
            key_id: env::var(OIDC_KEY_ID).unwrap_or(oidc_back.key_id),
            client_assertion_type: env::var(OIDC_CLIENT_ASSERTION_TYPE)
                .unwrap_or(oidc_back.client_assertion_type),
        },
        oidc_front: OidcFront {
            redirect_url: env::var(OIDC_FRONT_REDIRECT_URL).unwrap_or(oidc_front.redirect_url),
            client_id: env::var(OIDC_FRONT_CLIENT_ID).unwrap_or(oidc_front.client_id),
            token_url: env::var(OIDC_FRONT_TOKEN_URL).unwrap_or(oidc_front.token_url),
            auth_url: env::var(OIDC_FRONT_AUTH_URL).unwrap_or(oidc_front.auth_url),
            scopes: env::var(OIDC_FRONT_SCOPES).unwrap_or(oidc_front.scopes),
            issuer: env::var(OIDC_FRONT_ISSUER).unwrap_or(oidc_front.issuer),
        },
        auth: Auth {
            refresh_token_sign: env::var(AUTH_REFRESH_TOKEN_SIGN)
                .unwrap_or(config.auth.refresh_token_sign),
            access_token_sign: env::var(AUTH_ACCESS_TOKEN_SIGN)
                .unwrap_or(config.auth.access_token_sign),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config;
    use serial_test::serial;
    use std::env;
    use std::path::PathBuf;

    #[test]
    #[serial(config)]
    fn should_parse_a_config() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test_config.toml");
        let config = parse_config(d);
        let pers = config.persistence;

        assert_eq!("localhost", pers.host);
        assert_eq!(5433, pers.port.unwrap());
        assert_eq!("GB8eE8vh", pers.user);
        assert_eq!("1OLlRZo1tnNluvx", pers.password);
        assert_eq!("1pNkVsX3FgFeiQdga", pers.database);
        assert_eq!(Some(false), pers.tls);
        assert_eq!(Some(false), pers.tls_insecure);
    }

    #[test]
    #[serial(config)]
    fn should_override_a_parsed_config_with_env_vars() {
        env::set_var(PERSISTENCE_HOST, "my_host");
        env::set_var(PERSISTENCE_PORT, "1111");
        env::set_var(PERSISTENCE_USER, "just_me");
        env::set_var(PERSISTENCE_PWD, "what_a_pwd");
        env::set_var(PERSISTENCE_DB, "my_db");
        env::set_var(PERSISTENCE_TLS, "true");
        env::set_var(PERSISTENCE_TLS_INSECURE, "true");

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test_config.toml");
        let config = parse_config(d);
        let pers = config.persistence;

        assert_eq!("my_host", pers.host);
        assert_eq!(1111, pers.port.unwrap());
        assert_eq!("just_me", pers.user);
        assert_eq!("what_a_pwd", pers.password);
        assert_eq!("my_db", pers.database);
        assert_eq!(Some(true), pers.tls);
        assert_eq!(Some(true), pers.tls_insecure);

        // reset env vars
        env::remove_var(PERSISTENCE_HOST);
        env::remove_var(PERSISTENCE_PORT);
        env::remove_var(PERSISTENCE_USER);
        env::remove_var(PERSISTENCE_PWD);
        env::remove_var(PERSISTENCE_DB);
        env::remove_var(PERSISTENCE_SCHEMA_COLLECTION);
        env::remove_var(PERSISTENCE_TLS);
        env::remove_var(PERSISTENCE_TLS_INSECURE);
    }
}

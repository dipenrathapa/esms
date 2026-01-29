use std::{env, net::SocketAddr};
use tracing::info;
use url::Url;

#[derive(Clone)]
pub struct AppConfig {
    pub redis_url: String,
    pub mysql_url: String,
    pub bind_addr: String,
    pub use_serial: bool,
    pub serial_tcp_host: String,
    pub serial_tcp_port: u16,
}

impl AppConfig {
    fn from_env() -> Self {
        let use_serial = env::var("USE_SERIAL")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let serial_tcp_port = env::var("SERIAL_TCP_PORT")
            .unwrap_or_else(|_| "5555".to_string())
            .parse::<u16>()
            .unwrap_or(5555);

        Self {
            redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
            mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            use_serial,
            serial_tcp_host: env::var("SERIAL_TCP_HOST")
                .unwrap_or_else(|_| "host.docker.internal".to_string()),
            serial_tcp_port,
        }
    }

    /// Creates configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if any required environment variable is missing
    /// or cannot be parsed correctly.
    pub fn from_env_validated() -> Result<Self, ConfigError> {
        let config = Self::from_env();
        validate_config(&config)?;
        Ok(config)
    }
}

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("Invalid Redis URL: {0}")]
    InvalidRedisUrl(String),

    #[error("Invalid MySQL URL: {0}")]
    InvalidMysqlUrl(String),

    #[error("Invalid bind address: {0}")]
    InvalidBindAddr(String),

    #[error("Invalid serial TCP configuration: {0}")]
    InvalidSerialConfig(String),
}

fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    info!(
        operation = "config_validation_start",
        "Starting configuration validation"
    );

    validate_redis_url(&config.redis_url)?;
    validate_mysql_url(&config.mysql_url)?;
    validate_bind_addr(&config.bind_addr)?;

    if config.use_serial {
        validate_serial_config(&config.serial_tcp_host, config.serial_tcp_port)?;
    }

    info!(
        operation = "config_validation_complete",
        "Configuration validation successful"
    );

    Ok(())
}

fn validate_redis_url(url: &str) -> Result<(), ConfigError> {
    if url.is_empty() {
        return Err(ConfigError::InvalidRedisUrl(
            "Redis URL is empty".to_string(),
        ));
    }

    if !url.starts_with("redis://") && !url.starts_with("rediss://") {
        return Err(ConfigError::InvalidRedisUrl(format!(
            "Redis URL must start with redis:// or rediss://, got: {url}"
        )));
    }

    info!(
        operation = "config_validation",
        component = "redis_url",
        "Redis URL validated successfully"
    );

    Ok(())
}

fn validate_mysql_url(url: &str) -> Result<(), ConfigError> {
    if url.is_empty() {
        return Err(ConfigError::InvalidMysqlUrl(
            "MySQL URL is empty".to_string(),
        ));
    }

    if !url.starts_with("mysql://") {
        return Err(ConfigError::InvalidMysqlUrl(format!(
            "MySQL URL must start with mysql://, got: {url}"
        )));
    }

    Url::parse(url)
        .map_err(|e| ConfigError::InvalidMysqlUrl(format!("Invalid URL format: {e}")))?;

    info!(
        operation = "config_validation",
        component = "mysql_url",
        "MySQL URL validated successfully"
    );

    Ok(())
}

fn validate_bind_addr(addr: &str) -> Result<(), ConfigError> {
    if addr.is_empty() {
        return Err(ConfigError::InvalidBindAddr(
            "Bind address is empty".to_string(),
        ));
    }

    addr.parse::<SocketAddr>()
        .map_err(|e| ConfigError::InvalidBindAddr(format!("Invalid socket address format: {e}")))?;

    info!(
        operation = "config_validation",
        component = "bind_addr",
        bind_addr = %addr,
        "Bind address validated successfully"
    );

    Ok(())
}

fn validate_serial_config(host: &str, port: u16) -> Result<(), ConfigError> {
    if host.is_empty() {
        return Err(ConfigError::InvalidSerialConfig(
            "Serial TCP host is empty".to_string(),
        ));
    }

    if port == 0 {
        return Err(ConfigError::InvalidSerialConfig(format!(
            "Serial TCP port {port} is out of valid range (1-65535)"
        )));
    }

    info!(
        operation = "config_validation",
        component = "serial_tcp",
        host = %host,
        port = %port,
        "Serial TCP configuration validated successfully"
    );

    Ok(())
}

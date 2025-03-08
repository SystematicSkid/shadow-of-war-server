use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
    #[error("Environment variable parsing error: {0}")]
    EnvVarParseError(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,  // "json" or "text"
    pub file_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| ConfigError::EnvVarParseError(format!("SERVER_PORT: {}", e)))?,
            workers: env::var("SERVER_WORKERS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .map_err(|e| ConfigError::EnvVarParseError(format!("SERVER_WORKERS: {}", e)))?,
        };

        let logging = LoggingConfig {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            format: env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string()),
            file_path: env::var("LOG_FILE").ok(),
        };

        Ok(Config { server, logging })
    }
}
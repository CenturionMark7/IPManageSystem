use serde::Deserialize;
use std::fs;
use crate::error::{Result, ServerError};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
    pub api: ApiSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub file: String,
    pub max_file_size_mb: u64,
    pub max_backup_files: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiSettings {
    pub endpoint_path: String,
}

impl ServerConfig {
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ServerError::ConfigError(format!("Failed to read config file '{}': {}", path, e)))?;

        let config: ServerConfig = toml::from_str(&contents)
            .map_err(|e| ServerError::ConfigError(format!("Failed to parse config file: {}", e)))?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // ポート番号の検証
        if self.server.port == 0 {
            return Err(ServerError::ConfigError("Invalid port number: 0".to_string()));
        }

        // データベースURLの検証
        if self.database.url.is_empty() {
            return Err(ServerError::ConfigError("Database URL is empty".to_string()));
        }

        if !self.database.url.starts_with("mysql://") {
            return Err(ServerError::ConfigError("Database URL must start with 'mysql://'".to_string()));
        }

        // 接続数の検証
        if self.database.max_connections == 0 {
            return Err(ServerError::ConfigError("max_connections must be greater than 0".to_string()));
        }

        // ログレベルの検証
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ServerError::ConfigError(format!(
                "Invalid log level: '{}'. Must be one of: trace, debug, info, warn, error",
                self.logging.level
            )));
        }

        // エンドポイントパスの検証
        if !self.api.endpoint_path.starts_with('/') {
            return Err(ServerError::ConfigError(
                "API endpoint path must start with '/'".to_string()
            ));
        }

        Ok(())
    }
}

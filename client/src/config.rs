use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::{Result, ClientError};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientConfig {
    pub server: ServerSettings,
    pub client: ClientSettings,
    pub retry: RetrySettings,
    pub pc_info: PcInfoSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerSettings {
    pub url: String,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientSettings {
    pub last_send_datetime: String,
    pub check_interval_secs: u64,
    pub send_interval_secs: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RetrySettings {
    pub first_retry_delay_secs: u64,
    pub second_retry_delay_secs: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PcInfoSettings {
    pub user_name: String,
    pub uuid: String,
    pub mac_address: String,
    pub network_type: String,
    pub ip_address: String,
    pub os: String,
    pub os_version: String,
    pub model_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub file: String,
    pub max_file_size_mb: u64,
    pub max_backup_files: usize,
}

impl ClientConfig {
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ClientError::ConfigError(format!("Failed to read config file '{}': {}", path, e)))?;

        let config: ClientConfig = toml::from_str(&contents)
            .map_err(|e| ClientError::ConfigError(format!("Failed to parse config file: {}", e)))?;

        config.validate()?;

        Ok(config)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        // サーバーURLの検証
        if self.server.url.is_empty() {
            return Err(ClientError::InvalidConfig("Server URL is empty".to_string()));
        }

        if !self.server.url.starts_with("http://") && !self.server.url.starts_with("https://") {
            return Err(ClientError::InvalidConfig(
                "Server URL must start with 'http://' or 'https://'".to_string()
            ));
        }

        // ユーザー名の検証（必須）
        if self.pc_info.user_name.is_empty() {
            return Err(ClientError::MissingField(
                "user_name is required. Please set your name in config.toml".to_string()
            ));
        }

        // インターバルの検証
        if self.client.check_interval_secs == 0 {
            return Err(ClientError::InvalidConfig("check_interval_secs must be greater than 0".to_string()));
        }

        if self.client.send_interval_secs == 0 {
            return Err(ClientError::InvalidConfig("send_interval_secs must be greater than 0".to_string()));
        }

        // リトライ設定の検証
        if self.retry.first_retry_delay_secs == 0 {
            return Err(ClientError::InvalidConfig("first_retry_delay_secs must be greater than 0".to_string()));
        }

        if self.retry.second_retry_delay_secs == 0 {
            return Err(ClientError::InvalidConfig("second_retry_delay_secs must be greater than 0".to_string()));
        }

        // ログレベルの検証
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ClientError::InvalidConfig(format!(
                "Invalid log level: '{}'. Must be one of: trace, debug, info, warn, error",
                self.logging.level
            )));
        }

        Ok(())
    }

    pub fn update_pc_info(&mut self, uuid: String, mac_address: String, network_type: String,
                          ip_address: String, os: String, os_version: String, model_name: String) {
        self.pc_info.uuid = uuid;
        self.pc_info.mac_address = mac_address;
        self.pc_info.network_type = network_type;
        self.pc_info.ip_address = ip_address;
        self.pc_info.os = os;
        self.pc_info.os_version = os_version;
        self.pc_info.model_name = model_name;
    }

    pub fn update_last_send_datetime(&mut self, datetime: String) {
        self.client.last_send_datetime = datetime;
    }

    pub fn is_pc_info_complete(&self) -> bool {
        !self.pc_info.uuid.is_empty()
            && !self.pc_info.mac_address.is_empty()
            && !self.pc_info.network_type.is_empty()
            && !self.pc_info.ip_address.is_empty()
            && !self.pc_info.os.is_empty()
            && !self.pc_info.os_version.is_empty()
            && !self.pc_info.model_name.is_empty()
    }
}

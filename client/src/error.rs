use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("WMI error: {0}")]
    WmiError(String),

    #[error("Network detection error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;

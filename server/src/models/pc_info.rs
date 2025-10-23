use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// データベースから取得するPC情報のモデル
#[derive(Debug, FromRow)]
pub struct PcInfo {
    pub id: i32,
    pub uuid: String,
    pub mac_address: Option<String>,
    pub network_type: Option<String>,
    pub user_name: Option<String>,
    pub ip_address: Option<String>,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub model_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// クライアントからのリクエストDTO
#[derive(Debug, Deserialize)]
pub struct PcInfoRequest {
    pub uuid: String,
    pub mac_address: String,
    pub network_type: String,
    pub user_name: String,
    pub ip_address: String,
    pub os: String,
    pub os_version: String,
    pub model_name: String,
}

/// API レスポンスDTO（成功時）
#[derive(Debug, Serialize)]
pub struct PcInfoResponse {
    pub status: String,
    pub action: String, // "created" or "updated"
    pub id: i32,
}

impl PcInfoResponse {
    /// 新規作成時のレスポンスを生成
    pub fn created(id: i32) -> Self {
        Self {
            status: "success".to_string(),
            action: "created".to_string(),
            id,
        }
    }

    /// 更新時のレスポンスを生成
    pub fn updated(id: i32) -> Self {
        Self {
            status: "success".to_string(),
            action: "updated".to_string(),
            id,
        }
    }
}

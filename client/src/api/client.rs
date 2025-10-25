use crate::error::{ClientError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// API通信クライアント
///
/// サーバーのREST APIと通信し、PC情報を送信します。
pub struct ApiClient {
    /// HTTPクライアント
    client: Client,

    /// サーバーURL（例: "http://localhost:8080"）
    server_url: String,

    /// タイムアウト（秒）
    timeout_secs: u64,
}

/// サーバーに送信するPC情報データ
#[derive(Debug, Clone, Serialize)]
pub struct PcInfoData {
    pub uuid: String,
    pub mac_address: String,
    pub network_type: String,
    pub user_name: String,
    pub ip_address: String,
    pub os: String,
    pub os_version: String,
    pub model_name: String,
}

/// サーバーからのレスポンス（成功時）
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub action: String,
    pub id: i32,
}

/// サーバーからのエラーレスポンス
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl ApiClient {
    /// 新しいAPIクライアントを作成
    ///
    /// # 引数
    /// * `server_url` - サーバーURL（例: "http://localhost:8080"）
    /// * `timeout_secs` - タイムアウト秒数
    ///
    /// # エラー
    /// HTTPクライアントの作成に失敗した場合
    pub fn new(server_url: String, timeout_secs: u64) -> Result<Self> {
        tracing::debug!("Creating API client for server: {}", server_url);

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| ClientError::ApiError(e))?;

        tracing::info!("API client created. Server: {}, Timeout: {}s", server_url, timeout_secs);

        Ok(Self {
            client,
            server_url,
            timeout_secs,
        })
    }

    /// PC情報をサーバーに送信
    ///
    /// POST /api/pc-info エンドポイントにPC情報をJSON形式で送信します。
    ///
    /// # 引数
    /// * `data` - 送信するPC情報
    ///
    /// # 戻り値
    /// サーバーからのレスポンス（ApiResponse）
    ///
    /// # エラー
    /// - ネットワークエラー
    /// - タイムアウト
    /// - サーバーエラー（4xx, 5xx）
    /// - レスポンスのパースエラー
    pub async fn send_pc_info(&self, data: &PcInfoData) -> Result<ApiResponse> {
        let url = &self.server_url;

        tracing::info!("Sending PC info to server");
        tracing::debug!("  URL: {}", url);
        tracing::debug!("  UUID: {}", data.uuid);
        tracing::debug!("  User: {}", data.user_name);
        tracing::debug!("  IP: {}", data.ip_address);

        let response = self
            .client
            .post(url)
            .json(data)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send request: {}", e);
                ClientError::ApiError(e)
            })?;

        let status = response.status();
        tracing::debug!("Response status: {}", status);

        if status.is_success() {
            // 成功レスポンスをパース
            let api_response = response
                .json::<ApiResponse>()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to parse success response: {}", e);
                    ClientError::ApiError(e)
                })?;

            tracing::info!("PC info sent successfully. Action: {}, ID: {}",
                api_response.action, api_response.id);

            Ok(api_response)
        } else {
            // エラーレスポンスをパース
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            tracing::error!("Server returned error status: {}", status);
            tracing::error!("Error response: {}", error_text);

            // エラーメッセージを構築
            let error_message = if let Ok(err_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                format!("Server error: {}", err_response.message)
            } else {
                format!("HTTP {}: {}", status, error_text)
            };

            Err(ClientError::NetworkError(error_message))
        }
    }

    /// サーバーの疎通確認（ヘルスチェック）
    ///
    /// サーバーが起動しているか確認します。
    /// 注: このメソッドは将来のヘルスチェックエンドポイント用の予約です。
    ///
    /// # 戻り値
    /// サーバーが応答すればtrue
    #[allow(dead_code)]
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.server_url);

        tracing::debug!("Performing health check: {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                let is_ok = response.status().is_success();
                tracing::debug!("Health check result: {}", if is_ok { "OK" } else { "Failed" });
                is_ok
            }
            Err(e) => {
                tracing::debug!("Health check failed: {}", e);
                false
            }
        }
    }

    /// サーバーURLを取得
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// タイムアウト秒数を取得
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

impl PcInfoData {
    /// すべてのフィールドが設定されているか検証
    ///
    /// # エラー
    /// 必須フィールドが空の場合
    pub fn validate(&self) -> Result<()> {
        if self.uuid.trim().is_empty() {
            return Err(ClientError::MissingField("uuid".to_string()));
        }
        if self.mac_address.trim().is_empty() {
            return Err(ClientError::MissingField("mac_address".to_string()));
        }
        if self.network_type.trim().is_empty() {
            return Err(ClientError::MissingField("network_type".to_string()));
        }
        if self.user_name.trim().is_empty() {
            return Err(ClientError::MissingField("user_name".to_string()));
        }
        if self.ip_address.trim().is_empty() {
            return Err(ClientError::MissingField("ip_address".to_string()));
        }
        if self.os.trim().is_empty() {
            return Err(ClientError::MissingField("os".to_string()));
        }
        if self.os_version.trim().is_empty() {
            return Err(ClientError::MissingField("os_version".to_string()));
        }
        if self.model_name.trim().is_empty() {
            return Err(ClientError::MissingField("model_name".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("http://localhost:8080".to_string(), 30);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.server_url(), "http://localhost:8080");
        assert_eq!(client.timeout_secs(), 30);
    }

    #[test]
    fn test_pc_info_data_validate_success() {
        let data = PcInfoData {
            uuid: "test-uuid".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            network_type: "Ethernet".to_string(),
            user_name: "testuser".to_string(),
            ip_address: "192.168.1.100".to_string(),
            os: "Windows 11 Pro".to_string(),
            os_version: "10.0.22631".to_string(),
            model_name: "Test Model".to_string(),
        };

        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_pc_info_data_validate_missing_uuid() {
        let data = PcInfoData {
            uuid: "".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            network_type: "Ethernet".to_string(),
            user_name: "testuser".to_string(),
            ip_address: "192.168.1.100".to_string(),
            os: "Windows 11 Pro".to_string(),
            os_version: "10.0.22631".to_string(),
            model_name: "Test Model".to_string(),
        };

        let result = data.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClientError::MissingField(_)));
    }

    #[tokio::test]
    #[ignore] // 実際のサーバーが必要
    async fn test_send_pc_info() {
        let client = ApiClient::new("http://localhost:8080".to_string(), 30).unwrap();

        let data = PcInfoData {
            uuid: "test-uuid-client".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            network_type: "Ethernet".to_string(),
            user_name: "testuser".to_string(),
            ip_address: "192.168.1.100".to_string(),
            os: "Windows 11 Pro".to_string(),
            os_version: "10.0.22631".to_string(),
            model_name: "Test Model".to_string(),
        };

        let result = client.send_pc_info(&data).await;
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
}

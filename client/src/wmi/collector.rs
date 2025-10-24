use crate::error::{ClientError, Result};
use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection, Variant};
use std::collections::HashMap;

/// WMI情報収集構造体
///
/// Windows Management Instrumentation (WMI) を使用して、
/// PC のハードウェア情報、OS情報、システム情報を取得します。
pub struct WmiCollector {
    wmi_con: WMIConnection,
}

/// マザーボード情報（UUID取得用）
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystemProduct")]
#[serde(rename_all = "PascalCase")]
struct ComputerSystemProduct {
    #[serde(rename = "UUID")]
    uuid: Option<String>,
}

/// コンピュータシステム情報（機種名取得用）
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystem")]
#[serde(rename_all = "PascalCase")]
struct ComputerSystem {
    manufacturer: Option<String>,
    model: Option<String>,
}

/// OS情報
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_OperatingSystem")]
#[serde(rename_all = "PascalCase")]
struct OperatingSystem {
    caption: Option<String>,
    version: Option<String>,
}

impl WmiCollector {
    /// 新しいWMIコレクタインスタンスを作成
    ///
    /// # エラー
    /// COMライブラリの初期化またはWMI接続に失敗した場合
    pub fn new() -> Result<Self> {
        tracing::debug!("Initializing WMI connection");

        let com_con = COMLibrary::new()
            .map_err(|e| ClientError::WmiError(format!("Failed to initialize COM library: {}", e)))?;

        let wmi_con = WMIConnection::new(com_con)
            .map_err(|e| ClientError::WmiError(format!("Failed to connect to WMI: {}", e)))?;

        tracing::debug!("WMI connection established successfully");

        Ok(Self { wmi_con })
    }

    /// UUIDを取得
    ///
    /// Win32_ComputerSystemProduct の UUID プロパティから取得します。
    /// UUIDはマザーボードに固有の識別子です。
    ///
    /// # 戻り値
    /// UUID文字列
    ///
    /// # エラー
    /// WMIクエリの実行に失敗した場合、またはUUIDが取得できなかった場合
    pub fn get_uuid(&self) -> Result<String> {
        tracing::debug!("Querying UUID from Win32_ComputerSystemProduct");

        let results: Vec<ComputerSystemProduct> = self.wmi_con
            .query()
            .map_err(|e| ClientError::WmiError(format!("Failed to query UUID: {}", e)))?;

        let uuid = results
            .first()
            .and_then(|r| r.uuid.clone())
            .ok_or_else(|| ClientError::WmiError("UUID not found".to_string()))?;

        tracing::info!("UUID retrieved: {}", uuid);
        Ok(uuid)
    }

    /// 機種名を取得
    ///
    /// Win32_ComputerSystem の Manufacturer と Model から、
    /// "メーカー名 モデル名" の形式で機種名を生成します。
    ///
    /// # 戻り値
    /// 機種名文字列（例: "Dell Inc. OptiPlex 7090"）
    ///
    /// # エラー
    /// WMIクエリの実行に失敗した場合、または機種情報が取得できなかった場合
    pub fn get_model_name(&self) -> Result<String> {
        tracing::debug!("Querying model name from Win32_ComputerSystem");

        let results: Vec<ComputerSystem> = self.wmi_con
            .query()
            .map_err(|e| ClientError::WmiError(format!("Failed to query model name: {}", e)))?;

        let system = results
            .first()
            .ok_or_else(|| ClientError::WmiError("Computer system info not found".to_string()))?;

        let manufacturer = system.manufacturer.clone().unwrap_or_else(|| "Unknown".to_string());
        let model = system.model.clone().unwrap_or_else(|| "Unknown".to_string());

        let model_name = format!("{} {}", manufacturer, model);

        tracing::info!("Model name retrieved: {}", model_name);
        Ok(model_name)
    }

    /// OS情報を取得
    ///
    /// Win32_OperatingSystem の Caption と Version から、
    /// OS名とバージョンを取得します。
    ///
    /// # 戻り値
    /// (OS名, OSバージョン) のタプル
    /// 例: ("Microsoft Windows 11 Pro", "10.0.22631")
    ///
    /// # エラー
    /// WMIクエリの実行に失敗した場合、またはOS情報が取得できなかった場合
    pub fn get_os_info(&self) -> Result<(String, String)> {
        tracing::debug!("Querying OS info from Win32_OperatingSystem");

        let results: Vec<OperatingSystem> = self.wmi_con
            .query()
            .map_err(|e| ClientError::WmiError(format!("Failed to query OS info: {}", e)))?;

        let os = results
            .first()
            .ok_or_else(|| ClientError::WmiError("OS info not found".to_string()))?;

        let os_name = os.caption.clone().unwrap_or_else(|| "Unknown OS".to_string());
        let os_version = os.version.clone().unwrap_or_else(|| "Unknown Version".to_string());

        tracing::info!("OS info retrieved: {} ({})", os_name, os_version);
        Ok((os_name, os_version))
    }

    /// ユーザー名を取得
    ///
    /// 環境変数 USERNAME からWindowsログインユーザー名を取得します。
    ///
    /// # 戻り値
    /// ユーザー名文字列
    ///
    /// # エラー
    /// 環境変数が取得できなかった場合
    pub fn get_user_name() -> Result<String> {
        tracing::debug!("Getting username from environment variable");

        let username = std::env::var("USERNAME")
            .map_err(|_| ClientError::WmiError("USERNAME environment variable not found".to_string()))?;

        tracing::info!("Username retrieved: {}", username);
        Ok(username)
    }

    /// すべてのPC情報を一度に取得
    ///
    /// UUID、機種名、OS情報、ユーザー名をまとめて取得します。
    ///
    /// # 戻り値
    /// PcInfoData構造体
    pub fn collect_all(&self) -> Result<PcInfoData> {
        tracing::info!("Collecting all WMI information");

        let uuid = self.get_uuid()?;
        let model_name = self.get_model_name()?;
        let (os, os_version) = self.get_os_info()?;
        let user_name = Self::get_user_name()?;

        Ok(PcInfoData {
            uuid,
            model_name,
            os,
            os_version,
            user_name,
        })
    }
}

/// WMIから収集したPC情報
#[derive(Debug, Clone)]
pub struct PcInfoData {
    pub uuid: String,
    pub model_name: String,
    pub os: String,
    pub os_version: String,
    pub user_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // WMIはWindows環境でのみ動作するため、通常は無視
    fn test_wmi_collector_new() {
        let collector = WmiCollector::new();
        assert!(collector.is_ok());
    }

    #[test]
    #[ignore]
    fn test_get_uuid() {
        let collector = WmiCollector::new().unwrap();
        let uuid = collector.get_uuid();
        assert!(uuid.is_ok());
        let uuid_str = uuid.unwrap();
        assert!(!uuid_str.is_empty());
        println!("UUID: {}", uuid_str);
    }

    #[test]
    #[ignore]
    fn test_get_model_name() {
        let collector = WmiCollector::new().unwrap();
        let model = collector.get_model_name();
        assert!(model.is_ok());
        let model_str = model.unwrap();
        assert!(!model_str.is_empty());
        println!("Model: {}", model_str);
    }

    #[test]
    #[ignore]
    fn test_get_os_info() {
        let collector = WmiCollector::new().unwrap();
        let os_info = collector.get_os_info();
        assert!(os_info.is_ok());
        let (os, version) = os_info.unwrap();
        assert!(!os.is_empty());
        assert!(!version.is_empty());
        println!("OS: {} ({})", os, version);
    }

    #[test]
    #[ignore]
    fn test_get_user_name() {
        let username = WmiCollector::get_user_name();
        assert!(username.is_ok());
        let username_str = username.unwrap();
        assert!(!username_str.is_empty());
        println!("Username: {}", username_str);
    }

    #[test]
    #[ignore]
    fn test_collect_all() {
        let collector = WmiCollector::new().unwrap();
        let info = collector.collect_all();
        assert!(info.is_ok());
        let data = info.unwrap();
        println!("All WMI Info: {:?}", data);
    }
}

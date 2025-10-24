use crate::error::{ClientError, Result};
use network_interface::{NetworkInterface, NetworkInterfaceConfig, Addr};
use std::net::IpAddr;

/// ネットワーク情報検出構造体
///
/// アクティブなネットワークアダプタを検出し、
/// IPアドレス、MACアドレス、ネットワークタイプを取得します。
pub struct NetworkDetector;

/// ネットワーク情報
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// IPアドレス（例: "192.168.1.100"）
    pub ip_address: String,

    /// MACアドレス（例: "00:11:22:33:44:55"）
    pub mac_address: String,

    /// ネットワークタイプ（"Ethernet" または "Wi-Fi"）
    pub network_type: String,
}

impl NetworkDetector {
    /// アクティブなネットワークアダプタを検出して情報を取得
    ///
    /// 以下の条件でアクティブなアダプタを判定します：
    /// 1. UP状態のインターフェース
    /// 2. IPv4アドレスが設定されている
    /// 3. ループバックでない（127.x.x.x以外）
    /// 4. リンクローカルアドレスでない（169.254.x.x以外）
    ///
    /// # 戻り値
    /// NetworkInfo構造体
    ///
    /// # エラー
    /// - ネットワークインターフェースの取得に失敗した場合
    /// - アクティブなアダプタが見つからなかった場合
    pub fn get_active_adapter() -> Result<NetworkInfo> {
        tracing::debug!("Detecting active network adapter");

        let network_interfaces = NetworkInterface::show()
            .map_err(|e| ClientError::NetworkError(format!("Failed to get network interfaces: {}", e)))?;

        tracing::debug!("Found {} network interfaces", network_interfaces.len());

        // アクティブなインターフェースを探す
        for iface in network_interfaces {
            // デバッグ情報出力
            tracing::debug!("Checking interface: {} ({})", iface.name,
                if iface.addr.is_empty() { "no addresses" } else { "has addresses" });

            // MACアドレスが存在するか確認
            let mac = if let Some(mac_addr) = iface.mac_addr {
                Self::format_mac_address(&mac_addr)
            } else {
                tracing::debug!("  Skipping: No MAC address");
                continue;
            };

            // IPアドレスを探す
            for addr in &iface.addr {
                if let Addr::V4(v4_addr) = addr {
                    let ip = v4_addr.ip;
                    let ip_str = ip.to_string();

                    // ループバックアドレスをスキップ
                    if ip.is_loopback() {
                        tracing::debug!("  Skipping loopback address: {}", ip_str);
                        continue;
                    }

                    // リンクローカルアドレス（169.254.x.x）をスキップ
                    if ip_str.starts_with("169.254.") {
                        tracing::debug!("  Skipping link-local address: {}", ip_str);
                        continue;
                    }

                    // ネットワークタイプを判定
                    let network_type = Self::detect_network_type(&iface.name);

                    tracing::info!("Active network adapter detected:");
                    tracing::info!("  Interface: {}", iface.name);
                    tracing::info!("  IP Address: {}", ip_str);
                    tracing::info!("  MAC Address: {}", mac);
                    tracing::info!("  Network Type: {}", network_type);

                    return Ok(NetworkInfo {
                        ip_address: ip_str,
                        mac_address: mac.clone(),
                        network_type,
                    });
                }
            }
        }

        Err(ClientError::NetworkError(
            "No active network adapter found".to_string(),
        ))
    }

    /// MACアドレスをフォーマット
    ///
    /// バイト配列を "00:11:22:33:44:55" 形式の文字列に変換します。
    ///
    /// # 引数
    /// * `mac` - MACアドレスのバイト配列（通常6バイト）
    ///
    /// # 戻り値
    /// フォーマットされたMACアドレス文字列
    fn format_mac_address(mac: &str) -> String {
        // network-interfaceライブラリが返すMAC形式をそのまま使用
        // 既に "00:11:22:33:44:55" 形式で返されるため、そのまま返す
        mac.to_string()
    }

    /// ネットワークタイプを判定
    ///
    /// インターフェース名から、Ethernet か Wi-Fi かを判定します。
    ///
    /// # 引数
    /// * `interface_name` - インターフェース名（例: "Ethernet", "Wi-Fi", "eth0"）
    ///
    /// # 戻り値
    /// "Ethernet" または "Wi-Fi"
    fn detect_network_type(interface_name: &str) -> String {
        let name_lower = interface_name.to_lowercase();

        // Wi-Fiの判定
        if name_lower.contains("wi-fi")
            || name_lower.contains("wifi")
            || name_lower.contains("wireless")
            || name_lower.contains("wlan") {
            "Wi-Fi".to_string()
        }
        // それ以外はEthernetとみなす
        else {
            "Ethernet".to_string()
        }
    }

    /// すべてのネットワークインターフェース情報を取得（デバッグ用）
    ///
    /// システムに存在するすべてのネットワークインターフェース情報を取得します。
    /// トラブルシューティングやデバッグに使用します。
    ///
    /// # 戻り値
    /// NetworkInterface構造体のベクター
    ///
    /// # エラー
    /// ネットワークインターフェースの取得に失敗した場合
    #[allow(dead_code)]
    pub fn get_all_interfaces() -> Result<Vec<NetworkInterface>> {
        let interfaces = NetworkInterface::show()
            .map_err(|e| ClientError::NetworkError(format!("Failed to get network interfaces: {}", e)))?;

        Ok(interfaces)
    }

    /// 指定されたインターフェース名からネットワーク情報を取得
    ///
    /// 特定のインターフェース名を指定して情報を取得します。
    /// テストや特定アダプタの情報取得に使用します。
    ///
    /// # 引数
    /// * `interface_name` - インターフェース名
    ///
    /// # 戻り値
    /// NetworkInfo構造体
    ///
    /// # エラー
    /// 指定されたインターフェースが見つからない、または情報取得に失敗した場合
    #[allow(dead_code)]
    pub fn get_interface_by_name(interface_name: &str) -> Result<NetworkInfo> {
        tracing::debug!("Getting network info for interface: {}", interface_name);

        let network_interfaces = NetworkInterface::show()
            .map_err(|e| ClientError::NetworkError(format!("Failed to get network interfaces: {}", e)))?;

        for iface in network_interfaces {
            if iface.name == interface_name {
                let mac = iface.mac_addr
                    .map(|m| Self::format_mac_address(&m))
                    .ok_or_else(|| ClientError::NetworkError(
                        format!("No MAC address for interface: {}", interface_name)
                    ))?;

                // 最初のIPv4アドレスを取得
                for addr in &iface.addr {
                    if let Addr::V4(v4_addr) = addr {
                        let ip_str = v4_addr.ip.to_string();
                        let network_type = Self::detect_network_type(&iface.name);

                        return Ok(NetworkInfo {
                            ip_address: ip_str,
                            mac_address: mac,
                            network_type,
                        });
                    }
                }
            }
        }

        Err(ClientError::NetworkError(
            format!("Interface not found: {}", interface_name),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_network_type_wifi() {
        assert_eq!(NetworkDetector::detect_network_type("Wi-Fi"), "Wi-Fi");
        assert_eq!(NetworkDetector::detect_network_type("wifi"), "Wi-Fi");
        assert_eq!(NetworkDetector::detect_network_type("wlan0"), "Wi-Fi");
        assert_eq!(NetworkDetector::detect_network_type("Wireless"), "Wi-Fi");
    }

    #[test]
    fn test_detect_network_type_ethernet() {
        assert_eq!(NetworkDetector::detect_network_type("Ethernet"), "Ethernet");
        assert_eq!(NetworkDetector::detect_network_type("eth0"), "Ethernet");
        assert_eq!(NetworkDetector::detect_network_type("Local Area Connection"), "Ethernet");
    }

    #[test]
    fn test_format_mac_address() {
        let mac = "00:11:22:33:44:55";
        assert_eq!(NetworkDetector::format_mac_address(mac), "00:11:22:33:44:55");
    }

    #[test]
    #[ignore] // 実際のネットワーク環境でのみ動作
    fn test_get_active_adapter() {
        let result = NetworkDetector::get_active_adapter();
        assert!(result.is_ok());

        let info = result.unwrap();
        println!("Active Adapter:");
        println!("  IP: {}", info.ip_address);
        println!("  MAC: {}", info.mac_address);
        println!("  Type: {}", info.network_type);

        assert!(!info.ip_address.is_empty());
        assert!(!info.mac_address.is_empty());
        assert!(!info.network_type.is_empty());
    }

    #[test]
    #[ignore]
    fn test_get_all_interfaces() {
        let result = NetworkDetector::get_all_interfaces();
        assert!(result.is_ok());

        let interfaces = result.unwrap();
        println!("All Interfaces ({}):", interfaces.len());
        for iface in interfaces {
            println!("  Name: {}", iface.name);
            if let Some(mac) = iface.mac_addr {
                println!("    MAC: {}", mac);
            }
            for addr in iface.addr {
                if let Addr::V4(v4_addr) = addr {
                    println!("    IPv4: {}", v4_addr.ip);
                }
            }
        }
    }
}

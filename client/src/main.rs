mod api;
mod config;
mod error;
mod network;
mod wmi;

use api::{ApiClient, PcInfoData};
use config::ClientConfig;
use network::NetworkDetector;
use wmi::WmiCollector;
use chrono::Utc;
use tokio::time::{interval, sleep, Duration};
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use tokio::sync::Mutex;

/// リトライ状態
#[derive(Debug, Clone, Copy)]
enum RetryState {
    FirstRetry,
    SecondRetry,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 設定ファイルのパス（環境変数またはデフォルト）
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

    // 設定読み込み
    let mut config = ClientConfig::load(&config_path)?;

    // ログ初期化
    init_logging(&config);
    info!("PC Inventory Client starting...");
    info!("Configuration loaded from: {}", config_path);
    info!("Server URL: {}", config.server.url);

    // ユーザー名チェック
    if config.pc_info.user_name.is_empty() {
        error!("User name is not set in config.toml");
        error!("Please set your name in the [pc_info] section and restart the client");
        return Err("User name is required".into());
    }

    // リトライ中フラグ（スレッド間で共有）
    let is_retrying = Arc::new(Mutex::new(false));

    // 起動時処理
    if let Err(e) = initial_process(&mut config, &config_path).await {
        error!("Initial process failed: {}", e);
        // 送信失敗時はリトライサイクルを開始
        start_retry_cycle(is_retrying.clone(), config_path.clone()).await;
    }

    // 定期チェックタイマー
    info!("Starting periodic check timer (interval: {}s)", config.client.check_interval_secs);
    let mut check_timer = interval(Duration::from_secs(config.client.check_interval_secs));

    loop {
        check_timer.tick().await;
        debug!("Periodic check triggered");

        // 設定を再読み込み（変更を反映）
        match ClientConfig::load(&config_path) {
            Ok(new_config) => {
                config = new_config;
                debug!("Configuration reloaded");
            }
            Err(e) => {
                warn!("Failed to reload configuration: {}", e);
                // エラーでも既存の設定で継続
            }
        }

        // 送信が必要かチェック
        if should_send(&config).await {
            info!("Send interval elapsed, sending PC info");
            if let Err(e) = periodic_check(&mut config, &config_path).await {
                error!("Periodic check failed: {}", e);
                // 送信失敗時はリトライサイクルを開始
                start_retry_cycle(is_retrying.clone(), config_path.clone()).await;
            }
        } else {
            debug!("Send interval not elapsed yet, skipping");
        }
    }
}

/// ログ初期化
fn init_logging(config: &ClientConfig) {
    let file_appender = tracing_appender::rolling::never(".", &config.logging.file);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.logging.level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();
}

/// 起動時処理
///
/// WMI情報とネットワーク情報を取得し、設定ファイルを更新してサーバーに送信します。
async fn initial_process(config: &mut ClientConfig, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running initial process");

    // WMI情報取得
    info!("Collecting WMI information");
    let wmi_collector = WmiCollector::new()?;
    let wmi_data = wmi_collector.collect_all()?;

    info!("WMI information collected:");
    info!("  UUID: {}", wmi_data.uuid);
    info!("  Model: {}", wmi_data.model_name);
    info!("  OS: {} ({})", wmi_data.os, wmi_data.os_version);
    info!("  User: {}", wmi_data.user_name);

    // ネットワーク情報取得
    info!("Detecting network information");
    let network_info = NetworkDetector::get_active_adapter()?;

    info!("Network information detected:");
    info!("  IP: {}", network_info.ip_address);
    info!("  MAC: {}", network_info.mac_address);
    info!("  Type: {}", network_info.network_type);

    // 設定ファイルを更新
    config.update_pc_info(
        wmi_data.uuid.clone(),
        network_info.mac_address.clone(),
        network_info.network_type.clone(),
        network_info.ip_address.clone(),
        wmi_data.os.clone(),
        wmi_data.os_version.clone(),
        wmi_data.model_name.clone(),
    );

    // 設定ファイルを保存
    config.save(config_path)?;
    info!("Configuration updated and saved");

    // サーバーに送信
    send_to_server(config, config_path).await?;

    Ok(())
}

/// 定期チェック処理
///
/// ネットワーク情報を再取得し、サーバーに送信します。
async fn periodic_check(config: &mut ClientConfig, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running periodic check");

    // ネットワーク情報を再取得
    let network_info = NetworkDetector::get_active_adapter()?;

    debug!("Network information updated:");
    debug!("  IP: {}", network_info.ip_address);
    debug!("  MAC: {}", network_info.mac_address);
    debug!("  Type: {}", network_info.network_type);

    // 設定ファイルのネットワーク情報を更新
    config.pc_info.ip_address = network_info.ip_address;
    config.pc_info.mac_address = network_info.mac_address;
    config.pc_info.network_type = network_info.network_type;

    // サーバーに送信
    send_to_server(config, config_path).await?;

    Ok(())
}

/// サーバーに送信
async fn send_to_server(config: &mut ClientConfig, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // PC情報が完全でない場合はスキップ
    if !config.is_pc_info_complete() {
        warn!("PC information is incomplete, skipping send");
        return Ok(());
    }

    // APIクライアント作成
    let api_client = ApiClient::new(
        config.server.url.clone(),
        config.server.request_timeout_secs,
    )?;

    // 送信データ作成
    let data = PcInfoData {
        uuid: config.pc_info.uuid.clone(),
        mac_address: config.pc_info.mac_address.clone(),
        network_type: config.pc_info.network_type.clone(),
        user_name: config.pc_info.user_name.clone(),
        ip_address: config.pc_info.ip_address.clone(),
        os: config.pc_info.os.clone(),
        os_version: config.pc_info.os_version.clone(),
        model_name: config.pc_info.model_name.clone(),
    };

    // データ検証
    data.validate()?;

    // 送信
    info!("Sending PC information to server");
    let response = api_client.send_pc_info(&data).await?;

    info!("Server response: {} (action: {}, id: {})",
        response.status, response.action, response.id);

    // 最終送信日時を更新
    let now = Utc::now().to_rfc3339();
    config.update_last_send_datetime(now);
    config.save(config_path)?;

    info!("Last send datetime updated in config");

    Ok(())
}

/// 送信が必要かチェック
///
/// 最終送信日時からsend_interval_secsが経過しているか確認します。
async fn should_send(config: &ClientConfig) -> bool {
    // 最終送信日時が空の場合は送信必要
    if config.client.last_send_datetime.is_empty() {
        debug!("No last send datetime, should send");
        return true;
    }

    // 最終送信日時をパース
    match chrono::DateTime::parse_from_rfc3339(&config.client.last_send_datetime) {
        Ok(last_send) => {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_send.with_timezone(&Utc));
            let elapsed_secs = elapsed.num_seconds();

            debug!("Last send: {} ({} seconds ago)", last_send, elapsed_secs);

            elapsed_secs >= config.client.send_interval_secs as i64
        }
        Err(e) => {
            warn!("Failed to parse last send datetime: {}", e);
            // パースエラーの場合は送信する
            true
        }
    }
}

/// リトライサイクルを開始
///
/// 既にリトライ中でない場合のみ、新しいリトライタスクをspawnします。
async fn start_retry_cycle(is_retrying: Arc<Mutex<bool>>, config_path: String) {
    let mut retrying = is_retrying.lock().await;

    // 既にリトライ中の場合は何もしない
    if *retrying {
        debug!("Already retrying, skipping new retry cycle");
        return;
    }

    // リトライ開始
    *retrying = true;
    drop(retrying); // ロックを解放

    info!("Starting retry cycle");

    // リトライタスクをspawn
    let is_retrying_clone = is_retrying.clone();
    tokio::spawn(async move {
        handle_retry_cycle(is_retrying_clone, config_path).await;
    });
}

/// リトライサイクル処理
///
/// first_retry_delay_secs → second_retry_delay_secs を交互に繰り返します。
async fn handle_retry_cycle(is_retrying: Arc<Mutex<bool>>, config_path: String) {
    let mut state = RetryState::FirstRetry;

    loop {
        // 設定を読み込み
        let config = match ClientConfig::load(&config_path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load config during retry: {}", e);
                sleep(Duration::from_secs(60)).await; // エラー時は1分待機
                continue;
            }
        };

        // 待機時間を決定
        let delay_secs = match state {
            RetryState::FirstRetry => config.retry.first_retry_delay_secs,
            RetryState::SecondRetry => config.retry.second_retry_delay_secs,
        };

        info!("Retry scheduled in {} seconds (state: {:?})", delay_secs, state);
        sleep(Duration::from_secs(delay_secs)).await;

        // リトライ送信
        info!("Attempting retry send (state: {:?})", state);

        match retry_send(&config, &config_path).await {
            Ok(_) => {
                info!("Retry send successful, exiting retry cycle");
                // 成功したのでリトライフラグをfalseに
                let mut retrying = is_retrying.lock().await;
                *retrying = false;
                break;
            }
            Err(e) => {
                error!("Retry send failed: {}", e);
                // 次の状態に遷移
                state = match state {
                    RetryState::FirstRetry => RetryState::SecondRetry,
                    RetryState::SecondRetry => RetryState::FirstRetry,
                };
            }
        }
    }
}

/// リトライ送信
///
/// ネットワーク情報を再取得してサーバーに送信します。
async fn retry_send(_config: &ClientConfig, config_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Collecting information for retry send");

    // ネットワーク情報を再取得
    let network_info = NetworkDetector::get_active_adapter()?;

    debug!("Network information updated:");
    debug!("  IP: {}", network_info.ip_address);
    debug!("  MAC: {}", network_info.mac_address);
    debug!("  Type: {}", network_info.network_type);

    // 設定を再読み込みして最新の情報を取得
    let mut config = ClientConfig::load(config_path)?;

    // ネットワーク情報を更新
    config.pc_info.ip_address = network_info.ip_address;
    config.pc_info.mac_address = network_info.mac_address;
    config.pc_info.network_type = network_info.network_type;

    // PC情報が完全でない場合はエラー
    if !config.is_pc_info_complete() {
        warn!("PC information is incomplete, cannot retry send");
        return Err("PC information is incomplete".into());
    }

    // APIクライアント作成
    let api_client = ApiClient::new(
        config.server.url.clone(),
        config.server.request_timeout_secs,
    )?;

    // 送信データ作成
    let data = PcInfoData {
        uuid: config.pc_info.uuid.clone(),
        mac_address: config.pc_info.mac_address.clone(),
        network_type: config.pc_info.network_type.clone(),
        user_name: config.pc_info.user_name.clone(),
        ip_address: config.pc_info.ip_address.clone(),
        os: config.pc_info.os.clone(),
        os_version: config.pc_info.os_version.clone(),
        model_name: config.pc_info.model_name.clone(),
    };

    // データ検証
    data.validate()?;

    // 送信
    info!("Sending PC information to server (retry)");
    let response = api_client.send_pc_info(&data).await?;

    info!("Server response: {} (action: {}, id: {})",
        response.status, response.action, response.id);

    // 最終送信日時を更新
    let now = Utc::now().to_rfc3339();
    config.update_last_send_datetime(now);
    config.save(config_path)?;

    info!("Last send datetime updated in config");

    Ok(())
}

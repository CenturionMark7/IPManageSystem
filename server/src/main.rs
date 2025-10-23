mod config;
mod db;
mod error;
mod handlers;
mod models;

use axum::{routing::post, Router};
use sqlx::mysql::MySqlPoolOptions;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::ServerConfig;
use crate::db::repository::PcInfoRepository;
use crate::handlers::pc_info::handle_pc_info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 設定ファイルのパス（環境変数またはデフォルト）
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

    // 設定読み込み
    let config = ServerConfig::load(&config_path)?;

    // ログ初期化
    init_logging(&config);
    tracing::info!("Starting PC Inventory Server...");
    tracing::info!("Configuration loaded from: {}", config_path);

    // データベース接続プール作成
    tracing::info!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(config.database.connection_timeout_secs))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_secs))
        .connect(&config.database.url)
        .await?;
    tracing::info!("Database connection established");

    // リポジトリ作成
    let repository = PcInfoRepository::new(pool);

    // Axumルーター設定
    let app = Router::new()
        .route(&config.api.endpoint_path, post(handle_pc_info))
        .layer(TraceLayer::new_for_http())
        .with_state(repository);

    // サーバーアドレス設定
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Server listening on {}", addr);
    tracing::info!("API endpoint: POST {}", config.api.endpoint_path);

    // サーバー起動
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// ログ初期化
fn init_logging(config: &ServerConfig) {
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

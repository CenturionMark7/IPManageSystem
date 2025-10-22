# PC情報収集システム チケット管理

バージョン: 2.1  
最終更新日: 2025-10-21

---

## 工数サマリー

| フェーズ | チケット数 | 合計工数 |
|------|----------|---------|
| フェーズ1: 環境構築 | 2 | 1日 |
| フェーズ2: 共通基盤実装 | 2 | 1.5日 |
| フェーズ3: サーバー側実装 | 5 | 4日 |
| フェーズ4: クライアント側実装 | 6 | 6日 |
| フェーズ5: システム統合テスト | 2 | 2～3日 |
| フェーズ6: ドキュメント・デプロイ | 5 | 2.5日 + 1週間 |
| **合計** | **22** | **約17～18日 + パイロット1週間** |

---

## チケット依存関係

```
環境構築
#1 → #2 (DB初期構築)

共通基盤
#1 → #3 (共通エラー定義)
  → #4 (共通設定管理)

サーバー側
#2,#3,#4 → #5 (データモデル)
        → #6 (リポジトリ)
#5,#6 → #7 (APIハンドラー)
     → #8 (ログ実装)
     → #9 (サーバー統合テスト)

クライアント側
#3,#4 → #10 (WMI情報取得)
      → #11 (ネットワーク情報取得)
      → #12 (API通信クライアント)
#10,#11,#12 → #13 (メイン処理・タイマー)
           → #14 (リトライ処理)
           → #15 (クライアント統合テスト)

統合・デプロイ
#9,#15 → #16 (システム統合テスト)
      → #17 (不具合修正)
      → #18 (ドキュメント作成)
      → #19 (デプロイパッケージ)
      → #20 (本番環境構築)
      → #21 (パイロット運用)
```

---

## フェーズ1: 環境構築

### チケット #1: 開発環境セットアップ

**担当**: インフラ/開発  
**工数**: 0.5日  
**依存**: なし  
**優先度**: 高

#### タスク
- [ ] Rust開発環境構築 (rustup, cargo)
- [ ] Visual Studio Code + rust-analyzer セットアップ
- [ ] MySQL インストール・初期設定
- [ ] MySQL Workbench インストール
- [ ] Git リポジトリ初期化

#### 成果物
- 開発環境構築手順書
- リポジトリ構成

#### プロジェクト構造
```
pc-inventory/
├── server/          # サーバー側プロジェクト
│   ├── Cargo.toml
│   ├── src/
│   └── config.toml.template
├── client/          # クライアント側プロジェクト
│   ├── Cargo.toml
│   ├── src/
│   └── config.toml.template
└── docs/            # ドキュメント
```

#### 注意点
- Rustのバージョンは最新のstableを使用
- クロスコンパイル用のツールチェーン設定

---

### チケット #2: データベース初期構築

**担当**: サーバー側開発  
**工数**: 0.5日  
**依存**: #1  
**優先度**: 高

#### タスク
- [ ] MySQL にスキーマ作成 (`pc_inventory`)
- [ ] データベースユーザー作成・権限設定
- [ ] テーブル作成SQLスクリプト作成
- [ ] 接続確認

#### 成果物
- `init.sql` - DB初期構築スクリプト
- DB構築手順書

#### SQL構造
```sql
CREATE DATABASE pc_inventory CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE USER 'dbuser'@'localhost' IDENTIFIED BY 'dbpassword';
GRANT ALL PRIVILEGES ON pc_inventory.* TO 'dbuser'@'localhost';
FLUSH PRIVILEGES;

USE pc_inventory;

CREATE TABLE pc_info (
    id INT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(100) UNIQUE NOT NULL,
    mac_address VARCHAR(17),
    network_type VARCHAR(20),
    user_name VARCHAR(50),
    ip_address VARCHAR(15),
    os VARCHAR(100),
    os_version VARCHAR(100),
    model_name VARCHAR(100),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
```

#### テスト
- テーブル作成確認
- 接続テスト

---

## フェーズ2: 共通基盤実装

### チケット #3: 共通エラー定義

**担当**: 全体  
**工数**: 0.5日  
**依存**: #1  
**優先度**: 高

#### タスク
- [ ] サーバー側 `error.rs` 作成
- [ ] クライアント側 `error.rs` 作成
- [ ] カスタムエラー型定義（thiserror使用）
- [ ] エラーハンドリングパターン統一

#### 成果物
- `server/src/error.rs`
- `client/src/error.rs`

#### サーバー側エラー型
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}
```

#### クライアント側エラー型
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("WMI error: {0}")]
    WmiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("API error: {0}")]
    ApiError(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

---

### チケット #4: 共通設定管理

**担当**: 全体  
**工数**: 1日  
**依存**: #1  
**優先度**: 高

#### タスク
- [ ] サーバー側 `config.rs` 作成
- [ ] クライアント側 `config.rs` 作成
- [ ] config.toml テンプレート作成
- [ ] 設定値バリデーション実装
- [ ] 設定ファイル読み込みテスト

#### 成果物
- `server/src/config.rs`
- `client/src/config.rs`
- `server/config.toml.template`
- `client/config.toml.template`

#### サーバー側設定構造
```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
    pub api: ApiSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
}

// 他の設定構造体...
```

#### クライアント側設定構造
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    pub server: ServerSettings,
    pub client: ClientSettings,
    pub retry: RetrySettings,
    pub pc_info: PcInfoSettings,
    pub logging: LoggingSettings,
}

// 各設定構造体...
```

#### テスト
- 設定ファイル読み込みテスト
- バリデーションテスト
- 不正な設定値のエラーハンドリング

---

## フェーズ3: サーバー側実装

### チケット #5: データモデル作成

**担当**: サーバー側開発  
**工数**: 0.5日  
**依存**: #2, #3, #4  
**優先度**: 高

#### タスク
- [ ] `models/pc_info.rs` 作成
- [ ] PcInfo 構造体定義
- [ ] リクエスト/レスポンスDTO定義
- [ ] SQLx用のデータベースマッピング

#### 成果物
- `server/src/models/pc_info.rs`

#### データモデル構造
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, Serialize)]
pub struct PcInfoResponse {
    pub status: String,
    pub action: String,
    pub id: i32,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}
```

---

### チケット #6: リポジトリ作成

**担当**: サーバー側開発  
**工数**: 1日  
**依存**: #5  
**優先度**: 高

#### タスク
- [ ] `db/repository.rs` 作成
- [ ] データベース接続プール設定
- [ ] UUID検索クエリ実装
- [ ] 新規登録クエリ実装
- [ ] 更新クエリ実装
- [ ] トランザクション管理

#### 成果物
- `server/src/db/repository.rs`

#### リポジトリ構造
```rust
use sqlx::{MySqlPool, Result};
use chrono::Utc;
use crate::models::pc_info::{PcInfo, PcInfoRequest};

pub struct PcInfoRepository {
    pool: MySqlPool,
}

impl PcInfoRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
    
    pub async fn find_by_uuid(&self, uuid: &str) -> Result<Option<PcInfo>> {
        // UUID検索実装
    }
    
    pub async fn create(&self, request: &PcInfoRequest) -> Result<i32> {
        // 新規登録実装
    }
    
    pub async fn update(&self, id: i32, request: &PcInfoRequest) -> Result<()> {
        // 更新実装
    }
}
```

#### テスト
- 接続プールの動作確認
- CRUD操作のテスト
- トランザクションのテスト

---

### チケット #7: APIハンドラー作成

**担当**: サーバー側開発  
**工数**: 1.5日  
**依存**: #6  
**優先度**: 高

#### タスク
- [ ] `handlers/pc_info.rs` 作成
- [ ] POST /api/pc-info エンドポイント実装
- [ ] リクエストバリデーション
- [ ] レスポンス生成
- [ ] エラーハンドリング

#### 成果物
- `server/src/handlers/pc_info.rs`
- `server/src/main.rs` (ルーティング設定)

#### ハンドラー構造
```rust
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::models::pc_info::{PcInfoRequest, PcInfoResponse, ErrorResponse};
use crate::db::repository::PcInfoRepository;

pub async fn handle_pc_info(
    State(repo): State<PcInfoRepository>,
    Json(payload): Json<PcInfoRequest>,
) -> Result<Json<PcInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    // ハンドラー実装
}
```

#### メイン構造
```rust
use axum::{
    routing::post,
    Router,
};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // 設定読み込み
    // DB接続プール作成
    // ログ初期化
    
    let app = Router::new()
        .route("/api/pc-info", post(handle_pc_info))
        .layer(TraceLayer::new_for_http())
        .with_state(repository);
    
    // サーバー起動
}
```

#### テスト
- curlでのAPIテスト
- 新規登録テスト
- 更新テスト
- エラーケーステスト

---

### チケット #8: ログ実装

**担当**: サーバー側開発  
**工数**: 0.5日  
**依存**: #7  
**優先度**: 中

#### タスク
- [ ] tracing初期化
- [ ] ファイル出力設定
- [ ] ログローテーション設定
- [ ] 各処理へのログ追加

#### 成果物
- ログ設定コード
- ログ出力の動作確認

#### ログ初期化
```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn init_logging(config: &LoggingSettings) {
    let file_appender = RollingFileAppender::new(
        Rotation::NEVER,
        ".",
        &config.file,
    );
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.level))
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .init();
}
```

#### ログ出力例
```rust
tracing::info!("Server started on {}:{}", config.host, config.port);
tracing::info!("Created new record. ID: {}, UUID: {}", id, uuid);
tracing::error!("Database error: {}", err);
```

---

### チケット #9: サーバー統合テスト

**担当**: サーバー側開発  
**工数**: 0.5日  
**依存**: #8  
**優先度**: 高

#### タスク
- [ ] エンドツーエンドテスト
- [ ] 正常系テスト
- [ ] 異常系テスト
- [ ] ログ出力確認

#### 成果物
- テスト結果レポート

#### テストケース
1. 新規登録（正常系）
2. 更新（正常系）
3. 必須項目欠損（異常系）
4. UUID形式不正（異常系）
5. DB接続エラー（異常系）

---

## フェーズ4: クライアント側実装

### チケット #10: WMI情報取得

**担当**: クライアント側開発  
**工数**: 1.5日  
**依存**: #3, #4  
**優先度**: 高

#### タスク
- [ ] `wmi/collector.rs` 作成
- [ ] UUID取得実装
- [ ] 機種名取得実装
- [ ] OS情報取得実装
- [ ] エラーハンドリング

#### 成果物
- `client/src/wmi/collector.rs`

#### WMI収集構造
```rust
use wmi::{COMLibrary, WMIConnection};
use crate::error::ClientError;

pub struct WmiCollector {
    wmi_con: WMIConnection,
}

impl WmiCollector {
    pub fn new() -> Result<Self, ClientError> {
        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;
        Ok(Self { wmi_con })
    }
    
    pub fn get_uuid(&self) -> Result<String, ClientError> {
        // Win32_BaseBoard.SerialNumber
    }
    
    pub fn get_model_name(&self) -> Result<String, ClientError> {
        // Win32_ComputerSystem.Model
    }
    
    pub fn get_os_info(&self) -> Result<(String, String), ClientError> {
        // Win32_OperatingSystem.Caption, Version
    }
}
```

#### テスト
- 各項目の取得確認
- エラーハンドリング確認
- 複数PC環境での動作確認

---

### チケット #11: ネットワーク情報取得

**担当**: クライアント側開発  
**工数**: 1.5日  
**依存**: #3, #4  
**優先度**: 高

#### タスク
- [ ] `network/detector.rs` 作成
- [ ] アクティブアダプタ検出
- [ ] IPアドレス取得実装
- [ ] MACアドレス取得・正規化実装
- [ ] NetworkType判定実装

#### 成果物
- `client/src/network/detector.rs`

#### ネットワーク検出構造
```rust
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use crate::error::ClientError;

pub struct NetworkDetector;

impl NetworkDetector {
    pub fn get_active_adapter() -> Result<NetworkInfo, ClientError> {
        // デフォルトゲートウェイを持つアダプタを検出
    }
}

pub struct NetworkInfo {
    pub ip_address: String,
    pub mac_address: String,
    pub network_type: String,
}
```

#### テスト
- 有線環境での取得確認
- 無線環境での取得確認
- MACアドレス形式の確認
- エラーハンドリング確認

---

### チケット #12: API通信クライアント

**担当**: クライアント側開発  
**工数**: 1日  
**依存**: #3, #4  
**優先度**: 高

#### タスク
- [ ] `api/client.rs` 作成
- [ ] HTTP POSTリクエスト実装
- [ ] JSON シリアライズ/デシリアライズ
- [ ] タイムアウト設定
- [ ] エラーハンドリング

#### 成果物
- `client/src/api/client.rs`

#### API通信構造
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::ClientError;

pub struct ApiClient {
    client: Client,
    server_url: String,
    timeout_secs: u64,
}

impl ApiClient {
    pub fn new(server_url: String, timeout_secs: u64) -> Result<Self, ClientError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;
        Ok(Self { client, server_url, timeout_secs })
    }
    
    pub async fn send_pc_info(&self, data: &PcInfoData) -> Result<ApiResponse, ClientError> {
        // POST実装
    }
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub action: String,
    pub id: i32,
}
```

#### テスト
- サーバーへの送信テスト
- タイムアウトテスト
- エラーレスポンステスト

---

### チケット #13: メイン処理・タイマー

**担当**: クライアント側開発  
**工数**: 1.5日  
**依存**: #10, #11, #12  
**優先度**: 高

#### タスク
- [ ] `main.rs` 実装
- [ ] 起動時フロー実装
- [ ] 定期チェックタイマー実装
- [ ] config.toml 更新処理
- [ ] ログ初期化

#### 成果物
- `client/src/main.rs`

#### メイン処理構造
```rust
use tokio::time::{interval, Duration};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 設定読み込み
    // ログ初期化
    // user_nameチェック
    
    // 起動時処理
    if let Err(e) = initial_process().await {
        error!("Initial process failed: {}", e);
    }
    
    // 定期チェックタイマー
    let mut timer = interval(Duration::from_secs(config.check_interval_secs));
    loop {
        timer.tick().await;
        if should_send(&config).await {
            periodic_check().await;
        }
    }
}

async fn initial_process() -> Result<(), ClientError> {
    // WMI情報取得
    // config更新
    // 送信
}

async fn periodic_check() -> Result<(), ClientError> {
    // 定期チェック処理
}
```

#### テスト
- 起動テスト
- 定期チェック動作確認
- 各エラーケースの確認

---

### チケット #14: リトライ処理

**担当**: クライアント側開発  
**工数**: 1日  
**依存**: #13  
**優先度**: 中

#### タスク
- [ ] リトライスケジューラー実装
- [ ] first_retry / second_retry サイクル
- [ ] リトライ状態管理
- [ ] ログ出力

#### 成果物
- リトライ処理コード

#### リトライ構造
```rust
use tokio::time::{sleep, Duration};

enum RetryState {
    None,
    FirstRetry,
    SecondRetry,
}

async fn handle_send_failure(config: &ClientConfig) {
    let mut state = RetryState::FirstRetry;
    
    loop {
        let delay = match state {
            RetryState::FirstRetry => config.retry.first_retry_delay_secs,
            RetryState::SecondRetry => config.retry.second_retry_delay_secs,
            _ => break,
        };
        
        sleep(Duration::from_secs(delay)).await;
        
        if send_pc_info().await.is_ok() {
            break;
        }
        
        state = match state {
            RetryState::FirstRetry => RetryState::SecondRetry,
            RetryState::SecondRetry => RetryState::FirstRetry,
            _ => break,
        };
    }
}
```

#### テスト
- リトライサイクル確認
- 定期チェックとの共存確認

---

### チケット #15: クライアント統合テスト

**担当**: クライアント側開発  
**工数**: 0.5日  
**依存**: #14  
**優先度**: 高

#### タスク
- [ ] エンドツーエンドテスト
- [ ] 各シナリオのテスト
- [ ] ログ出力確認
- [ ] config更新確認

#### 成果物
- テスト結果レポート

#### テストケース
1. 初回起動（正常系）
2. user_name未入力（異常系）
3. WMI取得失敗（異常系）
4. サーバー接続失敗（異常系）
5. 定期チェック（正常系）
6. リトライサイクル（異常系）

---

## フェーズ5: システム統合テスト

### チケット #16: システム統合テスト

**担当**: 全体  
**工数**: 1日  
**依存**: #9, #15  
**優先度**: 高

#### タスク
- [ ] サーバー・クライアント連携テスト
- [ ] 複数クライアント同時送信テスト
- [ ] 長時間稼働テスト
- [ ] ネットワーク障害時のテスト

#### 成果物
- 統合テスト結果レポート
- 不具合リスト

#### テストシナリオ
1. 10台のクライアント同時起動
2. 各種ネットワーク環境
3. サーバー再起動時の挙動
4. DB接続エラー時の挙動
5. 長時間稼働（8時間以上）

---

### チケット #17: 不具合修正

**担当**: 全体  
**工数**: 1～2日  
**依存**: #16  
**優先度**: 高

#### タスク
- [ ] 統合テストで発見された不具合の修正
- [ ] 修正後の再テスト

#### 成果物
- 修正済みコード
- 修正内容の記録

---

## フェーズ6: ドキュメント・デプロイ

### チケット #18: ドキュメント作成

**担当**: 全体  
**工数**: 1日  
**依存**: #17  
**優先度**: 高

#### タスク
- [ ] サーバー構築手順書
- [ ] クライアント配布・セットアップ手順書
- [ ] トラブルシューティングガイド
- [ ] システム運用マニュアル
- [ ] Excel クエリ接続手順書

#### 成果物
```
docs/
├── server_setup.md
├── client_setup.md
├── troubleshooting.md
├── operation_manual.md
└── excel_query_guide.md
```

---

### チケット #19: デプロイパッケージ作成

**担当**: 全体  
**工数**: 0.5日  
**依存**: #17  
**優先度**: 高

#### タスク
- [ ] サーバー側バイナリビルド
- [ ] クライアント側バイナリビルド
- [ ] config.toml テンプレート同梱
- [ ] README 作成

#### 成果物
```
releases/
├── server/
│   ├── pc-inventory-server
│   ├── config.toml.template
│   └── README.md
└── client/
    ├── pc-inventory-client.exe
    ├── config.toml.template
    └── README.md
```

#### ビルドコマンド
```bash
# サーバー側
cd server
cargo build --release

# クライアント側
cd client
cargo build --release --target x86_64-pc-windows-gnu
```

---

### チケット #20: 本番環境構築

**担当**: インフラ/運用  
**工数**: 0.5日  
**依存**: #18, #19  
**優先度**: 高

#### タスク
- [ ] 本番サーバー環境構築
- [ ] MySQL セットアップ
- [ ] アプリケーションデプロイ
- [ ] 動作確認

#### 成果物
- 本番環境

---

### チケット #21: パイロット運用

**担当**: 全体  
**工数**: 1週間  
**依存**: #20  
**優先度**: 高

#### タスク
- [ ] 少数のPCでパイロット運用開始
- [ ] データ収集状況確認
- [ ] 問題点の洗い出し
- [ ] 必要に応じて微修正

#### 成果物
- パイロット運用報告書

---

## 開発スケジュール案（1人開発の場合）

### Week 1
- **Day 1**: #1, #2（環境構築・DB）
- **Day 2**: #3, #4（共通基盤）
- **Day 3**: #5, #6（データモデル・リポジトリ）
- **Day 4-5**: #7, #8（APIハンドラー・ログ）

### Week 2
- **Day 1**: #9（サーバー統合テスト）
- **Day 2**: #10（WMI情報取得）
- **Day 3**: #11（ネットワーク情報取得）
- **Day 4**: #12（API通信クライアント）
- **Day 5**: #13（メイン処理・タイマー）

### Week 3
- **Day 1**: #14（リトライ処理）
- **Day 2**: #15（クライアント統合テスト）
- **Day 3**: #16, #17（統合テスト・不具合修正）
- **Day 4**: #18, #19（ドキュメント・パッケージ）
- **Day 5**: #20（本番環境構築）

### Week 4-5
- **Week 4**: #21（パイロット運用開始）
- **Week 5**: パイロット運用継続・問題対応

---

## 優先度別チケット分類

### 高優先度（MVP）
- #1, #2: 環境構築
- #3, #4: 共通基盤
- #5, #6, #7: サーバー基本機能
- #10, #11, #12, #13: クライアント基本機能
- #9, #15, #16: 各テスト
- #18, #19, #20: デプロイ準備

### 中優先度
- #8: ログ出力
- #14: リトライ処理

### 低優先度（後回し可能）
- なし（全て必要な機能）

---

## チケット進捗管理テンプレート

```markdown
## チケット #XX: [タイトル]

**ステータス**: 未着手 / 進行中 / レビュー中 / 完了  
**担当者**: [名前]  
**開始日**: YYYY-MM-DD  
**完了予定日**: YYYY-MM-DD  
**実完了日**: YYYY-MM-DD

### 進捗メモ
- [日付] 着手開始
- [日付] [進捗内容]
- [日付] 完了

### 課題・懸念事項
- [課題があれば記載]

### レビューコメント
- [レビュー結果]
```

---

## 変更履歴

| バージョン | 日付 | 変更内容 | 担当者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-07 | 初版作成（Spring Boot + C#） | - |
| 2.0 | 2025-10-21 | Rustへの全面移行 | - |
| 2.1 | 2025-10-21 | コーディング規約統一、設定外部化、最小限実装 | - |
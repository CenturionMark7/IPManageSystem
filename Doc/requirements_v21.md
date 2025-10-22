# PC情報収集システム 要件定義書

バージョン: 2.1  
最終更新日: 2025-10-21

---

## 1. システム概要

ネットワーク内に所属するPCの情報を自動収集し、データベースに登録・更新するシステム。

### 構成
- **サーバー**: Rust による REST API
- **クライアント**: Rust によるCLI常駐アプリ
- **データベース**: MySQL (将来SQL Serverへの移行可能性を考慮)
- **データ参照**: Excel のクエリ機能を使用

### 技術スタック選定理由
- GUI不要のため、軽量で高速なRustを採用
- クロスコンパイルによる単一実行ファイル配布が可能
- メモリ安全性とパフォーマンスの両立
- Spring BootやC#に比べてリソース消費が少ない

---

## 2. コーディング規約・設計原則

### 2.1 統一性の確保

**命名規則**:
- 変数・関数: `snake_case`
- 型・構造体: `PascalCase`
- 定数: `UPPER_SNAKE_CASE`
- モジュール: `snake_case`

**コード構成の統一**:
```
サーバー側:
src/
├── main.rs           # エントリーポイント
├── config.rs         # 設定ファイル読み込み
├── models/           # データモデル
│   └── pc_info.rs
├── handlers/         # APIハンドラー
│   └── pc_info.rs
├── db/               # データベース操作
│   └── repository.rs
└── error.rs          # エラー定義

クライアント側:
src/
├── main.rs           # エントリーポイント
├── config.rs         # 設定ファイル読み込み
├── wmi/              # Windows情報取得
│   └── collector.rs
├── network/          # ネットワーク情報取得
│   └── detector.rs
├── api/              # API通信
│   └── client.rs
└── error.rs          # エラー定義
```

**エラーハンドリングの統一**:
- すべてのエラーは `Result<T, E>` 型で返す
- カスタムエラー型を定義し、`thiserror` クレートで実装
- エラーは適切にログ出力し、ユーザーにわかりやすいメッセージを表示

**ログ出力の統一**:
- `tracing` クレートを使用
- ログレベル: `trace`, `debug`, `info`, `warn`, `error`
- フォーマット: `[YYYY-MM-DD HH:MM:SS] [LEVEL] message`

### 2.2 必要最小限の実装

**DRY原則の徹底**:
- 共通処理は関数・モジュールに抽出
- 設定値の重複排除（すべて設定ファイルから読み込み）
- コードの再利用を最優先

**YAGNI原則の適用**:
- 現時点で必要な機能のみ実装
- 将来の拡張性は設計で担保、実装は最小限

**依存クレートの最小化**:
- 機能が重複するクレートは避ける
- 標準ライブラリで実現可能な場合は標準ライブラリを優先
- 必要最小限のfeatureのみ有効化

### 2.3 設定外部化の徹底

**ハードコーディング禁止事項**:
- データベース接続情報
- サーバーURL、ポート番号
- タイムアウト値、リトライ間隔
- ログレベル、ログファイルパス
- ユーザー入力値（使用者名など）
- その他すべての環境依存値

**設定ファイル必須項目**:
- すべての接続情報
- すべての動作パラメータ
- すべてのパス情報
- すべてのユーザー入力値

**設定変更時の動作**:
- 設定ファイル変更後、アプリケーション再起動で反映
- 設定ファイル読み込みエラー時は起動を中止し、エラー内容を明示

---

## 3. データベース設計

### スキーマ名
`pc_inventory`

### テーブル名
`pc_info`

### テーブル構造

| カラム名 | データ型 | 制約 | 備考 |
|---------|---------|------|------|
| ID | Int | PK, Auto Increment | DBが自動採番 |
| UUID | String(100) | Unique | マザーボードシリアル番号（WMI経由） |
| MACアドレス | String(17) | - | 現在アクティブなNICのMACアドレス |
| NetworkType | String(20) | - | "Wired" または "Wireless" |
| 使用者名 | String(50) | - | config.toml から取得 |
| IPアドレス | String(15) | - | IPv4アドレス |
| OS | String(100) | - | OS名 |
| OSVer | String(100) | - | OSバージョン |
| 機種名 | String(100) | - | PC機種名 |
| 初回登録日 | DateTime | - | 初回登録時のみ設定 |
| 最終更新日 | DateTime | - | 更新時に自動設定 |

---

## 4. API仕様

### エンドポイント
```
POST /api/pc-info
Content-Type: application/json
```

### リクエスト形式
```json
{
  "uuid": "AB12CD34EF56",
  "mac_address": "AA:BB:CC:DD:EE:FF",
  "network_type": "Wired",
  "user_name": "山田太郎",
  "ip_address": "192.168.1.100",
  "os": "Microsoft Windows 10 Pro",
  "os_version": "10.0.19045",
  "model_name": "HP ProDesk 600 G3 SFF"
}
```

### レスポンス形式

#### 成功 (200 OK)
```json
{
  "status": "success",
  "action": "created",
  "id": 123
}
```
- `action`: "created" (新規登録) または "updated" (更新)

#### エラー (400 Bad Request / 500 Internal Server Error)
```json
{
  "status": "error",
  "message": "エラーの詳細メッセージ"
}
```

### サーバー側処理ロジック
1. リクエストボディから UUID を取得
2. UUID でデータベースを検索
3. **存在しない場合**: 新規レコードとして登録
   - すべてのフィールドを設定
   - 初回登録日、最終更新日を現在時刻で設定
4. **存在する場合**: 該当レコードを更新
   - 以下のフィールドを更新:
     - MACアドレス
     - NetworkType
     - 使用者名
     - IPアドレス
     - OS
     - OSVer
     - 機種名
     - 最終更新日（現在時刻）
   - 初回登録日は更新しない

### エラーハンドリング
- 必須項目の欠損: 400 Bad Request
- UUID形式不正: 400 Bad Request
- DB接続エラー: 500 Internal Server Error

---

## 5. 設定ファイル仕様

### 5.1 サーバー側: config.toml

```toml
[server]
host = "0.0.0.0"
port = 8080
request_timeout_secs = 30

[database]
url = "mysql://dbuser:dbpassword@localhost:3306/pc_inventory"
max_connections = 10
connection_timeout_secs = 5
idle_timeout_secs = 600

[logging]
level = "info"  # trace, debug, info, warn, error
file = "server.log"
max_file_size_mb = 100
max_backup_files = 5

[api]
endpoint_path = "/api/pc-info"

# Future: Encryption settings
[security]
# enable_tls = false
# tls_cert_path = ""
# tls_key_path = ""
# encryption_key = ""
```

**配置場所**: 実行ファイルと同じディレクトリ

**必須項目**: すべて（コメントアウト項目を除く）

**検証**:
- 起動時に設定ファイルの存在確認
- 必須項目の欠損チェック
- 値の妥当性チェック（ポート番号、URL形式など）

### 5.2 クライアント側: config.toml

```toml
[server]
url = "http://192.168.1.10:8080/api/pc-info"
request_timeout_secs = 30

[client]
last_send_datetime = ""
check_interval_secs = 3600        # 1時間ごとの定期チェック
send_interval_secs = 21600        # 6時間ごとに送信

[retry]
first_retry_delay_secs = 900      # 15分
second_retry_delay_secs = 3600    # 1時間

[pc_info]
user_name = ""
uuid = ""
mac_address = ""
network_type = ""
ip_address = ""
os = ""
os_version = ""
model_name = ""

[logging]
level = "info"  # trace, debug, info, warn, error
file = "client.log"
max_file_size_mb = 50
max_backup_files = 3

# Future: Encryption settings
[security]
# enable_tls = false
# encryption_key = ""
```

**配置場所**: 実行ファイルと同じディレクトリ

**必須項目**: 
- `[server]` セクション全項目
- `[pc_info].user_name`（初回起動前にユーザーが入力）
- `[client]`, `[retry]`, `[logging]` セクション全項目

**自動更新項目**:
- `[client].last_send_datetime`
- `[pc_info]` セクションのWMI取得項目

**検証**:
- 起動時に設定ファイルの存在確認
- `user_name` が空の場合はエラーメッセージを表示して終了
- 必須項目の欠損チェック
- 値の妥当性チェック（URL形式、数値範囲など）

---

## 6. クライアント動作仕様

### 6.1 初期セットアップ

#### 配布物
- 実行ファイル (pc-inventory-client.exe)
- config.toml (テンプレート)
- README.txt (セットアップ手順)

#### セットアップ手順
1. クライアント一式を各PCに配布
2. ユーザーまたは管理者が config.toml を開く
3. `[server]` セクションの `url` を確認（必要に応じて変更）
4. `[pc_info]` セクションの `user_name` に使用者名を入力
5. 保存して閉じる
6. 手動でスタートアップに登録（手順書を別途作成）

### 6.2 起動時の動作フロー

```
1. config.toml を読み込み
   → 読み込み失敗: エラーメッセージを表示して終了
   ↓
2. 設定値の妥当性チェック
   → 不正な値: エラーメッセージを表示して終了
   ↓
3. user_name が空欄かチェック
   → 空欄の場合: エラーメッセージを表示して終了
   ↓
4. WMI でPC情報を取得
   - UUID (Win32_BaseBoard.SerialNumber)
   - 機種名 (Win32_ComputerSystem.Model)
   - OS (Win32_OperatingSystem.Caption)
   - OSVer (Win32_OperatingSystem.Version)
   - ネットワーク情報（デフォルトゲートウェイを持つアダプタ）:
     * IPアドレス (IPv4)
     * MACアドレス
     * NetworkType (Wired/Wireless)
   ↓
5. 取得成功した項目を config.toml に書き込み
   ↓
6. [pc_info] のすべての項目が埋まっているかチェック
   → 不足がある場合: ログ出力して送信せず待機
   → すべて揃っている場合: 次へ
   ↓
7. サーバーへデータ送信
   → 成功: last_send_datetime を更新
   → 失敗: リトライサイクルへ
   ↓
8. タイマー起動（check_interval_secsごとに定期チェック）
```

### 6.3 常駐中の定期チェック

**チェック間隔**: `config.toml` の `check_interval_secs` で設定

```
1. last_send_datetime から send_interval_secs 以上経過しているかチェック
   ↓
2. 経過していない場合: 何もせず次回チェックまで待機
   ↓
3. 経過している場合:
   a. WMI で PC情報を再取得
   b. 取得成功した項目を config.toml に更新
   c. [pc_info] のすべての項目が埋まっているかチェック
      → すべて揃っている場合のみ送信
      → 成功: last_send_datetime を更新
      → 失敗: リトライサイクルへ
```

### 6.4 送信失敗時のリトライサイクル

**リトライ間隔**: `config.toml` の `first_retry_delay_secs` と `second_retry_delay_secs` で設定

```
送信失敗
  ↓
first_retry_delay_secs 後にリトライ
  ↓
  成功 → last_send_datetime更新、定期チェックモードへ
  失敗 ↓
second_retry_delay_secs 後にリトライ
  ↓
  成功 → last_send_datetime更新、定期チェックモードへ
  失敗 ↓
first_retry_delay_secs 後にリトライ
  ↓
  成功 → last_send_datetime更新、定期チェックモードへ
  失敗 ↓
second_retry_delay_secs 後にリトライ
  （以下、first→secondのサイクルを繰り返す）
```

**注意事項**:
- 定期チェックのタイマー（check_interval_secsごと）は常に稼働
- リトライ中でも定期チェックのタイミングは独立して動作

---

## 7. データ取得仕様

### 7.1 Windows情報取得（WMI経由）

| 項目 | WMIクラス.プロパティ | 例 |
|------|---------------------|-----|
| UUID | Win32_BaseBoard.SerialNumber | "AB12CD34EF56" |
| 機種名 | Win32_ComputerSystem.Model | "HP ProDesk 600 G3 SFF" |
| OS | Win32_OperatingSystem.Caption | "Microsoft Windows 10 Pro" |
| OSVer | Win32_OperatingSystem.Version | "10.0.19045" |

**Rust実装**:
- `wmi` クレートを使用
- WMIクエリのエラーハンドリング
- 取得失敗時は既存値を保持

### 7.2 ネットワーク情報取得

#### 対象アダプタの選択基準
1. 物理ネットワークアダプタのみ（仮想アダプタ除外）
2. デフォルトゲートウェイを持つ
3. アクティブ状態
4. 複数該当する場合: メトリック値が最小のもの

#### 取得項目

**IPアドレス**:
- IPv4アドレスを取得

**MACアドレス**:
- 形式: `AA:BB:CC:DD:EE:FF` (大文字、コロン区切り)
- クライアント側で正規化処理を実施

**NetworkType**:
- アダプタの種類で判定
  - Ethernet → "Wired"
  - Wireless → "Wireless"

**Rust実装**:
- `network-interface` または `pnet` クレートを使用
- Windows APIの直接呼び出し（必要に応じて）
- 取得失敗時は既存値を保持

---

## 8. ログ出力仕様

### 8.1 サーバー側

**設定**: `config.toml` の `[logging]` セクションで管理

**出力内容**:
- アプリケーション起動/停止
- 設定ファイル読み込み結果
- API リクエスト受信（UUID、IPアドレス、アクション種別）
- DB操作結果
  - 新規登録: `[INFO] Created new record. ID: 123, UUID: AB12CD34EF56`
  - 更新: `[INFO] Updated record. ID: 123, UUID: AB12CD34EF56`
- エラー発生時の詳細

**ログローテーション**:
- `max_file_size_mb` を超えたら新しいファイルに切り替え
- `max_backup_files` の数だけ保持

**Rust実装**:
- `tracing` + `tracing-subscriber` クレートを使用
- `tracing-appender` でファイル出力とローテーション

### 8.2 クライアント側

**設定**: `config.toml` の `[logging]` セクションで管理

**出力内容**:
- アプリケーション起動
- 設定ファイル読み込み結果
- user_name 未入力エラー
- WMI データ取得開始/完了/失敗
- config.toml 更新
- データ送信開始/完了/失敗
- last_send_datetime 更新
- リトライスケジュール情報
- 次回チェック予定時刻
- エラー詳細

**ログローテーション**:
- `max_file_size_mb` を超えたら新しいファイルに切り替え
- `max_backup_files` の数だけ保持

**Rust実装**:
- `tracing` + `tracing-subscriber` クレートを使用
- `tracing-appender` でファイル出力とローテーション

---

## 9. Rust実装における技術スタック

### 9.1 サーバー側推奨クレート

```toml
[dependencies]
# Web Framework
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }

# Database
sqlx = { version = "0.7", features = ["mysql", "runtime-tokio-native-tls", "chrono"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = "0.13"
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"
```

### 9.2 クライアント側推奨クレート

```toml
[dependencies]
# HTTP Client
reqwest = { version = "0.11", features = ["json"] }

# Async Runtime
tokio = { version = "1", features = ["full"] }

# Configuration
config = "0.13"
toml = "0.8"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Windows WMI
wmi = "0.13"

# Network Information
network-interface = "1.1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_System_SystemInformation",
    "Win32_NetworkManagement_IpHelper"
] }
```

---

## 10. セキュリティ要件

### 現時点では実装しない機能

以下は将来対応として設定ファイルに項目のみ用意:
- TLS 通信（server.url を https に変更可能な設計）
- 暗号化キーによる通信の暗号化

### 認証・認可
現時点では実装しない

---

## 11. 非機能要件

### 11.1 パフォーマンス
- サーバー起動時間: 1秒以内
- API レスポンス時間: 100ms以内（通常時）
- クライアントメモリ使用量: 10MB以下
- サーバーメモリ使用量: 50MB以下

### 11.2 データ参照
- Excelのクエリ機能を使用してテーブル全体を取得
- システム側では参照機能を提供しない

### 11.3 設定管理
- すべての設定値は外部ファイル（config.toml）で管理
- ハードコーディングは一切行わない
- 設定ファイルの妥当性チェックを起動時に実施

### 11.4 データベース移行対応
- DB接続情報はすべて設定ファイル (config.toml) に集約
- SQLxの機能を活用し、将来的にSQL Serverへの移行が可能な設計

### 11.5 スタートアップ登録
- 自動登録機能は実装しない
- 手動設定の手順書を別途作成

### 11.6 クロスコンパイル
- Windows x86_64向けにビルド
- 単一実行ファイルとして配布（依存ライブラリの静的リンク）

---

## 12. システム制約事項

### 対象OS
- Windows 10/11 (x86_64)

### ネットワーク構成
- 固定IPアドレスを使用
- セグメント単位で部署・拠点が分かれている

### PC命名規則
- すべてのPCが同じホスト名で統一されている

---

## 13. リスクと注意事項

### 技術的リスク

**サーバー側**:
- MySQLの自動採番とSQLxの連携 → 早期にテスト
- 非同期処理のエラーハンドリング → レビュー強化
- データベース接続プールの管理
- 設定ファイルの破損対策 → バリデーション強化

**クライアント側**:
- WMI でのシリアル番号取得失敗（空白や"Default string"） → 十分なテスト
- 仮想マシンでの動作 → 対象外とするか、代替手段の検討
- ネットワークアダプタの識別ロジック → 複数環境でのテスト
- Windows APIの互換性 → 対象OSバージョンの明確化
- 設定ファイルの破損対策 → バリデーション強化

### 運用リスク

**データ品質**:
- マザーボード交換時の扱い（UUID変更）→ 運用ルールの明確化
- 使用者名の入力ミス → バリデーションルールの検討
- ネットワーク障害時のデータ欠損 → リトライ処理の重要性

**スケーラビリティ**:
- 大量のクライアントからの同時アクセス → 負荷テストの実施
- データベースの肥大化 → 定期的なメンテナンス計画

**設定管理**:
- 設定ファイルの誤編集 → 十分なドキュメント作成
- 設定値の一括変更 → 配布スクリプトの検討

---

## 14. 今後の拡張可能性

### 将来実装候補（現時点では対象外）

**セキュリティ強化**:
- TLS 通信
- 暗号化キーによる通信の暗号化
- API認証（APIキー、JWT など）

**機能追加**:
- Web UI での参照機能（Rust製SPAフレームワーク: Yew、Leptos等）
- データのエクスポート機能
- アラート機能（長期間未更新PCの検出など）
- クライアントバージョン管理・自動アップデート
- より詳細なハードウェア情報収集（CPU、メモリ、ディスク容量など）

**運用改善**:
- 設定ファイルの動的リロード
- データベースバックアップ自動化
- 監視・モニタリング機能
- Dockerコンテナ化
- 設定ファイルの一括配布ツール

---

## 変更履歴

| バージョン | 日付 | 変更内容 | 担当者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-07 | 初版作成（Spring Boot + C#） | - |
| 2.0 | 2025-10-21 | Rustへの全面移行 | - |
| 2.1 | 2025-10-21 | コーディング規約・設計原則追加、設定外部化の徹底 | - |
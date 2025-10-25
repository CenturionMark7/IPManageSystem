# PC情報収集システム サーバー v2.1

**PC Inventory Server** - クライアントから送信されるPC情報を収集・保存するサーバーアプリケーション

---

## パッケージ内容

```
server/
├── pc-inventory-server.exe    # サーバー実行ファイル
├── config.toml.template        # 設定ファイルテンプレート
└── README.md                   # このファイル
```

---

## クイックスタート

### 1. 前提条件

- **OS**: Windows Server 2019以降 / Windows 10/11 Pro
- **データベース**: MySQL 8.0以降 または MariaDB 10.4以降
- **ネットワーク**: 固定IPアドレス、ポート8080開放

### 2. データベースセットアップ

MySQLに接続して以下のSQLを実行:

```sql
-- データベース作成
CREATE DATABASE pc_inventory CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ユーザー作成（パスワードは変更してください）
CREATE USER 'pc_inv_user'@'localhost' IDENTIFIED BY 'your_strong_password';
GRANT ALL PRIVILEGES ON pc_inventory.* TO 'pc_inv_user'@'localhost';
FLUSH PRIVILEGES;

-- データベースを選択
USE pc_inventory;

-- テーブル作成
CREATE TABLE pc_info (
    id INT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(100) UNIQUE NOT NULL COMMENT 'PC固有ID（マザーボードシリアル）',
    mac_address VARCHAR(17) COMMENT 'MACアドレス',
    network_type VARCHAR(20) COMMENT 'ネットワーク種別（Wired/Wireless）',
    user_name VARCHAR(50) COMMENT '使用者名',
    ip_address VARCHAR(15) COMMENT 'IPアドレス',
    os VARCHAR(100) COMMENT 'OS名',
    os_version VARCHAR(100) COMMENT 'OSバージョン',
    model_name VARCHAR(100) COMMENT 'PC機種名',
    created_at DATETIME NOT NULL COMMENT '初回登録日時',
    updated_at DATETIME NOT NULL COMMENT '最終更新日時',
    INDEX idx_uuid (uuid),
    INDEX idx_updated_at (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

### 3. 設定ファイルの作成

```cmd
copy config.toml.template config.toml
```

`config.toml`を編集してデータベース接続情報を設定:

```toml
[database]
# データベース接続URL（パスワードを変更してください）
url = "mysql://pc_inv_user:your_strong_password@localhost:3306/pc_inventory"
```

### 4. サーバーの起動

#### 手動起動（テスト用）

```cmd
pc-inventory-server.exe
```

起動成功メッセージを確認:
```
[INFO] Starting PC Inventory Server...
[INFO] Database connection established
[INFO] Server listening on 0.0.0.0:8080
```

#### サービスとして登録（本番環境推奨）

NSSM (Non-Sucking Service Manager) を使用:

```cmd
nssm install PCInventoryServer "C:\path\to\pc-inventory-server.exe"
nssm set PCInventoryServer AppDirectory "C:\path\to"
nssm set PCInventoryServer DisplayName "PC Inventory Server"
nssm set PCInventoryServer Start SERVICE_AUTO_START
nssm start PCInventoryServer
```

### 5. ファイアウォール設定

Windows Defender ファイアウォールでポート8080を開放:

1. コントロールパネル > Windows Defender ファイアウォール > 詳細設定
2. 受信の規則 > 新しい規則
3. ポート > TCP > 8080
4. 接続を許可する
5. 名前: "PC Inventory Server"

---

## 動作確認

### APIテスト (PowerShell)

```powershell
Invoke-WebRequest -Uri "http://localhost:8080/api/pc-info" -Method POST -ContentType "application/json" -Body '{
  "uuid": "TEST-UUID-12345",
  "mac_address": "AA:BB:CC:DD:EE:FF",
  "network_type": "Wired",
  "user_name": "テストユーザー",
  "ip_address": "192.168.1.100",
  "os": "Microsoft Windows 10 Pro",
  "os_version": "10.0.19045",
  "model_name": "Test Model"
}'
```

成功レスポンス:
```json
{
  "status": "success",
  "action": "created",
  "id": 1
}
```

### データベース確認

```sql
SELECT * FROM pc_info ORDER BY created_at DESC LIMIT 10;
```

---

## トラブルシューティング

### サーバーが起動しない

**症状**: `Error: Os { code: 10048, kind: AddrInUse }`

**対処**:
```cmd
netstat -ano | findstr :8080
taskkill /PID [プロセスID] /F
```

### データベース接続エラー

**対処チェックリスト**:
- [ ] MySQLサービスが起動しているか (`sc query MySQL`)
- [ ] `config.toml`の接続情報が正しいか
- [ ] データベースとテーブルが存在するか
- [ ] ユーザー権限が正しいか

---

## ログファイル

- **場所**: `server.log` (実行ファイルと同じディレクトリ)
- **ログレベル**: INFO, WARN, ERROR

### 正常起動のログ例

```
[INFO] Starting PC Inventory Server...
[INFO] Configuration loaded from: config.toml
[INFO] Connecting to database...
[INFO] Database connection established
[INFO] Server listening on 0.0.0.0:8080
[INFO] API endpoint: POST /api/pc-info
```

---

## 詳細ドキュメント

詳しい情報は以下のドキュメントを参照:

- **サーバー構築手順書**: `docs/server_setup.md`
- **運用マニュアル**: `docs/operation_manual.md`
- **トラブルシューティングガイド**: `docs/troubleshooting.md`

---

## サポート

問題が発生した場合:

1. `server.log`を確認
2. エラーメッセージを記録
3. システム管理者に連絡

---

**バージョン**: 2.1
**リリース日**: 2025-10-25

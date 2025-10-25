# サーバー構築手順書

**プロジェクト**: PC情報収集システム v2.1
**対象**: サーバー側アプリケーション
**最終更新**: 2025-10-25

---

## 1. 前提条件

### 必要な環境
- **OS**: Windows Server 2019以降 / Windows 10/11 Pro
- **データベース**: MySQL 8.0以降 または MariaDB 10.4以降
- **ネットワーク**: 固定IPアドレス、ポート8080開放

### 必要なソフトウェア
- MySQL Server (https://dev.mysql.com/downloads/mysql/)
- または XAMPP (https://www.apachefriends.org/jp/index.html)

---

## 2. データベースセットアップ

### 2.1 MySQLのインストール

XAMPPを使用する場合:
1. XAMPPをダウンロードしてインストール
2. XAMPPコントロールパネルを起動
3. Apache と MySQL を起動

### 2.2 データベースの作成

1. MySQL WorkbenchまたはphpMyAdminを開く

2. 以下のSQLを実行:

```sql
-- データベース作成
CREATE DATABASE pc_inventory CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ユーザー作成（本番環境では強力なパスワードを設定）
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

3. テーブルが作成されたことを確認:
```sql
SHOW TABLES;
DESCRIBE pc_info;
```

---

## 3. サーバーアプリケーションのデプロイ

### 3.1 ファイル配置

1. リリースパッケージから以下のファイルをサーバーに配置:

```
C:\IPManageSystem\server\
├── pc-inventory-server.exe    # サーバー実行ファイル
├── config.toml                 # 設定ファイル
└── server.log                  # ログファイル（自動生成）
```

### 3.2 設定ファイルの編集

`config.toml`を編集:

```toml
[server]
host = "0.0.0.0"                # すべてのネットワークインターフェースでリッスン
port = 8080                      # ポート番号（必要に応じて変更）
request_timeout_secs = 30

[database]
# データベース接続URL
# 形式: mysql://ユーザー名:パスワード@ホスト:ポート/データベース名
url = "mysql://pc_inv_user:your_strong_password@localhost:3306/pc_inventory"
max_connections = 10
connection_timeout_secs = 5
idle_timeout_secs = 600

[logging]
level = "info"                   # ログレベル: trace, debug, info, warn, error
file = "server.log"
max_file_size_mb = 100
max_backup_files = 5

[api]
endpoint_path = "/api/pc-info"
```

**重要**: データベース接続URLの `your_strong_password` を実際のパスワードに変更してください。

---

## 4. サーバーの起動

### 4.1 手動起動（テスト用）

1. コマンドプロンプトを開く
2. サーバーディレクトリに移動:
```cmd
cd C:\IPManageSystem\server
```

3. サーバーを起動:
```cmd
pc-inventory-server.exe
```

4. 起動成功メッセージを確認:
```
[INFO] Starting PC Inventory Server...
[INFO] Configuration loaded from: config.toml
[INFO] Connecting to database...
[INFO] Database connection established
[INFO] Server listening on 0.0.0.0:8080
[INFO] API endpoint: POST /api/pc-info
```

### 4.2 サービスとして登録（本番環境推奨）

Windowsサービスとして登録するには、NSSM（Non-Sucking Service Manager）を使用:

1. NSSMをダウンロード: https://nssm.cc/download
2. 管理者権限でコマンドプロンプトを開く
3. 以下のコマンドを実行:

```cmd
nssm install PCInventoryServer "C:\IPManageSystem\server\pc-inventory-server.exe"
nssm set PCInventoryServer AppDirectory "C:\IPManageSystem\server"
nssm set PCInventoryServer DisplayName "PC Inventory Server"
nssm set PCInventoryServer Description "PC情報収集システム サーバー"
nssm set PCInventoryServer Start SERVICE_AUTO_START
nssm start PCInventoryServer
```

4. サービスの状態を確認:
```cmd
sc query PCInventoryServer
```

---

## 5. 動作確認

### 5.1 ログファイルの確認

`server.log`を開いて、エラーがないことを確認:
```
[INFO] Server listening on 0.0.0.0:8080
[INFO] API endpoint: POST /api/pc-info
```

### 5.2 APIエンドポイントのテスト

PowerShellで以下のコマンドを実行:
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

### 5.3 データベースの確認

MySQL Workbenchまたはphp MyAdminで確認:
```sql
SELECT * FROM pc_info ORDER BY created_at DESC LIMIT 10;
```

---

## 6. ファイアウォール設定

### Windows Defenderファイアウォールでポート8080を開放:

1. コントロールパネル > システムとセキュリティ > Windows Defender ファイアウォール
2. 「詳細設定」をクリック
3. 「受信の規則」を右クリック > 「新しい規則」
4. ポート > TCP > 特定のローカルポート: 8080
5. 接続を許可する
6. すべてのプロファイルを選択（ドメイン、プライベート、パブリック）
7. 名前: "PC Inventory Server" > 完了

---

## 7. 運用開始後の監視

### 7.1 定期的な確認項目

- **ログファイル**: `server.log`のサイズと内容
- **データベース接続**: 接続エラーがないか
- **ディスク容量**: ログとデータベースの増加を監視

### 7.2 ログローテーション

現在の設定:
- 最大ファイルサイズ: 100MB
- 保持ファイル数: 5個

合計最大容量: 500MB

---

## 8. トラブルシューティング

詳細は `troubleshooting.md` を参照してください。

### よくある問題

**問題**: サーバーが起動しない
- データベース接続URLが正しいか確認
- MySQLサービスが起動しているか確認
- ポート8080が他のプログラムで使用されていないか確認

**問題**: クライアントから接続できない
- ファイアウォール設定を確認
- サーバーのIPアドレスが正しいか確認
- サーバーのログでエラーを確認

---

## 9. バックアップ

### データベースのバックアップ

定期的にデータベースをバックアップ:
```cmd
mysqldump -u pc_inv_user -p pc_inventory > backup_YYYYMMDD.sql
```

リストア:
```cmd
mysql -u pc_inv_user -p pc_inventory < backup_YYYYMMDD.sql
```

---

**作成者**: システム管理者
**承認日**: 2025-10-25
**ドキュメントバージョン**: 1.0

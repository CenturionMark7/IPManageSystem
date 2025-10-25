# PC情報収集システム

バージョン: 2.1

## 概要

ネットワーク内に所属するPCの情報を自動収集し、データベースに登録・更新するシステム。

## 構成

- **サーバー**: Rust による REST API (`/server`)
- **クライアント**: Rust によるCLI常駐アプリ (`/client`)
- **データベース**: MySQL/MariaDB
- **ドキュメント**: `/Doc`

## 技術スタック

- Rust 1.90.0
- MySQL/MariaDB 10.4+
- Axum (サーバー側Webフレームワーク)
- SQLx (データベースアクセス)

## プロジェクト構造

```
IPManageSystem/
├── server/          # サーバー側プロジェクト
│   ├── Cargo.toml
│   ├── src/
│   └── config.toml.template
├── client/          # クライアント側プロジェクト
│   ├── Cargo.toml
│   ├── src/
│   └── config.toml.template
├── docs/            # システムドキュメント
└── Doc/             # 要件定義・チケット管理
    ├── requirements_v21.md
    └── tickets_v21.md
```

## セットアップ手順

### 1. 必要なツール

- Rust 1.90.0以上
- MySQL 8.0以上 または MariaDB 10.4以上
- Git
- Visual Studio Code (推奨)

### 2. データベースセットアップ

データベースとテーブルを作成します：

```sql
-- データベース作成
CREATE DATABASE pc_inventory CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ユーザー作成（本番環境推奨）
CREATE USER 'pc_inventory_user'@'localhost' IDENTIFIED BY 'YOUR_STRONG_PASSWORD';
GRANT SELECT, INSERT, UPDATE ON pc_inventory.* TO 'pc_inventory_user'@'localhost';
FLUSH PRIVILEGES;

-- テーブル作成
USE pc_inventory;

CREATE TABLE pc_info (
    id INT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(100) UNIQUE NOT NULL,
    mac_address VARCHAR(17),
    network_type VARCHAR(20),
    user_name VARCHAR(100),
    ip_address VARCHAR(45),
    os VARCHAR(100),
    os_version VARCHAR(50),
    model_name VARCHAR(100),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    INDEX idx_uuid (uuid),
    INDEX idx_updated_at (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

### 3. サーバー側セットアップ

```bash
cd server

# 設定ファイルをテンプレートからコピー
cp config.toml.template config.toml

# config.toml を編集
# - [database] の url にデータベース接続情報を設定
# - 本番環境では専用ユーザーを使用してください

# ビルド
cargo build --release

# 実行
cargo run --release
```

**config.toml 編集例:**
```toml
[database]
url = "mysql://pc_inventory_user:YOUR_PASSWORD@localhost:3306/pc_inventory"
```

### 4. クライアント側セットアップ

```bash
cd client

# 設定ファイルをテンプレートからコピー
cp config.toml.template config.toml

# config.toml を編集
# - [server] の url にサーバーのアドレスを設定
# - [pc_info] の user_name に使用者名を設定（必須）

# ビルド
cargo build --release

# 実行
cargo run --release
```

**config.toml 編集例:**
```toml
[server]
url = "http://192.168.1.10:8080/api/pc-info"

[pc_info]
user_name = "山田太郎"
```

### 5. 開発環境でのビルド（デバッグモード）

```bash
# サーバー側
cd server
cargo build

# クライアント側
cd client
cargo build
```

## ドキュメント

詳細な要件定義とチケット管理は `/Doc` フォルダを参照してください。

- [要件定義書](Doc/requirements_v21.md)
- [チケット管理](Doc/tickets_v21.md)

## ライセンス

内部利用のみ

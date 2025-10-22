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

## 開発環境セットアップ

### 必要なツール

- Rust 1.90.0以上
- MySQL 8.0以上 または MariaDB 10.4以上
- Git
- Visual Studio Code (推奨)

### ビルド方法

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

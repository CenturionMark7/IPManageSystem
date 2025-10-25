# PC情報収集システム - 設計ドキュメント

**バージョン**: 2.1
**最終更新**: 2025-10-25
**作成目的**: システム理解・Rust学習

---

## 概要

このディレクトリには、PC情報収集システムの設計ドキュメントと詳細注釈付きコードが含まれています。
特にRust学習を目的として、各行の動作や概念を詳細に説明しています。

---

## ドキュメント一覧

### 1. データベース設計

#### `er_diagram.puml`
**ER図（Entity-Relationship Diagram）**

- **内容**: データベーステーブル（pc_info）の構造
- **主な要素**:
  - フィールド定義（id, uuid, mac_address等）
  - ユニーク制約（uuid）
  - インデックス（idx_uuid, idx_updated_at）
  - 監査項目（created_at, updated_at）
- **学習ポイント**:
  - UUID（マザーボードシリアル）による一意性保証
  - MACアドレスは参考情報（ネットワーク切替で変わる）
  - UPSERT処理（INSERT or UPDATE）の設計

### 2. システムアーキテクチャ

#### `architecture_diagram.puml`
**システムアーキテクチャ図**

- **内容**: クライアント・サーバー・データベースの全体構成
- **主な要素**:
  - クライアント側コンポーネント（WMI、Network、API Client）
  - サーバー側コンポーネント（Handler、Repository、Model）
  - 通信プロトコル（HTTP POST /api/pc-info）
  - データフロー（起動時・定期送信・リトライ）
- **学習ポイント**:
  - クライアント-サーバーアーキテクチャ
  - 非同期処理とタイマー制御
  - リトライパターン（15分 → 1時間サイクル）

### 3. コンポーネント設計

#### `component_diagram.puml`
**クライアント側コンポーネント相関図**

- **内容**: クライアントの詳細な構造とコンポーネント間の呼び出し関係
- **主な要素**:
  - main関数のフロー（エントリーポイント）
  - 各モジュールの役割と責務
  - 関数呼び出しの順序
  - データの流れ
- **学習ポイント**:
  - Rustのモジュールシステム
  - async/awaitによる非同期処理
  - Arc/Mutexによるスレッド間共有
  - WMI/ネットワーク情報の取得方法

#### `server_component_diagram.puml`
**サーバー側コンポーネント相関図**

- **内容**: サーバーの詳細な構造とコンポーネント間の呼び出し関係
- **主な要素**:
  - main関数のフロー
  - Axumルーターの構築
  - ハンドラー → リポジトリ → データベースの流れ
  - UPSERT処理の詳細
- **学習ポイント**:
  - Axum Webフレームワークの使い方
  - sqlx接続プールの管理
  - タイプセーフなエクストラクター
  - エラーハンドリングとIntoResponseトレイト

### 4. 処理フロー

#### `sequence_diagram.puml`
**シーケンス図（正常系）**

- **内容**: PC情報送信の時系列フロー
- **主な要素**:
  - 起動時処理の詳細ステップ
  - WMI情報取得
  - ネットワーク情報検出
  - HTTP通信
  - データベースUPSERT
  - 定期チェック（6時間後）
- **学習ポイント**:
  - 非同期処理の実行順序
  - awaitポイントの理解
  - データベーストランザクションの流れ
  - INSERTとUPDATEの分岐条件

### 5. 詳細注釈付きコード

#### `annotated_client_main.rs`
**クライアント main.rs の詳細注釈版**

- **内容**: 各行の動作を詳細に説明したRustコード
- **注釈の内容**:
  - 各構文の説明（use, mod, async fn等）
  - 所有権・借用の動作
  - Arc/Mutexの仕組み
  - tokioの非同期ランタイム
  - リトライサイクルの実装
- **学習ポイント**:
  - Rustの基本構文
  - 所有権システム（ownership, borrowing, move）
  - async/await非同期プログラミング
  - エラーハンドリング（Result, ?演算子）
  - スレッド間共有（Arc, Mutex）

#### `annotated_server_main.rs`
**サーバー main.rs の詳細注釈版**

- **内容**: 各行の動作を詳細に説明したRustコード
- **注釈の内容**:
  - Axumフレームワークの使い方
  - sqlx接続プールの構築
  - ルーティングとミドルウェア
  - トレーシング（ログ）の設定
- **学習ポイント**:
  - Webフレームワークの基礎
  - 非同期SQL処理
  - トレイトベースの設計
  - レイヤーベースのミドルウェア

---

## PlantUMLの表示方法

PlantUML図（.pumlファイル）を表示するには、以下のいずれかの方法を使用してください：

### 方法1: VS Code拡張機能
1. VS CodeにPlantUML拡張機能をインストール
2. .pumlファイルを開く
3. `Alt + D`でプレビュー表示

### 方法2: オンラインエディタ
1. https://www.plantuml.com/plantuml/uml/ にアクセス
2. .pumlファイルの内容をコピー&ペースト
3. 図が自動生成される

### 方法3: ローカルでPNG生成
```bash
# PlantUMLをインストール（要Java）
brew install plantuml  # macOS
# または
apt-get install plantuml  # Linux

# PNG画像を生成
plantuml docs/*.puml
# → docs/xxx.pngファイルが生成される
```

---

## MACアドレス取得に関する重要な仕様

### 質問：MACアドレスはマザーボードのものか？

**回答**: **いいえ、LANモジュール（ネットワークアダプタ）のMACアドレスです。**

### 詳細説明

#### UUID（マザーボード）
- **取得元**: WMI `Win32_BaseBoard.SerialNumber`
- **特性**: PC固有、ハードウェア交換しない限り不変
- **用途**: レコードの一意性を保証（UNIQUE制約）

#### MACアドレス（LANモジュール）
- **取得元**: ネットワークインターフェース（`network-interface`クレート）
- **特性**: 有線と無線で異なる、接続方法で変わる
- **用途**: 参考情報（更新される）

#### 設計上の対策

データベースのユニーク制約は**UUIDのみ**に設定されているため：
- 同じPCが有線 → 無線に切り替えても、**同じレコードが更新される**
- MACアドレスが変わっても、UUIDで識別するため問題なし
- 複数レコードとして重複登録されることはない

#### 実例

| 状況 | UUID | MACアドレス | ネットワーク種別 | 動作 |
|------|------|-------------|------------------|------|
| 初回起動（有線） | 37C03080-... | 00:11:22:33:44:55 | Ethernet | INSERT（新規） |
| 無線に切替 | 37C03080-... | AA:BB:CC:DD:EE:FF | Wi-Fi | UPDATE（既存） |
| 次回起動（無線） | 37C03080-... | AA:BB:CC:DD:EE:FF | Wi-Fi | UPDATE（既存） |

→ UUIDが同じなので、常に同じレコード（ID=1等）が更新される

---

## Rust学習のポイント

このプロジェクトで学べるRustの主要概念：

### 基礎

1. **所有権システム (Ownership)**
   - 各値には1つのオーナーがいる
   - 所有権の移動（move）と借用（borrow）
   - `&T`（不変参照）、`&mut T`（可変参照）

2. **Result型とエラーハンドリング**
   - `Result<T, E>`: 成功/失敗を型で表現
   - `?演算子`: エラーの伝播
   - `match式`: パターンマッチング

3. **トレイト (Trait)**
   - インターフェース的な機能
   - `Debug`, `Clone`, `Send`, `Sync`
   - カスタムトレイト実装

### 非同期プログラミング

4. **async/await**
   - `async fn`: 非同期関数
   - `.await`: 完了を待つ
   - `tokio`: 非同期ランタイム

5. **並行処理**
   - `Arc<T>`: スレッド間共有
   - `Mutex<T>`: 排他制御
   - `tokio::spawn`: タスク起動

### Webプログラミング

6. **Axum Webフレームワーク**
   - ルーティング
   - エクストラクター（State, Json）
   - ミドルウェア（TraceLayer）

7. **sqlx**
   - 非同期SQL
   - 接続プール
   - コンパイル時クエリチェック

---

## ドキュメント使用ガイド

### システム全体を理解したい場合
1. `architecture_diagram.puml` - 全体像
2. `sequence_diagram.puml` - 処理フロー
3. `er_diagram.puml` - データ構造

### クライアント実装を理解したい場合
1. `component_diagram.puml` - コンポーネント構造
2. `annotated_client_main.rs` - 詳細コード注釈
3. 実際のソースコード（client/src/main.rs）

### サーバー実装を理解したい場合
1. `server_component_diagram.puml` - コンポーネント構造
2. `annotated_server_main.rs` - 詳細コード注釈
3. 実際のソースコード（server/src/main.rs）

### Rustを学習したい場合
1. `annotated_client_main.rs` - 基礎概念（所有権、async/await）
2. `annotated_server_main.rs` - Webフレームワーク（Axum）
3. 実際にコードを書いて動かす

---

## 参考リンク

### Rust公式
- [The Rust Programming Language (日本語)](https://doc.rust-jp.rs/book-ja/)
- [Rust By Example (日本語)](https://doc.rust-jp.rs/rust-by-example-ja/)
- [Rust API Documentation](https://doc.rust-lang.org/std/)

### 使用クレート
- [tokio](https://docs.rs/tokio/) - 非同期ランタイム
- [axum](https://docs.rs/axum/) - Webフレームワーク
- [sqlx](https://docs.rs/sqlx/) - 非同期SQL
- [tracing](https://docs.rs/tracing/) - ログフレームワーク
- [serde](https://docs.rs/serde/) - シリアライズ/デシリアライズ
- [wmi](https://docs.rs/wmi/) - Windows Management Instrumentation
- [network-interface](https://docs.rs/network-interface/) - ネットワーク情報取得

---

## 質問・フィードバック

このドキュメントに関する質問や改善提案があれば、プロジェクトのIssueで報告してください。

---

**作成者**: Claude Code
**作成日**: 2025-10-25

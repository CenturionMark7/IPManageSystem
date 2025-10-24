# チケット #9: サーバー統合テスト レポート

バージョン: 2.1
実施日: 2025-10-25
担当: サーバー側開発

---

## テスト概要

PC情報収集システムのサーバー側APIに対するエンドツーエンドテストを実施しました。

### テスト環境

- **OS**: Windows
- **Rustバージョン**: stable
- **データベース**: MySQL (XAMPP)
- **サーバーURL**: http://localhost:8080
- **APIエンドポイント**: POST /api/pc-info

---

## テストケース一覧

### テストケース1: 新規登録（正常系）

**目的**: 新しいPC情報を正常に登録できることを確認

**テストデータ**:
```json
{
  "uuid": "test-uuid-001",
  "mac_address": "00:11:22:33:44:55",
  "network_type": "Ethernet",
  "user_name": "testuser",
  "ip_address": "192.168.1.100",
  "os": "Windows 11 Pro",
  "os_version": "10.0.22631",
  "model_name": "Test Model 001"
}
```

**期待される結果**:
- HTTPステータスコード: 200 OK
- レスポンス: `{"status": "success", "action": "created", "id": <新規ID>}`
- データベースに新しいレコードが作成される
- ログに「Creating new PC info. UUID: test-uuid-001」が記録される

**結果**: ✅ PASS / ❌ FAIL

**備考**:

---

### テストケース2: 更新（正常系）

**目的**: 既存のPC情報を正常に更新できることを確認

**テストデータ**:
```json
{
  "uuid": "test-uuid-001",
  "mac_address": "00:11:22:33:44:66",
  "network_type": "Wi-Fi",
  "user_name": "testuser",
  "ip_address": "192.168.1.101",
  "os": "Windows 11 Pro",
  "os_version": "10.0.22631",
  "model_name": "Test Model 001 Updated"
}
```

**期待される結果**:
- HTTPステータスコード: 200 OK
- レスポンス: `{"status": "success", "action": "updated", "id": <既存ID>}`
- データベースの既存レコードが更新される
- ログに「Updating existing PC info. ID: <ID>, UUID: test-uuid-001」が記録される

**結果**: ✅ PASS / ❌ FAIL

**備考**:

---

### テストケース3: 別のPC新規登録（正常系）

**目的**: 複数のPCを同時に管理できることを確認

**テストデータ**:
```json
{
  "uuid": "test-uuid-002",
  "mac_address": "AA:BB:CC:DD:EE:FF",
  "network_type": "Ethernet",
  "user_name": "testuser2",
  "ip_address": "192.168.1.200",
  "os": "Windows 10 Pro",
  "os_version": "10.0.19045",
  "model_name": "Test Model 002"
}
```

**期待される結果**:
- HTTPステータスコード: 200 OK
- レスポンス: `{"status": "success", "action": "created", "id": <新規ID>}`
- データベースに2つのレコードが存在する

**結果**: ✅ PASS / ❌ FAIL

**備考**:

---

### テストケース4: 必須項目欠損（異常系）

**目的**: UUIDが空の場合にエラーハンドリングが正しく動作することを確認

**テストデータ**:
```json
{
  "uuid": "",
  "mac_address": "00:11:22:33:44:55",
  "network_type": "Ethernet",
  "user_name": "testuser",
  "ip_address": "192.168.1.100",
  "os": "Windows 11 Pro",
  "os_version": "10.0.22631",
  "model_name": "Test Model"
}
```

**期待される結果**:
- HTTPステータスコード: 400 Bad Request
- レスポンス: `{"status": "error", "message": "UUID cannot be empty"}`
- データベースにレコードは作成されない
- ログに「Invalid request: UUID cannot be empty」が記録される

**結果**: ✅ PASS / ❌ FAIL

**備考**:

---

### テストケース5: 不正なJSON（異常系）

**目的**: JSON形式が不正な場合にエラーハンドリングが正しく動作することを確認

**テストデータ**:
```
{invalid json}
```

**期待される結果**:
- HTTPステータスコード: 400 Bad Request または 422 Unprocessable Entity
- エラーレスポンスが返される
- データベースにレコードは作成されない

**結果**: ✅ PASS / ❌ FAIL

**備考**:

---

## ログ出力確認

### 確認項目

- [ ] サーバー起動時のログ
  - 設定ファイル読み込み
  - データベース接続
  - サーバーリッスン開始
- [ ] リクエスト処理のログ
  - UUID検索
  - レコード作成
  - レコード更新
- [ ] エラー時のログ
  - バリデーションエラー
  - データベースエラー

### ログサンプル

```
[実際のログ出力をここに記載]
```

---

## データベース確認

### 実行SQLクエリ

```sql
-- テスト後のデータ確認
SELECT * FROM pc_info ORDER BY id;

-- レコード数確認
SELECT COUNT(*) FROM pc_info;

-- 特定UUIDの確認
SELECT * FROM pc_info WHERE uuid = 'test-uuid-001';
SELECT * FROM pc_info WHERE uuid = 'test-uuid-002';
```

### データベース状態

```
[クエリ実行結果をここに記載]
```

---

## 発見された問題

### 問題1

**説明**:

**重要度**: 高 / 中 / 低

**対応方法**:

---

## テスト実行手順

### 前提条件

1. XAMPPまたはMySQLサーバーが起動している
2. データベース `pc_inventory` が作成されている
3. テーブル `pc_info` が作成されている

### 実行方法

#### サーバー起動

```bash
cd server
cargo run
```

#### テスト実行（別ターミナル）

Windowsの場合:
```cmd
cd server
test_api.bat
```

Linux/Macの場合:
```bash
cd server
chmod +x test_api.sh
./test_api.sh
```

---

## 総評

**テスト実施状況**:
- 実施日: YYYY-MM-DD
- 正常系テスト: X/X 成功
- 異常系テスト: X/X 成功
- 総合結果: ✅ PASS / ❌ FAIL

**コメント**:

---

## 次のステップ

- [ ] 発見された問題の修正
- [ ] 追加テストケースの実施
- [ ] クライアント側実装への移行（チケット #10）

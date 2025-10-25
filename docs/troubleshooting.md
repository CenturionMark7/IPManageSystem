# トラブルシューティングガイド

**プロジェクト**: PC情報収集システム v2.1
**最終更新**: 2025-10-25

---

## 目次
1. [サーバー側の問題](#1-サーバー側の問題)
2. [クライアント側の問題](#2-クライアント側の問題)
3. [ネットワーク関連の問題](#3-ネットワーク関連の問題)
4. [データベース関連の問題](#4-データベース関連の問題)
5. [ログの確認方法](#5-ログの確認方法)

---

## 1. サーバー側の問題

### 問題 1-1: サーバーが起動しない

#### 症状
```
Error: Os { code: 10048, kind: AddrInUse, message: "..." }
```

#### 原因
ポート8080が既に使用されている

#### 対処方法
1. 使用中のプロセスを確認:
```cmd
netstat -ano | findstr :8080
```

2. プロセスを終了:
```cmd
taskkill /PID [プロセスID] /F
```

3. サーバーを再起動

---

### 問題 1-2: データベースに接続できない

#### 症状
```
[ERROR] Database error: ...
```

#### 対処方法チェックリスト

**□ MySQLサービスが起動しているか確認**:
```cmd
sc query MySQL
```
または XAMPPコントロールパネルでMySQLを起動

**□ 接続情報が正しいか確認** (`config.toml`):
- ユーザー名
- パスワード
- データベース名
- ポート番号(通常3306)

**□ データベースとテーブルが存在するか確認**:
```sql
SHOW DATABASES;
USE pc_inventory;
SHOW TABLES;
```

**□ ユーザー権限を確認**:
```sql
SHOW GRANTS FOR 'pc_inv_user'@'localhost';
```

---

### 問題 1-3: ログファイルが肥大化

#### 症状
`server.log`が数百MBに達している

#### 対処方法
1. サーバーを停止
2. 古いログファイルをアーカイブ:
```cmd
move server.log server.log.old
```
3. サーバーを再起動（新しいログファイルが自動生成される）

**恒久対策**: `config.toml`のログ設定を確認:
```toml
[logging]
max_file_size_mb = 100
max_backup_files = 5
```

---

## 2. クライアント側の問題

### 問題 2-1: user_name未入力エラー

#### 症状
```
Error: MissingField("user_name is required. Please set your name in config.toml")
```

#### 対処方法
1. `config.toml`を開く
2. `[pc_info]`セクションの`user_name`に名前を入力:
```toml
[pc_info]
user_name = "山田太郎"
```
3. 保存してクライアントを再起動

---

### 問題 2-2: サーバーに接続できない

#### 症状
```
[ERROR] API error: error sending request for url (http://...)
[INFO] Starting retry cycle
```

#### 対処方法チェックリスト

**□ サーバーが起動しているか確認**

**□ サーバーURLが正しいか確認** (`config.toml`):
```toml
[server]
url = "http://192.168.1.10:8080/api/pc-info"
```

**□ ネットワーク接続を確認**:
```cmd
ping サーバーのIPアドレス
```

**□ ポート8080への接続を確認**:
```powershell
Test-NetConnection -ComputerName サーバーのIPアドレス -Port 8080
```

**□ ファイアウォール設定を確認**

**注意**: クライアントは自動的にリトライを繰り返すため、サーバーが復旧すれば自動的に送信されます。

---

### 問題 2-3: WMI情報が取得できない

#### 症状
```
[ERROR] WMI error: ...
```

#### 対処方法
1. WMIサービスの状態を確認:
```cmd
sc query winmgmt
```

2. WMIサービスを再起動:
```cmd
net stop winmgmt
net start winmgmt
```

3. それでも解決しない場合、WMIリポジトリを再構築:
```cmd
winmgmt /salvagerepository
```

---

### 問題 2-4: ネットワーク情報が取得できない

#### 症状
ログに「Network adapter not found」など

#### 対処方法
1. アクティブなネットワークアダプタを確認:
```cmd
ipconfig /all
```

2. デフォルトゲートウェイが設定されているか確認

3. ネットワークアダプタを再起動:
   - デバイスマネージャー > ネットワークアダプタ > 右クリック > 無効化/有効化

---

### 問題 2-5: クライアントが多重起動している

#### 症状
タスクマネージャーに`pc-inventory-client.exe`が複数表示される

#### 対処方法
1. タスクマネージャーですべてのプロセスを終了
2. タスクスケジューラで重複タスクがないか確認
3. スタートアップフォルダに重複ショートカットがないか確認

---

## 3. ネットワーク関連の問題

### 問題 3-1: ファイアウォールでブロックされる

#### Windows Defenderファイアウォールの設定

**サーバー側（ポート8080を開放）**:
1. コントロールパネル > Windows Defender ファイアウォール
2. 詳細設定 > 受信の規則 > 新しい規則
3. ポート > TCP > 8080 > 接続を許可する

**クライアント側（送信を許可）**:
通常は設定不要（デフォルトで送信許可）

---

### 問題 3-2: プロキシ環境での接続

現在のバージョンではプロキシ未対応。

#### 回避策
サーバーとクライアントを同一ネットワークセグメントに配置

---

## 4. データベース関連の問題

### 問題 4-1: データベースの容量不足

#### 対処方法
1. 古いレコードを削除:
```sql
DELETE FROM pc_info WHERE updated_at < DATE_SUB(NOW(), INTERVAL 1 YEAR);
```

2. データベースを最適化:
```sql
OPTIMIZE TABLE pc_info;
```

---

### 問題 4-2: レコードが重複している

#### 原因
同一UUIDで複数レコードが存在（通常は発生しない）

#### 確認方法
```sql
SELECT uuid, COUNT(*) as count
FROM pc_info
GROUP BY uuid
HAVING count > 1;
```

#### 対処方法
```sql
-- 古い方のレコードを削除（最新のIDを残す）
DELETE t1 FROM pc_info t1
INNER JOIN pc_info t2
WHERE t1.uuid = t2.uuid
AND t1.id < t2.id;
```

---

### 問題 4-3: データベース接続プールの枯渇

#### 症状
サーバーログに「Too many connections」エラー

#### 対処方法
1. MySQLの最大接続数を確認:
```sql
SHOW VARIABLES LIKE 'max_connections';
```

2. `config.toml`の接続プール設定を見直す:
```toml
[database]
max_connections = 10  # 必要に応じて調整
```

3. サーバーを再起動

---

## 5. ログの確認方法

### 5.1 サーバーログ (`server.log`)

**ログの場所**: サーバー実行ファイルと同じディレクトリ

**重要なログレベル**:
- `[INFO]` - 正常な動作
- `[WARN]` - 警告（動作は継続）
- `[ERROR]` - エラー（要対処）

**確認すべきログ**:
```
[INFO] Server listening on 0.0.0.0:8080
[INFO] Creating new PC info. UUID: ...
[INFO] Updating existing PC info. ID: ..., UUID: ...
```

**エラー例**:
```
[ERROR] Database error: ...
```

---

### 5.2 クライアントログ (`client.log`)

**ログの場所**: クライアント実行ファイルと同じディレクトリ

**正常動作の確認**:
```
[INFO] PC Inventory Client starting...
[INFO] WMI information collected: UUID: ...
[INFO] Network information detected: IP: ...
[INFO] PC info sent successfully. Action: updated, ID: ...
[INFO] Server response: success (action: updated, id: ...)
```

**エラー例**:
```
[ERROR] Initial process failed: ...
[ERROR] API error: error sending request
[ERROR] WMI error: ...
```

---

## 6. よくある質問（FAQ）

### Q1: クライアントはいつデータを送信しますか？

**A**:
- 起動時に1回送信
- その後、`send_interval_secs`（デフォルト6時間）ごとに送信
- 送信失敗時は自動リトライ（15分→1時間のサイクル）

### Q2: サーバーを再起動するとどうなりますか？

**A**:
- クライアントは次回の送信タイミングで自動的にサーバーに接続
- リトライ中の場合も、サーバー復旧後に自動送信

### Q3: PCの機種を交換したらどうなりますか？

**A**:
- UUID（マザーボードシリアル）が変わるため、新しいレコードとして登録
- 古いPCのレコードは自動削除されないので、手動で削除が必要

### Q4: ログファイルは自動削除されますか？

**A**:
- 現在はログローテーション未実装
- 定期的に手動でアーカイブまたは削除が必要

---

## 7. サポート連絡先

問題が解決しない場合:
1. ログファイル（`server.log`, `client.log`）を確認
2. エラーメッセージを記録
3. システム管理者に連絡

**緊急連絡先**: システム管理者

---

**作成者**: システム管理者
**承認日**: 2025-10-25
**ドキュメントバージョン**: 1.0

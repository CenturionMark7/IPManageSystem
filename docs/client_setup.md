# クライアントセットアップ手順書

**プロジェクト**: PC情報収集システム v2.1
**対象**: クライアント側アプリケーション
**最終更新**: 2025-10-25

---

## 1. 前提条件

### 対象環境
- **OS**: Windows 10/11 (x86_64)
- **権限**: 管理者権限は不要（通常ユーザーで実行可能）
- **ネットワーク**: サーバーへのHTTP通信が可能

---

## 2. クライアントのインストール

### 2.1 ファイルの配置

1. リリースパッケージから以下のファイルを各PCにコピー:

```
C:\IPManageSystem\client\
├── pc-inventory-client.exe    # クライアント実行ファイル
├── config.toml                 # 設定ファイル
└── client.log                  # ログファイル（自動生成）
```

**推奨配置場所**: `C:\IPManageSystem\client\`

### 2.2 設定ファイルの編集

`config.toml`を**テキストエディタ**で開いて編集:

```toml
[server]
# サーバーのURL（完全なエンドポイントURLを指定）
# 例: http://192.168.1.10:8080/api/pc-info
url = "http://サーバーのIPアドレス:8080/api/pc-info"
request_timeout_secs = 30

[client]
# 最終送信日時（自動更新されます - 編集不要）
last_send_datetime = ""

# チェック間隔（秒） - 定期的にタイマーで確認する間隔
# デフォルト: 3600秒 = 1時間
check_interval_secs = 3600

# 送信間隔（秒） - この時間が経過したら再送信
# デフォルト: 21600秒 = 6時間
send_interval_secs = 21600

[retry]
# 送信失敗時の1回目リトライまでの待機時間（秒）
# デフォルト: 900秒 = 15分
first_retry_delay_secs = 900

# 送信失敗時の2回目リトライまでの待機時間（秒）
# デフォルト: 3600秒 = 1時間
second_retry_delay_secs = 3600

[pc_info]
# 使用者名（必須 - 初回起動前に必ず入力してください）
user_name = "山田太郎"

# 以下の項目は自動取得されるため、空白のままにしてください
uuid = ""
mac_address = ""
network_type = ""
ip_address = ""
os = ""
os_version = ""
model_name = ""

[logging]
# ログレベル: trace, debug, info, warn, error
level = "info"
file = "client.log"
max_file_size_mb = 50
max_backup_files = 3
```

**必須の編集項目**:
1. `[server].url` - サーバーのIPアドレスに変更
2. `[pc_info].user_name` - PCの使用者名を入力

---

## 3. 初回起動テスト

### 3.1 動作確認

1. コマンドプロンプトまたはPowerShellを開く
2. クライアントディレクトリに移動:
```cmd
cd C:\IPManageSystem\client
```

3. クライアントを起動:
```cmd
pc-inventory-client.exe
```

### 3.2 正常起動の確認

ログに以下のメッセージが表示されれば成功:
```
[INFO] PC Inventory Client starting...
[INFO] Configuration loaded from: config.toml
[INFO] Server URL: http://サーバーIP:8080/api/pc-info
[INFO] Running initial process
[INFO] Collecting WMI information
[INFO] WMI information collected:
[INFO]   UUID: xxxxx
[INFO]   Model: xxxxx
[INFO]   OS: Microsoft Windows 10 Pro (10.0.xxxxx)
[INFO] Network information detected:
[INFO]   IP: 192.168.x.x
[INFO]   MAC: xx:xx:xx:xx:xx:xx
[INFO]   Type: Wi-Fi (or Wired)
[INFO] Configuration updated and saved
[INFO] Sending PC information to server
[INFO] PC info sent successfully. Action: created, ID: x
[INFO] Server response: success (action: created, id: x)
[INFO] Last send datetime updated in config
[INFO] Starting periodic check timer (interval: 3600s)
```

### 3.3 config.tomlの更新確認

`config.toml`を開いて、以下の項目が自動入力されているか確認:
- `[client].last_send_datetime` - 送信日時が記録されている
- `[pc_info]` セクションの各項目 - PC情報が入力されている

---

## 4. スタートアップ登録

### 4.1 手動登録（推奨）

クライアントをWindows起動時に自動実行させる手順:

#### 方法1: タスクスケジューラを使用

1. タスクスケジューラを開く
   - Windowsキー + R → `taskschd.msc` → Enter

2. 「基本タスクの作成」をクリック

3. タスク名を入力:
   - 名前: `PC Inventory Client`
   - 説明: `PC情報収集クライアント`

4. トリガー: `コンピューターの起動時`

5. 操作: `プログラムの開始`

6. プログラム/スクリプト:
   ```
   C:\IPManageSystem\client\pc-inventory-client.exe
   ```

7. 開始: (オプション)
   ```
   C:\IPManageSystem\client
   ```

8. 「完了」をクリック

9. タスクのプロパティを開いて以下を設定:
   - 全般タブ:
     - 「ユーザーがログオンしているかどうかにかかわらず実行する」をチェック
     - 「最上位の特権で実行する」をチェック
   - 条件タブ:
     - 「コンピューターをAC電源で使用している場合のみタスクを開始する」のチェックを外す
   - 設定タブ:
     - 「タスクが失敗した場合の再起動の間隔」: 1分
     - 「タスクの再起動の試行回数」: 3

#### 方法2: スタートアップフォルダを使用

1. スタートアップフォルダを開く:
   - Windowsキー + R → `shell:startup` → Enter

2. ショートカットを作成:
   - `pc-inventory-client.exe`を右クリック → 「ショートカットの作成」
   - 作成したショートカットをスタートアップフォルダに移動

**注意**: この方法はユーザーログイン後に実行されます。

---

## 5. 動作の確認

### 5.1 プロセスの確認

タスクマネージャーで`pc-inventory-client.exe`が実行中であることを確認:
1. Ctrl + Shift + Esc でタスクマネージャーを開く
2. 「詳細」タブで`pc-inventory-client.exe`を探す

### 5.2 ログの確認

`client.log`を開いて、定期的にログが出力されていることを確認:
```
[INFO] Starting periodic check timer (interval: 3600s)
[INFO] Send interval elapsed, sending PC info
[INFO] Running periodic check
...
```

---

## 6. アンインストール

### 6.1 クライアントの停止

#### タスクスケジューラを使用している場合:
1. タスクスケジューラを開く
2. `PC Inventory Client`タスクを右クリック → 「無効化」または「削除」

#### スタートアップフォルダを使用している場合:
1. タスクマネージャーを開く
2. `pc-inventory-client.exe`を右クリック → 「タスクの終了」
3. スタートアップフォルダからショートカットを削除

### 6.2 ファイルの削除

`C:\IPManageSystem\client\`フォルダごと削除

---

## 7. トラブルシューティング

### よくある問題

#### 問題1: 「User name is required」エラー

**原因**: config.tomlの`user_name`が空白

**対処**:
1. `config.toml`を開く
2. `[pc_info]`セクションの`user_name`に使用者名を入力
3. 保存してクライアントを再起動

#### 問題2: サーバーに接続できない

**現象**: ログに`ERROR: API error: error sending request`

**対処**:
1. サーバーが起動しているか確認
2. `config.toml`の`[server].url`が正しいか確認
3. ネットワーク接続を確認
4. ファイアウォール設定を確認

クライアントは自動的にリトライを繰り返します:
- 1回目: 15分後
- 2回目: 1時間後
- 以降、15分→1時間を繰り返す

#### 問題3: WMI情報が取得できない

**現象**: ログに`ERROR: WMI error`

**対処**:
1. WMIサービスが実行中か確認:
   ```cmd
   sc query winmgmt
   ```
2. 必要に応じてWMIサービスを再起動:
   ```cmd
   net stop winmgmt
   net start winmgmt
   ```

---

## 8. 設定のカスタマイズ

### チェック間隔の変更

デフォルトでは1時間ごとにチェック、6時間ごとに送信:

頻度を上げる場合（例: 30分ごとにチェック、2時間ごとに送信）:
```toml
[client]
check_interval_secs = 1800    # 30分
send_interval_secs = 7200     # 2時間
```

頻度を下げる場合（例: 2時間ごとにチェック、12時間ごとに送信）:
```toml
[client]
check_interval_secs = 7200    # 2時間
send_interval_secs = 43200    # 12時間
```

**注意**: 設定変更後、クライアントを再起動してください。

---

**作成者**: システム管理者
**承認日**: 2025-10-25
**ドキュメントバージョン**: 1.0

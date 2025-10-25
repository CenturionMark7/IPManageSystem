# PC情報収集システム クライアント v2.1

**PC Inventory Client** - PC情報を自動収集してサーバーに送信するクライアントアプリケーション

---

## パッケージ内容

```
client/
├── pc-inventory-client.exe     # クライアント実行ファイル
├── config.toml.template         # 設定ファイルテンプレート
└── README.md                    # このファイル
```

---

## クイックスタート

### 1. 前提条件

- **OS**: Windows 10/11 (x86_64)
- **権限**: 管理者権限は不要（通常ユーザーで実行可能）
- **ネットワーク**: サーバーへのHTTP通信が可能

### 2. インストール

#### ファイルの配置

推奨配置場所: `C:\IPManageSystem\client\`

```cmd
mkdir C:\IPManageSystem\client
copy pc-inventory-client.exe C:\IPManageSystem\client\
copy config.toml.template C:\IPManageSystem\client\config.toml
```

#### 設定ファイルの編集

`C:\IPManageSystem\client\config.toml`をテキストエディタで開いて編集:

```toml
[server]
# サーバーのURL（IPアドレスを変更してください）
url = "http://192.168.1.10:8080/api/pc-info"

[pc_info]
# 使用者名（必須 - 必ず入力してください）
user_name = "山田太郎"
```

**必須の編集項目**:
1. `[server].url` - サーバーのIPアドレスに変更
2. `[pc_info].user_name` - PCの使用者名を入力

### 3. 動作テスト

コマンドプロンプトで以下を実行:

```cmd
cd C:\IPManageSystem\client
pc-inventory-client.exe
```

### 正常起動の確認

ログに以下のメッセージが表示されれば成功:

```
[INFO] PC Inventory Client starting...
[INFO] WMI information collected: UUID: xxxxx
[INFO] Network information detected: IP: 192.168.x.x
[INFO] PC info sent successfully. Action: created, ID: x
[INFO] Server response: success (action: created, id: x)
[INFO] Starting periodic check timer (interval: 3600s)
```

### 4. 自動起動の設定

#### 方法1: Windowsサービス化（推奨）

**最も堅牢で安定した方法です。**

バックグラウンドで動作し、タスクバーに表示されないため、誤操作による停止を防ぎます。

1. NSSM（Non-Sucking Service Manager）をダウンロード
   - https://nssm.cc/download から最新版をダウンロード
   - `win64`フォルダ内の`nssm.exe`をクライアントフォルダにコピー

2. `install_service.bat`を右クリック → **「管理者として実行」**

3. `start_service.bat`を右クリック → **「管理者として実行」**

4. サービスが起動し、以降は自動的に動作します

**詳細は `SERVICE_SETUP.md` を参照してください。**

**メリット**:
- タスクバーに表示されず、プロセスとして常駐
- Windows起動時に自動起動
- ユーザーログオフ後も動作継続
- 障害時の自動再起動機能

#### 方法2: タスクスケジューラを使用

1. Windows検索で「タスクスケジューラ」を開く
2. 「基本タスクの作成」をクリック
3. 名前: `PC Inventory Client`
4. トリガー: `コンピューターの起動時`
5. 操作: `プログラムの開始`
6. プログラム/スクリプト: `C:\IPManageSystem\client\pc-inventory-client.exe`
7. 開始: `C:\IPManageSystem\client`
8. 完了

**詳細設定**:
- 全般タブ > 「ユーザーがログオンしているかどうかにかかわらず実行する」をチェック
- 条件タブ > 「コンピューターをAC電源で使用している場合のみ」のチェックを外す

#### 方法3: スタートアップフォルダを使用（簡易）

1. Windowsキー + R → `shell:startup`
2. `pc-inventory-client.exe`のショートカットを作成
3. ショートカットをスタートアップフォルダに移動

**注意**: この方法はユーザーログイン後に実行され、コンソールウィンドウが表示されます。

---

## 動作の仕組み

### 自動送信タイミング

- **起動時**: 1回送信
- **定期送信**: 6時間ごとに送信（デフォルト）
- **送信失敗時**: 自動リトライ（15分後 → 1時間後を繰り返す）

### 収集される情報

- **UUID**: マザーボードシリアル（PC固有ID）
- **使用者名**: config.tomlで設定した名前
- **IPアドレス**: 現在のIPアドレス
- **MACアドレス**: ネットワークアダプタのMACアドレス
- **ネットワーク種別**: Wired（有線） または Wi-Fi（無線）
- **OS情報**: OS名とバージョン
- **機種名**: PCのモデル名

---

## 設定のカスタマイズ

### 送信間隔の変更

`config.toml`を編集:

```toml
[client]
# チェック間隔（秒） - デフォルト: 3600秒 = 1時間
check_interval_secs = 3600

# 送信間隔（秒） - デフォルト: 21600秒 = 6時間
send_interval_secs = 21600
```

**例**: 2時間ごとに送信する場合:
```toml
send_interval_secs = 7200
```

### リトライ設定の変更

```toml
[retry]
# 1回目リトライまでの待機時間（秒） - デフォルト: 900秒 = 15分
first_retry_delay_secs = 900

# 2回目リトライまでの待機時間（秒） - デフォルト: 3600秒 = 1時間
second_retry_delay_secs = 3600
```

---

## トラブルシューティング

### 問題1: 「User name is required」エラー

**原因**: config.tomlの`user_name`が空白

**対処**:
1. `config.toml`を開く
2. `[pc_info]`セクションの`user_name`に使用者名を入力
3. 保存してクライアントを再起動

### 問題2: サーバーに接続できない

**症状**: ログに`ERROR: API error: error sending request`

**対処チェックリスト**:
- [ ] サーバーが起動しているか確認
- [ ] `config.toml`の`[server].url`が正しいか確認
- [ ] ネットワーク接続を確認 (`ping サーバーIP`)
- [ ] ファイアウォール設定を確認

**注意**: クライアントは自動的にリトライを繰り返します（15分後 → 1時間後）。

### 問題3: WMI情報が取得できない

**症状**: ログに`ERROR: WMI error`

**対処**:
```cmd
# WMIサービスの確認
sc query winmgmt

# WMIサービスの再起動
net stop winmgmt
net start winmgmt
```

---

## ログファイル

- **場所**: `client.log` (実行ファイルと同じディレクトリ)
- **最大サイズ**: 50MB (デフォルト)
- **バックアップ**: 3世代 (デフォルト)

### 正常動作のログ例

```
[INFO] PC Inventory Client starting...
[INFO] Configuration loaded from: config.toml
[INFO] Server URL: http://192.168.1.10:8080/api/pc-info
[INFO] WMI information collected: UUID: xxxxx
[INFO] Network information detected: IP: 192.168.x.x, MAC: xx:xx:xx:xx:xx:xx
[INFO] PC info sent successfully. Action: updated, ID: 1
[INFO] Starting periodic check timer (interval: 3600s)
```

---

## アンインストール

### クライアントの停止

#### Windowsサービスとして実行している場合:
1. `uninstall_service.bat`を右クリック → **「管理者として実行」**
2. サービスが停止・削除されます

詳細は `SERVICE_SETUP.md` を参照してください。

#### タスクスケジューラを使用している場合:
1. タスクスケジューラを開く
2. `PC Inventory Client`タスクを削除

#### スタートアップフォルダを使用している場合:
1. タスクマネージャーで`pc-inventory-client.exe`を終了
2. スタートアップフォルダからショートカットを削除

### ファイルの削除

```cmd
rmdir /S C:\IPManageSystem\client
```

---

## 詳細ドキュメント

詳しい情報は以下のドキュメントを参照:

- **Windowsサービス化セットアップガイド**: `SERVICE_SETUP.md`（同じフォルダ内）
- **クライアントセットアップ手順書**: `docs/client_setup.md`
- **運用マニュアル**: `docs/operation_manual.md`
- **トラブルシューティングガイド**: `docs/troubleshooting.md`

---

## サポート

問題が発生した場合:

1. `client.log`を確認
2. エラーメッセージを記録
3. システム管理者に連絡

---

**バージョン**: 2.1
**リリース日**: 2025-10-25

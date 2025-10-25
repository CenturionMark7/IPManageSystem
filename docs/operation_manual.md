# 運用マニュアル

**プロジェクト**: PC情報収集システム v2.1
**対象**: システム管理者
**最終更新**: 2025-10-25

---

## 1. 日常運用

### 1.1 毎日の確認事項

**□ サーバー稼働状況の確認**
- サーバープロセスが実行中であることを確認
- タスクマネージャーまたは `sc query PCInventoryServer`

**□ ログファイルのチェック**
- `server.log`にERRORログがないか確認
- 異常なアクセスパターンがないか確認

**□ データベース接続の確認**
- サーバーログで「Database connection established」を確認

---

### 1.2 週次の確認事項

**□ ログファイルサイズの確認**
- `server.log`: 目安100MB以下
- `client.log` (各PC): 目安50MB以下

**□ ディスク容量の確認**
- サーバーマシンの空き容量
- データベースのサイズ

**□ 収集データの確認**
```sql
-- 最近1週間の登録・更新状況
SELECT
    DATE(updated_at) as date,
    COUNT(*) as count,
    COUNT(DISTINCT uuid) as unique_pcs
FROM pc_info
WHERE updated_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)
GROUP BY DATE(updated_at)
ORDER BY date DESC;
```

---

### 1.3 月次の確認事項

**□ データベースのバックアップ**
```cmd
mysqldump -u pc_inv_user -p pc_inventory > backup_YYYYMMDD.sql
```

**□ ログファイルのアーカイブ**
- 古いログファイルを圧縮してアーカイブ
- アーカイブは3ヶ月間保持

**□ システムリソースの確認**
- CPUおよびメモリ使用率
- ディスクI/O
- ネットワーク帯域

**□ 長期間未更新PCの確認**
```sql
-- 1ヶ月以上未更新のPC
SELECT
    user_name,
    ip_address,
    model_name,
    updated_at
FROM pc_info
WHERE updated_at < DATE_SUB(NOW(), INTERVAL 1 MONTH)
ORDER BY updated_at ASC;
```

---

## 2. データ参照

### 2.1 Excelでのデータ参照

#### 手順

1. Microsoft Excelを開く

2. 「データ」タブ > 「データの取得」 > 「データベースから」 > 「MySQLデータベース」

3. 接続情報を入力:
   - サーバー: localhost (または MySQLサーバーのIP)
   - データベース: pc_inventory

4. ユーザー名とパスワードを入力

5. `pc_info`テーブルを選択

6. 「読み込み」をクリック

#### データの自動更新設定

1. 「データ」タブ > 「すべて更新」の横の▼ > 「接続のプロパティ」

2. 「使用」タブ:
   - 「バックグラウンドで更新する」をチェック
   - 「ファイルを開くときにデータを更新する」をチェック
   - 「定期的に更新する」をチェック（例: 60分ごと）

---

### 2.2 よく使うSQL

#### 全PC一覧（最新順）
```sql
SELECT
    user_name as '使用者',
    ip_address as 'IPアドレス',
    mac_address as 'MACアドレス',
    network_type as 'ネットワーク種別',
    model_name as '機種名',
    os as 'OS',
    updated_at as '最終更新'
FROM pc_info
ORDER BY updated_at DESC;
```

#### 部署別集計（IPアドレスのセグメント別）
```sql
SELECT
    SUBSTRING_INDEX(ip_address, '.', 3) as 'ネットワークセグメント',
    COUNT(*) as 'PC台数',
    COUNT(DISTINCT user_name) as '使用者数'
FROM pc_info
GROUP BY SUBSTRING_INDEX(ip_address, '.', 3)
ORDER BY 'PC台数' DESC;
```

#### OS別集計
```sql
SELECT
    os as 'OS',
    COUNT(*) as '台数'
FROM pc_info
GROUP BY os
ORDER BY '台数' DESC;
```

#### ネットワーク種別集計
```sql
SELECT
    network_type as 'ネットワーク種別',
    COUNT(*) as '台数'
FROM pc_info
GROUP BY network_type;
```

---

## 3. メンテナンス作業

### 3.1 サーバーの再起動

#### サービスとして実行している場合:
```cmd
net stop PCInventoryServer
net start PCInventoryServer
```

または

```cmd
sc stop PCInventoryServer
sc start PCInventoryServer
```

#### 手動実行の場合:
1. タスクマネージャーでプロセスを終了
2. 実行ファイルを再実行

---

### 3.2 設定変更

#### サーバー設定の変更

1. サーバーを停止
2. `config.toml`を編集
3. サーバーを再起動
4. ログで設定が反映されたことを確認

#### クライアント設定の変更

**一括変更の手順**:
1. テンプレート`config.toml`を編集
2. 各PCに配布（GPOやスクリプトを使用）
3. クライアントを再起動（または次回のPC再起動時に反映）

---

### 3.3 データベースメンテナンス

#### 古いデータの削除
```sql
-- 2年以上前のデータを削除
DELETE FROM pc_info
WHERE updated_at < DATE_SUB(NOW(), INTERVAL 2 YEAR);
```

#### テーブルの最適化
```sql
OPTIMIZE TABLE pc_info;
```

#### インデックスの再構築
```sql
ALTER TABLE pc_info
DROP INDEX idx_uuid,
ADD INDEX idx_uuid (uuid);

ALTER TABLE pc_info
DROP INDEX idx_updated_at,
ADD INDEX idx_updated_at (updated_at);
```

---

## 4. 障害対応

### 4.1 サーバー障害

#### 症状: サーバーが応答しない

**対処手順**:
1. サーバープロセスの状態を確認
2. ログファイルでエラーを確認
3. データベース接続を確認
4. 必要に応じてサーバーを再起動

#### 症状: データベース接続エラー

**対処手順**:
1. MySQLサービスの状態を確認
2. データベース接続情報（`config.toml`）を確認
3. MySQLログでエラーを確認
4. 必要に応じてMySQLを再起動

---

### 4.2 クライアント障害

#### 症状: クライアントが送信しない

**対処手順**:
1. クライアントプロセスが実行中か確認
2. `client.log`でエラーを確認
3. ネットワーク接続を確認
4. 必要に応じてクライアントを再起動

#### 症状: 大量のクライアントが接続できない

**対処手順**:
1. サーバーの負荷を確認
2. データベース接続プール設定を確認
3. ネットワーク帯域を確認
4. 必要に応じてサーバーのスケールアップを検討

---

## 5. セキュリティ

### 5.1 アクセス制御

#### データベースアクセス
- 本番環境では強力なパスワードを使用
- 必要最小限の権限のみ付与
- 定期的なパスワード変更（推奨: 3ヶ月ごと）

#### ネットワークアクセス
- ファイアウォールで必要なポート（8080, 3306）のみ開放
- 社内ネットワークからのアクセスのみ許可

---

### 5.2 監査ログ

#### サーバーログの保管
- 最低3ヶ月間保管
- 圧縮してアーカイブ
- セキュリティインシデント時の調査に使用

#### データベースログ
```sql
-- アクセスログの有効化（my.ini/my.cnf）
general_log = ON
general_log_file = mysql_query.log
```

---

## 6. パフォーマンス最適化

### 6.1 データベースチューニング

#### インデックスの確認
```sql
SHOW INDEX FROM pc_info;
```

#### クエリパフォーマンスの確認
```sql
EXPLAIN SELECT * FROM pc_info WHERE uuid = 'xxx';
```

---

### 6.2 ログローテーション

現在は手動でログローテーションが必要:

#### サーバーログ
```cmd
move server.log server_%date:~0,4%%date:~5,2%%date:~8,2%.log
```

#### クライアントログ
各PCで定期的に実行（タスクスケジューラ等で自動化可能）

---

## 7. バックアップとリカバリ

### 7.1 バックアップ手順

#### データベースの完全バックアップ
```cmd
mysqldump -u pc_inv_user -p --databases pc_inventory > full_backup_%date:~0,4%%date:~5,2%%date:~8,2%.sql
```

#### 差分バックアップ（増分データのみ）
```sql
-- 最新のバックアップ日時以降のデータをエクスポート
SELECT * FROM pc_info
WHERE updated_at > '2025-10-25 00:00:00'
INTO OUTFILE 'C:/backup/incremental.csv'
FIELDS TERMINATED BY ','
ENCLOSED BY '"'
LINES TERMINATED BY '\n';
```

---

### 7.2 リカバリ手順

#### 完全リストア
```cmd
mysql -u pc_inv_user -p pc_inventory < full_backup_20251025.sql
```

#### データの確認
```sql
SELECT COUNT(*) FROM pc_info;
SELECT MAX(updated_at) FROM pc_info;
```

---

## 8. 拡張と改善

### 8.1 将来の拡張候補

- **TLS通信**: HTTPSへの移行
- **Web UI**: データ参照用のWebインターフェース
- **アラート機能**: 長期間未更新PCの自動通知
- **レポート機能**: 定期レポートの自動生成

### 8.2 パイロット運用からの改善

パイロット運用期間中に以下を監視:
- API応答時間
- データベースサイズの増加率
- ログファイルサイズの増加率
- ネットワーク帯域使用量

---

**作成者**: システム管理者
**承認日**: 2025-10-25
**ドキュメントバージョン**: 1.0

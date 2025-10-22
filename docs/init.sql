-- PC情報収集システム データベース初期構築スクリプト
-- バージョン: 2.1
-- 最終更新: 2025-10-22

-- データベース作成
CREATE DATABASE IF NOT EXISTS pc_inventory
CHARACTER SET utf8mb4
COLLATE utf8mb4_unicode_ci;

-- データベースユーザー作成
-- 注意: 本番環境では強力なパスワードに変更してください
CREATE USER IF NOT EXISTS 'pc_inventory_user'@'localhost' IDENTIFIED BY 'pc_inventory_pass';

-- 権限付与
GRANT ALL PRIVILEGES ON pc_inventory.* TO 'pc_inventory_user'@'localhost';
FLUSH PRIVILEGES;

-- pc_inventoryデータベースを使用
USE pc_inventory;

-- pc_infoテーブル作成
CREATE TABLE IF NOT EXISTS pc_info (
    id INT AUTO_INCREMENT PRIMARY KEY COMMENT 'DBが自動採番する主キー',
    uuid VARCHAR(100) UNIQUE NOT NULL COMMENT 'マザーボードシリアル番号（WMI経由）',
    mac_address VARCHAR(17) COMMENT '現在アクティブなNICのMACアドレス',
    network_type VARCHAR(20) COMMENT 'Wired または Wireless',
    user_name VARCHAR(50) COMMENT '使用者名（config.tomlから取得）',
    ip_address VARCHAR(15) COMMENT 'IPv4アドレス',
    os VARCHAR(100) COMMENT 'OS名',
    os_version VARCHAR(100) COMMENT 'OSバージョン',
    model_name VARCHAR(100) COMMENT 'PC機種名',
    created_at DATETIME NOT NULL COMMENT '初回登録日時',
    updated_at DATETIME NOT NULL COMMENT '最終更新日時',
    INDEX idx_uuid (uuid),
    INDEX idx_mac_address (mac_address),
    INDEX idx_updated_at (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='PC情報管理テーブル';

-- 初期構築完了確認
SELECT 'Database initialization completed successfully!' AS status;
SHOW TABLES;
DESCRIBE pc_info;

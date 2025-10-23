use sqlx::{MySqlPool, Error as SqlxError};
use chrono::Utc;
use crate::models::pc_info::{PcInfo, PcInfoRequest};

/// PC情報のデータベースリポジトリ
#[derive(Clone)]
pub struct PcInfoRepository {
    pool: MySqlPool,
}

impl PcInfoRepository {
    /// 新しいリポジトリインスタンスを作成
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// UUIDでPC情報を検索
    ///
    /// # 引数
    /// * `uuid` - 検索するUUID
    ///
    /// # 戻り値
    /// * `Ok(Some(PcInfo))` - レコードが見つかった場合
    /// * `Ok(None)` - レコードが見つからなかった場合
    /// * `Err(SqlxError)` - データベースエラーが発生した場合
    pub async fn find_by_uuid(&self, uuid: &str) -> Result<Option<PcInfo>, SqlxError> {
        let result = sqlx::query_as::<_, PcInfo>(
            r#"
            SELECT id, uuid, mac_address, network_type, user_name,
                   ip_address, os, os_version, model_name,
                   created_at, updated_at
            FROM pc_info
            WHERE uuid = ?
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// 新しいPC情報レコードを作成
    ///
    /// # 引数
    /// * `request` - PC情報リクエストDTO
    ///
    /// # 戻り値
    /// * `Ok(i32)` - 作成されたレコードのID
    /// * `Err(SqlxError)` - データベースエラーが発生した場合
    pub async fn create(&self, request: &PcInfoRequest) -> Result<i32, SqlxError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO pc_info (
                uuid, mac_address, network_type, user_name,
                ip_address, os, os_version, model_name,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&request.uuid)
        .bind(&request.mac_address)
        .bind(&request.network_type)
        .bind(&request.user_name)
        .bind(&request.ip_address)
        .bind(&request.os)
        .bind(&request.os_version)
        .bind(&request.model_name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    /// 既存のPC情報レコードを更新
    ///
    /// # 引数
    /// * `id` - 更新するレコードのID
    /// * `request` - PC情報リクエストDTO
    ///
    /// # 戻り値
    /// * `Ok(())` - 更新成功
    /// * `Err(SqlxError)` - データベースエラーが発生した場合
    pub async fn update(&self, id: i32, request: &PcInfoRequest) -> Result<(), SqlxError> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE pc_info
            SET mac_address = ?,
                network_type = ?,
                user_name = ?,
                ip_address = ?,
                os = ?,
                os_version = ?,
                model_name = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&request.mac_address)
        .bind(&request.network_type)
        .bind(&request.user_name)
        .bind(&request.ip_address)
        .bind(&request.os)
        .bind(&request.os_version)
        .bind(&request.model_name)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// データベース接続プールを取得
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }
}

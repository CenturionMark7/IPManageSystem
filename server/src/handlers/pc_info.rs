use axum::{extract::State, Json};
use crate::db::repository::PcInfoRepository;
use crate::models::pc_info::{PcInfoRequest, PcInfoResponse};
use crate::error::ServerError;

/// POST /api/pc-info エンドポイントハンドラー
///
/// クライアントから送信されたPC情報を受け取り、
/// UUIDで既存レコードを検索し、新規登録または更新を行う
///
/// # 引数
/// * `State(repo)` - PcInfoRepositoryインスタンス
/// * `Json(payload)` - PC情報リクエストDTO
///
/// # 戻り値
/// * `Ok(Json<PcInfoResponse>)` - 成功時のレスポンス
/// * `Err(ServerError)` - エラー時のレスポンス（自動的にHTTPレスポンスに変換される）
pub async fn handle_pc_info(
    State(repo): State<PcInfoRepository>,
    Json(payload): Json<PcInfoRequest>,
) -> Result<Json<PcInfoResponse>, ServerError> {
    // バリデーション: UUIDが空でないことを確認
    if payload.uuid.trim().is_empty() {
        return Err(ServerError::InvalidRequest(
            "UUID cannot be empty".to_string(),
        ));
    }

    // UUIDで既存レコードを検索
    let existing = repo
        .find_by_uuid(&payload.uuid)
        .await
        .map_err(|e| ServerError::DatabaseError(e))?;

    match existing {
        Some(pc_info) => {
            // 既存レコードが見つかった場合: 更新
            tracing::info!(
                "Updating existing PC info. ID: {}, UUID: {}",
                pc_info.id,
                payload.uuid
            );

            repo.update(pc_info.id, &payload)
                .await
                .map_err(|e| ServerError::DatabaseError(e))?;

            Ok(Json(PcInfoResponse::updated(pc_info.id)))
        }
        None => {
            // 既存レコードが見つからない場合: 新規作成
            tracing::info!("Creating new PC info. UUID: {}", payload.uuid);

            let id = repo
                .create(&payload)
                .await
                .map_err(|e| ServerError::DatabaseError(e))?;

            tracing::info!("Created new PC info. ID: {}, UUID: {}", id, payload.uuid);

            Ok(Json(PcInfoResponse::created(id)))
        }
    }
}

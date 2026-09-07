//! 文件路由 — 真实 gRPC 调用 file-service（部分接口）

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::{get, post}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct FileQuery { page: Option<i32>, page_size: Option<i32>, folder_id: Option<i64>, keyword: Option<String>, r#type: Option<String> }

/// 获取 file-service gRPC 客户端
async fn get_file_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::FileGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.file_client().await
        .map_err(|e| json_error(&format!("file-service 不可用: {e}")))
}

/// proto FileInfo → JSON
fn file_to_json(f: &grpc_proto::file::FileInfo) -> Value {
    json!({
        "id": f.id,
        "filename": f.filename,
        "original_filename": f.original_filename,
        "size": f.size,
        "mime_type": f.mime_type,
        "file_type": f.file_type,
        "folder_id": f.folder_id,
        "created_at": f.created_at,
        "updated_at": f.updated_at,
    })
}

/// proto Folder → JSON
fn folder_to_json(f: &grpc_proto::file::Folder) -> Value {
    json!({
        "id": f.id,
        "name": f.name,
        "parent_id": f.parent_id,
        "created_at": f.created_at,
    })
}

async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FileQuery>,
) -> Json<Value> {
    let mut client = match get_file_client(&state).await { Ok(c) => c, Err(r) => return r };
    let folder = q.folder_id.map(|v| v.to_string()).unwrap_or_default();
    let file_type = q.r#type.as_deref().unwrap_or("").parse::<i32>().unwrap_or(-1);
    match client.list_files(folder, 0, file_type, q.keyword.clone().unwrap_or_default(), q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await {
        Ok(resp) => json_success(json!({
            "list": resp.files.iter().map(file_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "total_size": resp.total_size,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
            "folder_id": q.folder_id,
            "keyword": q.keyword,
            "type": q.r#type,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn list_folders(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_file_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_folders(String::new(), 0).await {
        Ok(resp) => json_success(json!(resp.folders.iter().map(folder_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_file_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_folder(
        body["name"].as_str().unwrap_or("").to_string(),
        body["parent_id"].as_i64().map(|v| v.to_string()).unwrap_or_default(),
        0,
    ).await {
        Ok(resp) => {
            if let Some(f) = resp.folder {
                json_success(folder_to_json(&f))
            } else {
                json_ok()
            }
        }
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn search_files(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FileQuery>,
) -> Json<Value> {
    let mut client = match get_file_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_files(String::new(), 0, -1, q.keyword.clone().unwrap_or_default(), q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await {
        Ok(resp) => json_success(json!({
            "list": resp.files.iter().map(file_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("搜索失败: {e}")),
    }
}

async fn get_file_stats(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_file_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_files(String::new(), 0, -1, String::new(), 1, 1).await {
        Ok(resp) => json_success(json!({"used": resp.total_size, "total": 1073741824, "file_count": resp.total})),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

// ============ 无 file.proto RPC 的接口（保留占位） ============

async fn get_recent_files() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": {"list": [], "total": 0}})) }
async fn get_favorites() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": {"list": [], "total": 0}})) }
async fn upload_file() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": {"id": 0, "url": ""}})) }
async fn upload_signature() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": {"upload_url": "", "upload_method": "PUT"}})) }
async fn share_file() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": {"token": "", "url": ""}})) }
async fn batch_delete_files() -> Json<Value> { json_ok() }
async fn batch_move_files() -> Json<Value> { json_ok() }
async fn batch_copy_files() -> Json<Value> { json_ok() }

async fn get_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_file_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_file(id.to_string(), 0).await {
        Ok(resp) => {
            if let Some(f) = resp.file {
                json_success(file_to_json(&f))
            } else {
                json_error("文件不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}
async fn rename_file(Path(_id): Path<i64>) -> Json<Value> { json_ok() }
async fn move_file(Path(_id): Path<i64>) -> Json<Value> { json_ok() }
async fn copy_file(Path(_id): Path<i64>) -> Json<Value> { json_ok() }
async fn download_file(Path(_id): Path<i64>) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"url": "", "filename": ""}}))
}
async fn preview_file(Path(_id): Path<i64>) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"type": "unknown", "url": ""}}))
}
async fn thumbnail_file(Path(_id): Path<i64>) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"url": ""}}))
}
async fn favorite_file(Path(_id): Path<i64>) -> Json<Value> { json_ok() }
async fn unfavorite_file(Path(_id): Path<i64>) -> Json<Value> { json_ok() }
async fn get_folder_path() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": []})) }
async fn get_share(Path(token): Path<String>) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"token": token, "expires_at": null}}))
}
async fn verify_share(Path(_token): Path<String>) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"valid": true}}))
}
async fn download_share(Path((_token, _file_id)): Path<(String, i64)>) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"url": "", "filename": ""}}))
}
async fn delete_share(Path(_token): Path<String>) -> Json<Value> { json_ok() }

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/files", get(list_files))
        .route("/api/files/search", get(search_files))
        .route("/api/files/recent", get(get_recent_files))
        .route("/api/files/favorites", get(get_favorites))
        .route("/api/files/stats", get(get_file_stats))
        .route("/api/files/folders", get(list_folders).post(create_folder))
        .route("/api/files/folders/path", get(get_folder_path))
        .route("/api/files/upload", post(upload_file))
        .route("/api/files/upload/signature", post(upload_signature))
        .route("/api/files/share", post(share_file))
        .route("/api/files/batch-delete", post(batch_delete_files))
        .route("/api/files/batch-move", post(batch_move_files))
        .route("/api/files/batch-copy", post(batch_copy_files))
        // 单文件操作
        .route("/api/files/{id}", get(get_file))
        .route("/api/files/{id}/rename", axum::routing::put(rename_file))
        .route("/api/files/{id}/move", axum::routing::put(move_file))
        .route("/api/files/{id}/copy", post(copy_file))
        .route("/api/files/{id}/download", get(download_file))
        .route("/api/files/{id}/preview", get(preview_file))
        .route("/api/files/{id}/thumbnail", get(thumbnail_file))
        .route("/api/files/{id}/favorite", post(favorite_file).delete(unfavorite_file))
        // 分享链接操作
        .route("/api/files/share/{token}", get(get_share).delete(delete_share))
        .route("/api/files/share/{token}/verify", post(verify_share))
        .route("/api/files/share/{token}/download/{fileId}", get(download_share))
}

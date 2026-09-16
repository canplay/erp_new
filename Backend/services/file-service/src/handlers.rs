//! HTTP 处理器

use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use futures_util::StreamExt;
use tracing::{error, info};
use std::sync::Arc;

use common::AppError;
use common::AppResult;
use crate::models::{SysFile, FileListQuery, FileListResponse, FileDetailResponse};
use crate::repository::FileRepository;
use crate::storage::{LocalStorage, StorageBackend, generate_file_name};
use crate::helpers::{json_success_msg, json_ok_msg, json_health};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub repository: FileRepository,
    pub storage: Arc<LocalStorage>,
}

/// 创建路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/upload" , axum::routing::post(upload_file))
        .route("/" , axum::routing::get(list_files))
        .route("/{id}" , axum::routing::get(get_file).delete(delete_file))
        .route("/{id}/download" , axum::routing::get(download_file))
        .route("/categories" , axum::routing::get(list_categories))
        .with_state(state)
}

/// 文件上传处理器
///
/// 支持 multipart/form-data 格式上传文件
async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    // 获取上传者 ID（从环境变量获取，默认为 "anonymous"）
    let _uploaded_by = get_uploaded_by();

    // 获取文件分类（从环境变量获取，默认为 "general"）
    let mut category = get_category();

    let mut file_data: Option<FileUploadData> = None;
    let mut metadata: FileMetadata = FileMetadata::default();

    // 解析 multipart 数据
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("解析 multipart 失败: {}" , e);
        AppError::InvalidParam("表单解析失败".to_string())
    })? {
        let field_name = field.name().unwrap_or(" ").to_string();

        match field_name.as_str() {
            "file" => {
                let filename = field.file_name().unwrap_or("unknown" ).to_string();
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream" )
                    .to_string();

                // 读取文件内容
                let mut data = Vec::new();
                let mut stream = field;
                while let Some(chunk) = stream.next().await {
                    data.extend_from_slice(&chunk.map_err(|e| {
                        error!("读取文件数据失败: {}" , e);
                        AppError::InvalidParam("文件读取失败".to_string())
                    })?);
                }

                // 限制最大文件大小（100MB）
                const MAX_SIZE: usize = 100 * 1024 * 1024;
                if data.len() > MAX_SIZE {
                    return Err(AppError::InvalidParam("文件大小超过100MB限制".to_string()));
                }

                metadata.size = data.len() as i64;
                metadata.mime_type = Some(content_type.clone());

                file_data = Some(FileUploadData {
                    filename,
                    data,
                    content_type,
                });
            }
            "name" | "original_name" => {
                metadata.name = Some(read_text_field(field).await?);
            }
            "description" => {
                metadata.description = Some(read_text_field(field).await?);
            }
            "category" => {
                if let Ok(cat) = read_text_field(field).await {
                    category = cat;
                }
            }
            _ => {
                // 忽略未知字段
            }
        }
    }

    let file_data = file_data.ok_or_else(|| AppError::InvalidParam("未提供文件".to_string()))?;

    // 生成唯一存储路径
    let storage_key = generate_storage_path(&file_data.filename, &category);

    // 上传到存储后端
    let stored_key = state
        .storage
        .upload(&storage_key, &file_data.data, &file_data.content_type)
        .await
        .map_err(|e| {
            error!("文件上传失败: {}" , e);
            AppError::FileUploadFailed(e.to_string())
        })?;

    // 创建文件记录
    let file = SysFile {
        id: 0, // 数据库自增
        file_name: generate_file_name(&file_data.filename, None),
        original_name: metadata.name.unwrap_or_else(|| file_data.filename.clone()),
        file_size: metadata.size,
        mime_type: metadata.mime_type,
        storage_path: stored_key.clone(),
        storage_type: "local".to_string(),
        bucket: None,
        url: Some(state.storage.get_url(&stored_key)),
        md5: None,
        created_by: None,
        tenant_id: None,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
        deleted_at: None,
    };

    // 设置描述（如果有）
    // SysFile 没有 description 字段，可以通过扩展元数据存储
    let _ = metadata.description;

    let id = state.repository.insert(&file).await?;

    info!("文件上传成功: id={}, name={}" , id, file.original_name);

    Ok((
        StatusCode::CREATED,
        json_success_msg(
            serde_json::json!({
                "id": id,
                "name": file.original_name,
                "url": state.storage.get_url(&stored_key),
                "size": file.file_size,
                "mime_type": file.mime_type
            }),
            "文件上传成功" ,
        ),
    ))
}

/// 辅助数据结构
struct FileUploadData {
    filename: String,
    data: Vec<u8>,
    content_type: String,
}

#[derive(Default)]
struct FileMetadata {
    name: Option<String>,
    description: Option<String>,
    #[allow(dead_code)]
    category: Option<String>, // 预留字段，供前端扩展使用
    size: i64,
    mime_type: Option<String>,
}


/// 生成存储路径
fn generate_storage_path(filename: &str, category: &str) -> String {
    let now = chrono::Utc::now();
    let date = now.format("%Y/%m/%d" ).to_string();
    let uuid = uuid::Uuid::new_v4().to_string().replace('-', "" );
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("" );

    format!(
        "{}/{}/{}{}" ,
        category,
        date,
        uuid,
        if ext.is_empty() {
            String::new()
        } else {
            format!(".{ext}" )
        }
    )
}

/// 从 multipart 中提取文本字段
async fn read_text_field(
    field: impl futures_util::StreamExt<
        Item = Result<bytes::Bytes, axum::extract::multipart::MultipartError>,
    >,
) -> Result<String, AppError> {
    futures_util::pin_mut!(field);
    let mut text = String::new();
    while let Some(chunk) = futures_util::StreamExt::next(&mut field).await {
        match chunk {
            Ok(bytes) => text.push_str(&String::from_utf8_lossy(&bytes)),
            Err(e) => return Err(AppError::InvalidParam(format!("读取字段失败: {e}" ))),
        }
    }
    Ok(text)
}

/// 从 multipart 中提取请求头（需要通过 `FromRequestParts` 获取）
/// 这里简化处理，从环境变量获取
fn get_uploaded_by() -> String {
    std::env::var("FILE_UPLOADED_BY" ).unwrap_or_else(|_| "anonymous".to_string())
}

fn get_category() -> String {
    std::env::var("FILE_CATEGORY" ).unwrap_or_else(|_| "general".to_string())
}

/// 文件列表查询
async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<FileListQuery>,
) -> AppResult<Json<FileListResponse>> {
    let (files, total) = state
        .repository
        .find_list(
            query.page,
            query.page_size,
            query.category.as_deref(),
            query.keyword.as_deref(),
            query.start_date.as_deref(),
            query.end_date.as_deref(),
        )
        .await?;

    Ok(Json(FileListResponse {
        items: files,
        total,
        page: query.page,
        page_size: query.page_size,
    }))
}

/// 获取文件详情
async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<FileDetailResponse>> {
    let file = state
        .repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::FileNotFound)?;

    Ok(Json(FileDetailResponse {
        file,
        can_delete: true,
    }))
}

/// 文件下载
async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let file = state
        .repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::FileNotFound)?;

    let data = state.storage.download(&file.storage_path).await.map_err(|e| AppError::FileStorageError(e.to_string()))?;

    let filename = urlencoding::encode(&file.original_name);
    let content_disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}" ,
        file.original_name, filename
    );

    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                file.mime_type
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
            ),
            (header::CONTENT_DISPOSITION, content_disposition),
            (header::CONTENT_LENGTH, data.len().to_string()),
        ],
        data,
    )
        .into_response())
}

/// 删除文件
async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let file = state
        .repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::FileNotFound)?;

    // 从存储中删除文件
    state.storage.delete(&file.storage_path).await.map_err(|e| AppError::FileStorageError(e.to_string()))?;

    // 从数据库删除记录
    state.repository.delete(id).await?;

    Ok(json_ok_msg("文件删除成功" ))
}

/// 获取文件分类列表
async fn list_categories() -> Json<Vec<String>> {
    Json(vec![
        "general".to_string(),
        "image".to_string(),
        "document".to_string(),
        "video".to_string(),
        "audio".to_string(),
        "avatar".to_string(),
        "attachment".to_string(),
    ])
}

/// 健康检查
pub fn health_check() -> Json<serde_json::Value> {
    json_health("file-service" )
}

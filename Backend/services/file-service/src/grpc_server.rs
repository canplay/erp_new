use tonic::{Request, Response, Status};

use grpc_proto::file::file_service_server::FileService;
use grpc_proto::file::{
    CompleteUploadRequest, CompleteUploadResponse, CreateFolderRequest, CreateFolderResponse,
    DeleteFileRequest, DeleteFileResponse, DeleteFolderRequest, DeleteFolderResponse,
    FileInfo, GetDownloadUrlRequest, GetDownloadUrlResponse, GetFileRequest, GetFileResponse,
    ListFilesRequest, ListFilesResponse, ListFoldersRequest, ListFoldersResponse,
    UploadChunkRequest, UploadChunkResponse, UploadRequest, UploadResponse, FileType,
};

use crate::grpc_handlers::FileGrpcService;

fn file_info_to_proto(f: crate::grpc_handlers::FileInfo) -> FileInfo {
    FileInfo {
        id: f.id.to_string(),
        filename: f.file_name,
        original_filename: f.original_name,
        url: f.url.unwrap_or_default(),
        file_type: FileType::Other as i32,
        mime_type: f.mime_type.unwrap_or_default(),
        size: f.file_size,
        folder_id: String::new(),
        user_id: f.created_by.unwrap_or(0),
        etag: f.md5.unwrap_or_default(),
        created_at: f.created_at.map_or(0, |t| t.timestamp()),
        updated_at: 0,
    }
}

#[tonic::async_trait]
impl FileService for FileGrpcService {
    async fn upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadResponse>, Status> {
        let req = request.into_inner();
        // Generate a simple file_id for tracking uploads
        let file_id = format!("upload_{}", chrono::Utc::now().timestamp_millis());
        let upload_url = format!("/api/files/upload/{}/{}", file_id, req.filename);
        Ok(Response::new(UploadResponse {
            file_id,
            upload_url,
            upload_method: "PUT".to_string(),
        }))
    }

    async fn upload_chunk(
        &self,
        request: Request<UploadChunkRequest>,
    ) -> Result<Response<UploadChunkResponse>, Status> {
        let req = request.into_inner();
        // Simulate chunk storage by writing chunk data to temp location
        let chunk_dir = format!("/tmp/chunks/{}", req.file_id);
        std::fs::create_dir_all(&chunk_dir).map_err(|e| Status::internal(format!("IO error: {e}")))?;
        let chunk_path = format!("{}/chunk_{}", chunk_dir, req.chunk_index);
        std::fs::write(&chunk_path, &req.data).map_err(|e| Status::internal(format!("IO error: {e}")))?;
        // Count uploaded chunks
        let uploaded_chunks = std::fs::read_dir(&chunk_dir)
            .map(|d| d.filter_map(Result::ok).count() as i32)
            .unwrap_or(0);
        Ok(Response::new(UploadChunkResponse {
            success: true,
            uploaded_chunks,
            total_chunks: 0,
        }))
    }

    async fn complete_upload(
        &self,
        request: Request<CompleteUploadRequest>,
    ) -> Result<Response<CompleteUploadResponse>, Status> {
        let req = request.into_inner();
        let chunk_dir = format!("/tmp/chunks/{}", req.file_id);
        // Find all chunk files and merge them
        let mut chunk_files: Vec<_> = std::fs::read_dir(&chunk_dir)
            .map(|d| d.filter_map(Result::ok).collect())
            .unwrap_or_default();
        chunk_files.sort_by_key(|e| e.file_name().to_string_lossy().to_string());
        let mut merged = Vec::new();
        for chunk_entry in &chunk_files {
            let data = std::fs::read(chunk_entry.path()).unwrap_or_default();
            merged.extend_from_slice(&data);
        }
        // Compute hash (using djb2 hash as md5 crate is not available)
        let hash: u64 = merged.iter().fold(5381u64, |h, &b| -> u64 {
            h.wrapping_mul(33).wrapping_add(b as u64)
        });
        let hash_str = format!("{:x}", hash);
        // Store merged file
        let storage_path = format!("uploads/{}.bin", req.file_id);
        std::fs::write(&storage_path, &merged).ok();
        // Cleanup chunks
        std::fs::remove_dir_all(&chunk_dir).ok();
        // Create file record
        let file_size = merged.len() as i64;
        let sys_file = crate::models::SysFile {
            id: 0,
            file_name: format!("{}.bin", req.file_id),
            original_name: "uploaded_file".to_string(),
            file_size,
            mime_type: Some("application/octet-stream".to_string()),
            storage_path: storage_path.clone(),
            storage_type: "local".to_string(),
            bucket: None,
            url: Some(format!("/files/{}", storage_path)),
            md5: Some(hash_str.clone()),
            created_by: None,
            tenant_id: None,
            created_at: Some(chrono::Utc::now()),
            updated_at: None,
            deleted_at: None,
        };
        let id = self.state.repository.insert(&sys_file).await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;
        Ok(Response::new(CompleteUploadResponse {
            file: Some(FileInfo {
                id: id.to_string(),
                filename: format!("{}.bin", req.file_id),
                original_filename: "uploaded_file".to_string(),
                url: format!("/files/{}", storage_path),
                file_type: FileType::Other as i32,
                mime_type: "application/octet-stream".to_string(),
                size: file_size,
                folder_id: String::new(),
                user_id: 0,
                etag: hash_str,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: 0,
            }),
        }))
    }

    async fn list_files(
        &self,
        request: Request<ListFilesRequest>,
    ) -> Result<Response<ListFilesResponse>, Status> {
        let req = request.into_inner();
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 20 };
        let keyword = if req.keyword.is_empty() {
            None
        } else {
            Some(req.keyword)
        };

        let (files, total) = crate::grpc_handlers::list_files(
            self.state().clone(),
            page,
            page_size,
            None,
            keyword,
        )
        .await?;

        let files: Vec<FileInfo> = files.into_iter().map(file_info_to_proto).collect();

        Ok(Response::new(ListFilesResponse {
            files,
            total,
            total_size: 0,
        }))
    }

    async fn get_file(
        &self,
        request: Request<GetFileRequest>,
    ) -> Result<Response<GetFileResponse>, Status> {
        let req = request.into_inner();
        let id: i64 = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid file id"))?;

        let result = crate::grpc_handlers::get_file(self.state().clone(), id).await?;

        Ok(Response::new(GetFileResponse {
            file: result.map(file_info_to_proto),
        }))
    }

    async fn delete_file(
        &self,
        request: Request<DeleteFileRequest>,
    ) -> Result<Response<DeleteFileResponse>, Status> {
        let req = request.into_inner();
        let mut count = 0;

        // Delete single
        if !req.id.is_empty() {
            let id: i64 = req
                .id
                .parse()
                .map_err(|_| Status::invalid_argument("invalid file id"))?;
            if crate::grpc_handlers::delete_file(self.state().clone(), id).await? {
                count += 1;
            }
        }

        // Batch delete
        if !req.ids.is_empty() {
            let ids: Vec<i64> = req
                .ids
                .iter()
                .filter_map(|s| s.parse::<i64>().ok())
                .collect();
            if !ids.is_empty() {
                count += crate::grpc_handlers::batch_delete_files(self.state().clone(), ids).await?
                    as i32;
            }
        }

        Ok(Response::new(DeleteFileResponse { count }))
    }

    async fn get_download_url(
        &self,
        _request: Request<GetDownloadUrlRequest>,
    ) -> Result<Response<GetDownloadUrlResponse>, Status> {
        Err(Status::unimplemented(
            "get_download_url not implemented via gRPC",
        ))
    }

    async fn list_folders(
        &self,
        _request: Request<ListFoldersRequest>,
    ) -> Result<Response<ListFoldersResponse>, Status> {
        Ok(Response::new(ListFoldersResponse {
            folders: vec![],
        }))
    }

    async fn create_folder(
        &self,
        _request: Request<CreateFolderRequest>,
    ) -> Result<Response<CreateFolderResponse>, Status> {
        Err(Status::unimplemented("create_folder not implemented via gRPC"))
    }

    async fn delete_folder(
        &self,
        _request: Request<DeleteFolderRequest>,
    ) -> Result<Response<DeleteFolderResponse>, Status> {
        Err(Status::unimplemented("delete_folder not implemented via gRPC"))
    }
}

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
        _request: Request<UploadRequest>,
    ) -> Result<Response<UploadResponse>, Status> {
        Err(Status::unimplemented("upload not implemented via gRPC"))
    }

    async fn upload_chunk(
        &self,
        _request: Request<UploadChunkRequest>,
    ) -> Result<Response<UploadChunkResponse>, Status> {
        Err(Status::unimplemented("upload_chunk not implemented via gRPC"))
    }

    async fn complete_upload(
        &self,
        _request: Request<CompleteUploadRequest>,
    ) -> Result<Response<CompleteUploadResponse>, Status> {
        Err(Status::unimplemented("complete_upload not implemented via gRPC"))
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

// ============== 文件服务 gRPC 调用封装 ==============

impl FileGrpcClient {
    pub async fn upload(
        &mut self,
        filename: String,
        size: i64,
        mime_type: String,
        file_type: i32,
        folder_id: String,
        user_id: i64,
    ) -> Result<grpc_proto::file::UploadResponse, tonic::Status> {
        let request = grpc_proto::file::UploadRequest {
            filename, size, mime_type,
            file_type: grpc_proto::file::FileType::try_from(file_type)
                .unwrap_or(grpc_proto::file::FileType::Other).into(),
            folder_id, user_id,
        };
        Ok(self.inner.upload(request).await?.into_inner())
    }

    pub async fn list_files(
        &mut self,
        folder_id: String,
        user_id: i64,
        file_type: i32,
        keyword: String,
        page: i32,
        page_size: i32,
    ) -> Result<grpc_proto::file::ListFilesResponse, tonic::Status> {
        let request = grpc_proto::file::ListFilesRequest {
            folder_id, user_id,
            file_type: grpc_proto::file::FileType::try_from(file_type)
                .unwrap_or(grpc_proto::file::FileType::Other).into(),
            keyword, page, page_size,
        };
        Ok(self.inner.list_files(request).await?.into_inner())
    }

    pub async fn get_file(
        &mut self,
        id: String,
        user_id: i64,
    ) -> Result<grpc_proto::file::GetFileResponse, tonic::Status> {
        let request = grpc_proto::file::GetFileRequest { id, user_id };
        Ok(self.inner.get_file(request).await?.into_inner())
    }

    pub async fn delete_file(
        &mut self,
        id: String,
        ids: Vec<String>,
    ) -> Result<grpc_proto::file::DeleteFileResponse, tonic::Status> {
        let request = grpc_proto::file::DeleteFileRequest { id, ids };
        Ok(self.inner.delete_file(request).await?.into_inner())
    }

    pub async fn get_download_url(
        &mut self,
        file_id: String,
        user_id: i64,
        expires_in: i32,
    ) -> Result<grpc_proto::file::GetDownloadUrlResponse, tonic::Status> {
        let request = grpc_proto::file::GetDownloadUrlRequest { file_id, user_id, expires_in };
        Ok(self.inner.get_download_url(request).await?.into_inner())
    }

    pub async fn list_folders(
        &mut self,
        parent_id: String,
        user_id: i64,
    ) -> Result<grpc_proto::file::ListFoldersResponse, tonic::Status> {
        let request = grpc_proto::file::ListFoldersRequest { parent_id, user_id };
        Ok(self.inner.list_folders(request).await?.into_inner())
    }

    pub async fn create_folder(
        &mut self,
        name: String,
        parent_id: String,
        user_id: i64,
    ) -> Result<grpc_proto::file::CreateFolderResponse, tonic::Status> {
        let request = grpc_proto::file::CreateFolderRequest { name, parent_id, user_id };
        Ok(self.inner.create_folder(request).await?.into_inner())
    }

    pub async fn delete_folder(
        &mut self,
        id: String,
        recursive: bool,
    ) -> Result<grpc_proto::file::DeleteFolderResponse, tonic::Status> {
        let request = grpc_proto::file::DeleteFolderRequest { id, recursive };
        Ok(self.inner.delete_folder(request).await?.into_inner())
    }
}


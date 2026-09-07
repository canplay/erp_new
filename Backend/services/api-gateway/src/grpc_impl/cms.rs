// ============== CMS 服务 gRPC 调用封装 ==============

impl CmsGrpcClient {
    pub async fn list_articles(
        &mut self,
        page: i32,
        page_size: i32,
        category_id: i64,
        keyword: String,
        status: String,
    ) -> Result<grpc_proto::cms::ListArticlesResponse, tonic::Status> {
        let request = grpc_proto::cms::ListArticlesRequest {
            page, page_size, category_id, keyword, status,
        };
        Ok(self.inner.list_articles(request).await?.into_inner())
    }

    pub async fn get_article(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::cms::GetArticleResponse, tonic::Status> {
        let request = grpc_proto::cms::GetArticleRequest { id };
        Ok(self.inner.get_article(request).await?.into_inner())
    }

    pub async fn create_article(
        &mut self,
        params: CreateArticleParams,
    ) -> Result<grpc_proto::cms::CreateArticleResponse, tonic::Status> {
        let request = grpc_proto::cms::CreateArticleRequest {
            title: params.title,
            content: params.content,
            summary: params.summary,
            cover_image: params.cover_image,
            category_id: params.category_id,
            tags: params.tags,
            author: params.author,
            status: params.status,
        };
        Ok(self.inner.create_article(request).await?.into_inner())
    }

    pub async fn update_article(
        &mut self,
        params: UpdateArticleParams,
    ) -> Result<grpc_proto::cms::UpdateArticleResponse, tonic::Status> {
        let request = grpc_proto::cms::UpdateArticleRequest {
            id: params.id,
            title: params.title,
            content: params.content,
            summary: params.summary,
            cover_image: params.cover_image,
            category_id: params.category_id,
            tags: params.tags,
            status: params.status,
        };
        Ok(self.inner.update_article(request).await?.into_inner())
    }

    pub async fn delete_article(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::cms::DeleteArticleResponse, tonic::Status> {
        let request = grpc_proto::cms::DeleteArticleRequest { id };
        Ok(self.inner.delete_article(request).await?.into_inner())
    }

    pub async fn list_categories(
        &mut self,
        parent_id: i64,
        include_children: bool,
    ) -> Result<grpc_proto::cms::ListCategoriesResponse, tonic::Status> {
        let request = grpc_proto::cms::ListCategoriesRequest { parent_id, include_children };
        Ok(self.inner.list_categories(request).await?.into_inner())
    }

    pub async fn get_category(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::cms::GetCategoryResponse, tonic::Status> {
        let request = grpc_proto::cms::GetCategoryRequest { id };
        Ok(self.inner.get_category(request).await?.into_inner())
    }

    pub async fn create_category(
        &mut self,
        name: String,
        slug: String,
        description: String,
        parent_id: i64,
        sort_order: i32,
        icon: String,
    ) -> Result<grpc_proto::cms::CreateCategoryResponse, tonic::Status> {
        let request = grpc_proto::cms::CreateCategoryRequest {
            name, slug, description, parent_id, sort_order, icon,
        };
        Ok(self.inner.create_category(request).await?.into_inner())
    }

    pub async fn update_category(
        &mut self,
        id: i64,
        name: String,
        slug: String,
        description: String,
        parent_id: i64,
        sort_order: i32,
        icon: String,
    ) -> Result<grpc_proto::cms::UpdateCategoryResponse, tonic::Status> {
        let request = grpc_proto::cms::UpdateCategoryRequest {
            id, name, slug, description, parent_id, sort_order, icon,
        };
        Ok(self.inner.update_category(request).await?.into_inner())
    }

    pub async fn delete_category(
        &mut self,
        id: i64,
        force: bool,
    ) -> Result<grpc_proto::cms::DeleteCategoryResponse, tonic::Status> {
        let request = grpc_proto::cms::DeleteCategoryRequest { id, force };
        Ok(self.inner.delete_category(request).await?.into_inner())
    }

    pub async fn list_tags(
        &mut self,
        page: i32,
        page_size: i32,
    ) -> Result<grpc_proto::cms::ListTagsResponse, tonic::Status> {
        let request = grpc_proto::cms::ListTagsRequest { page, page_size };
        Ok(self.inner.list_tags(request).await?.into_inner())
    }

    pub async fn create_tag(
        &mut self,
        name: String,
        slug: String,
    ) -> Result<grpc_proto::cms::CreateTagResponse, tonic::Status> {
        let request = grpc_proto::cms::CreateTagRequest { name, slug };
        Ok(self.inner.create_tag(request).await?.into_inner())
    }

    pub async fn delete_tag(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::cms::DeleteTagResponse, tonic::Status> {
        let request = grpc_proto::cms::DeleteTagRequest { id };
        Ok(self.inner.delete_tag(request).await?.into_inner())
    }
}


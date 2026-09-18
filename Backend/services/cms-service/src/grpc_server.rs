//! CMS Service gRPC Server
//!
//! 实现 cms.proto 中定义的 gRPC 服务 trait

use std::net::SocketAddr;
use grpc_proto::cms::cms_service_server::CmsServiceServer;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use grpc_proto::cms::{
    Article, Category, CreateArticleRequest, CreateArticleResponse, CreateCategoryRequest,
    CreateCategoryResponse, CreateTagRequest, CreateTagResponse, DeleteArticleRequest,
    DeleteArticleResponse, DeleteCategoryRequest, DeleteCategoryResponse, DeleteTagRequest,
    DeleteTagResponse, GetArticleRequest, GetArticleResponse, GetCategoryRequest,
    GetCategoryResponse, ListArticlesRequest, ListArticlesResponse, ListCategoriesRequest,
    ListCategoriesResponse, ListTagsRequest, ListTagsResponse, UpdateArticleRequest,
    UpdateArticleResponse, UpdateCategoryRequest, UpdateCategoryResponse,
    cms_service_server::CmsService,
};

use crate::CmsAppState;

const DEFAULT_PAGE_SIZE: i32 = 20;
const MAX_PAGE_SIZE: i32 = 100;

/// `CmsService` 实现
#[derive(Clone)]
pub struct CmsGrpcServer {
    state: Arc<CmsAppState>,
}

impl CmsGrpcServer {
    /// 创建新的 `CmsService` 实例
    #[must_use]
    pub const fn new(state: Arc<CmsAppState>) -> Self {
        Self { state }
    }
}

/// 将 `chrono::DateTime` 转换为 Unix 时间戳（i64）
const fn datetime_to_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> i64 {
    dt.timestamp()
}

/// 将 Option<DateTime> 转换为 Unix 时间戳
fn optional_datetime_to_timestamp(dt: &Option<chrono::DateTime<chrono::Utc>>) -> i64 {
    dt.map_or(0, |d| d.timestamp())
}

/// 将 `CmsArticle` 转换为 proto Article
fn article_to_proto(a: &crate::models::CmsArticle) -> Article {
    let tags: Vec<String> = a
        .tags
        .as_ref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();

    let status_str = match a.status {
        0 => "draft" ,
        1 => "pending" ,
        2 => "published" ,
        3 => "rejected" ,
        4 => "archived" ,
        _ => "draft" ,
    };

    Article {
        id: a.id,
        title: a.title.clone(),
        content: a.content.clone(),
        summary: a.summary.clone().unwrap_or_default(),
        cover_image: a.cover_image.clone().unwrap_or_default(),
        category_id: a.category_id,
        category_name: String::new(), // 需要从分类服务获取
        tags,
        author: a.author_id.to_string(),
        status: status_str.to_string(),
        view_count: a.view_count,
        created_at: datetime_to_timestamp(&a.created_at),
        updated_at: datetime_to_timestamp(&a.updated_at),
        published_at: optional_datetime_to_timestamp(&a.published_at),
    }
}

/// 将 `CmsCategory` 转换为 proto Category
fn category_to_proto(c: &crate::models::CmsCategory) -> Category {
    Category {
        id: c.id,
        name: c.name.clone(),
        slug: c.slug.clone(),
        description: c.description.clone().unwrap_or_default(),
        parent_id: c.parent_id.unwrap_or(0),
        sort_order: c.sort_order,
        icon: c.icon.clone().unwrap_or_default(),
        children: vec![], // 需要递归获取
        article_count: 0, // 需要查询
        created_at: datetime_to_timestamp(&c.created_at),
        updated_at: datetime_to_timestamp(&c.updated_at),
    }
}


#[tonic::async_trait]
impl CmsService for CmsGrpcServer {
    // ============== 文章相关 ==============

    async fn list_articles(
        &self,
        request: Request<ListArticlesRequest>,
    ) -> Result<Response<ListArticlesResponse>, Status> {
        let req = request.into_inner();
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 {
            req.page_size.min(MAX_PAGE_SIZE)
        } else {
            DEFAULT_PAGE_SIZE
        };
        let keyword = if req.keyword.is_empty() {
            None
        } else {
            Some(req.keyword.as_str())
        };

        // 解析分类 ID 和状态
        let category_id = if req.category_id > 0 {
            Some(req.category_id)
        } else {
            None
        };
        let status = match req.status.as_str() {
            "draft" => Some(0),
            "published" => Some(2),
            "archived" => Some(4),
            _ => None,
        };

        match self
            .state
            .repository
            .article
            .list(page, page_size, category_id, status, keyword)
            .await
        {
            Ok(result) => {
                let articles: Vec<Article> = result.articles.iter().map(article_to_proto).collect();
                Ok(Response::new(ListArticlesResponse {
                    articles,
                    total: result.total,
                }))
            }
            Err(e) => {
                tracing::error!("列出文章失败: {e}" );
                Err(Status::internal("列出文章失败" ))
            }
        }
    }

    async fn get_article(
        &self,
        request: Request<GetArticleRequest>,
    ) -> Result<Response<GetArticleResponse>, Status> {
        let req = request.into_inner();

        match self.state.repository.article.find_by_id(req.id).await {
            Ok(Some(article)) => Ok(Response::new(GetArticleResponse {
                article: Some(article_to_proto(&article)),
            })),
            Ok(None) => Err(Status::not_found("文章不存在" )),
            Err(e) => {
                tracing::error!("获取文章失败: {e}" );
                Err(Status::internal("获取文章失败" ))
            }
        }
    }

    async fn create_article(
        &self,
        request: Request<CreateArticleRequest>,
    ) -> Result<Response<CreateArticleResponse>, Status> {
        let req = request.into_inner();

        let is_draft = req.status == "draft" || req.status.is_empty();
        let author_id: i64 = req.author.parse().unwrap_or(1);

        match self
            .state
            .repository
            .article
            .create(
                req.category_id,
                &req.title,
                &req.content,
                author_id,
                is_draft,
            )
            .await
        {
            Ok(id) => Ok(Response::new(CreateArticleResponse {
                id,
                title: req.title,
            })),
            Err(e) => {
                tracing::error!("创建文章失败: {e}" );
                Err(Status::internal("创建文章失败" ))
            }
        }
    }

    async fn update_article(
        &self,
        request: Request<UpdateArticleRequest>,
    ) -> Result<Response<UpdateArticleResponse>, Status> {
        let req = request.into_inner();

        let title = if req.title.is_empty() {
            None
        } else {
            Some(req.title.as_str())
        };
        let content = if req.content.is_empty() {
            None
        } else {
            Some(req.content.as_str())
        };

        match self
            .state
            .repository
            .article
            .update(req.id, title, content, None, None)
            .await
        {
            Ok(true) => Ok(Response::new(UpdateArticleResponse {
                id: req.id,
                title: req.title,
            })),
            Ok(false) => Err(Status::not_found("文章不存在" )),
            Err(e) => {
                tracing::error!("更新文章失败: {e}" );
                Err(Status::internal("更新文章失败" ))
            }
        }
    }

    async fn delete_article(
        &self,
        request: Request<DeleteArticleRequest>,
    ) -> Result<Response<DeleteArticleResponse>, Status> {
        let req = request.into_inner();

        match self.state.repository.article.delete(req.id).await {
            Ok(true) => Ok(Response::new(DeleteArticleResponse { success: true })),
            Ok(false) => Err(Status::not_found("文章不存在" )),
            Err(e) => {
                tracing::error!("删除文章失败: {e}" );
                Err(Status::internal("删除文章失败" ))
            }
        }
    }

    // ============== 分类相关 ==============

    async fn list_categories(
        &self,
        request: Request<ListCategoriesRequest>,
    ) -> Result<Response<ListCategoriesResponse>, Status> {
        let req = request.into_inner();

        // parent_id: 0 表示顶级分类
        let parent_id = if req.parent_id > 0 {
            Some(req.parent_id)
        } else {
            None
        };

        if req.include_children {
            // 获取分类树
            let tree = self.state.repository.category.get_tree(parent_id);
            let categories = tree
                .await
                .map_err(|e| {
                    tracing::error!("获取分类树失败: {e}" );
                    Status::internal("获取分类树失败" )
                })?
                .into_iter()
                .map(|node| Category {
                    id: node.id,
                    name: node.name,
                    slug: node.slug,
                    description: String::new(),
                    parent_id: node.parent_id.unwrap_or(0),
                    sort_order: node.sort_order,
                    icon: node.icon.unwrap_or_default(),
                    children: vec![], // 递归转换
                    article_count: 0,
                    created_at: 0,
                    updated_at: 0,
                })
                .collect();

            Ok(Response::new(ListCategoriesResponse { categories }))
        } else {
            // 获取分类列表（分页）
            let page = 1;
            let page_size = 100;

            match self
                .state
                .repository
                .category
                .list(page, page_size, None)
                .await
            {
                Ok(result) => {
                    let categories: Vec<Category> =
                        result.categories.iter().map(category_to_proto).collect();
                    Ok(Response::new(ListCategoriesResponse { categories }))
                }
                Err(e) => {
                    tracing::error!("列出分类失败: {e}" );
                    Err(Status::internal("列出分类失败" ))
                }
            }
        }
    }

    async fn get_category(
        &self,
        request: Request<GetCategoryRequest>,
    ) -> Result<Response<GetCategoryResponse>, Status> {
        let req = request.into_inner();

        match self.state.repository.category.find_by_id(req.id).await {
            Ok(Some(category)) => Ok(Response::new(GetCategoryResponse {
                category: Some(category_to_proto(&category)),
            })),
            Ok(None) => Err(Status::not_found("分类不存在" )),
            Err(e) => {
                tracing::error!("获取分类失败: {e}" );
                Err(Status::internal("获取分类失败" ))
            }
        }
    }

    async fn create_category(
        &self,
        request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CreateCategoryResponse>, Status> {
        let req = request.into_inner();

        let parent_id = if req.parent_id > 0 {
            Some(req.parent_id)
        } else {
            None
        };
        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.as_str())
        };

        match self
            .state
            .repository
            .category
            .create(&req.name, &req.slug, parent_id, description)
            .await
        {
            Ok(id) => Ok(Response::new(CreateCategoryResponse { id, name: req.name })),
            Err(crate::repository::category_repository::CategoryRepositoryError::AlreadyExists) => {
                Err(Status::already_exists("分类slug已存在" ))
            }
            Err(e) => {
                tracing::error!("创建分类失败: {e}" );
                Err(Status::internal("创建分类失败" ))
            }
        }
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<UpdateCategoryResponse>, Status> {
        let req = request.into_inner();

        let name = if req.name.is_empty() {
            None
        } else {
            Some(req.name.as_str())
        };
        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.as_str())
        };

        match self
            .state
            .repository
            .category
            .update(req.id, name, description, Some(req.sort_order), None)
            .await
        {
            Ok(true) => Ok(Response::new(UpdateCategoryResponse {
                id: req.id,
                name: req.name,
            })),
            Ok(false) => Err(Status::not_found("分类不存在" )),
            Err(e) => {
                tracing::error!("更新分类失败: {e}" );
                Err(Status::internal("更新分类失败" ))
            }
        }
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<DeleteCategoryResponse>, Status> {
        let req = request.into_inner();

        if req.force {
            // 强制删除：递归删除子分类
            match self.force_delete_category(req.id).await {
                Ok(()) => Ok(Response::new(DeleteCategoryResponse { success: true })),
                Err(e) => Err(e),
            }
        } else {
            match self.state.repository.category.delete(req.id).await {
                Ok(true) => Ok(Response::new(DeleteCategoryResponse { success: true })),
                Ok(false) => Err(Status::not_found("分类不存在" )),
                Err(
                    crate::repository::category_repository::CategoryRepositoryError::HasChildren,
                ) => Err(Status::failed_precondition(
                    "分类下存在子分类或关联文章，请使用 force 参数" ,
                )),
                Err(e) => {
                    tracing::error!("删除分类失败: {e}" );
                    Err(Status::internal("删除分类失败" ))
                }
            }
        }
    }

    // ============== 标签相关 ==============

    async fn list_tags(
        &self,
        _request: Request<ListTagsRequest>,
    ) -> Result<Response<ListTagsResponse>, Status> {
        // 标签列表（简化实现，返回空列表或从数据库查询）
        // 注意：实际项目中应实现 tag repository
        let tags = vec![];

        Ok(Response::new(ListTagsResponse { tags, total: 0 }))
    }

    async fn create_tag(
        &self,
        request: Request<CreateTagRequest>,
    ) -> Result<Response<CreateTagResponse>, Status> {
        let req = request.into_inner();

        // 简化实现：返回成功但不创建（需要 tag repository）
        let id = chrono::Utc::now().timestamp_millis();

        Ok(Response::new(CreateTagResponse { id, name: req.name }))
    }

    async fn delete_tag(
        &self,
        _request: Request<DeleteTagRequest>,
    ) -> Result<Response<DeleteTagResponse>, Status> {
        // 简化实现：返回成功（需要 tag repository）
        Ok(Response::new(DeleteTagResponse { success: true }))
    }
}

impl CmsGrpcServer {
    /// 递归强制删除分类及其子分类
    async fn force_delete_category(&self, id: i64) -> Result<(), Status> {
        // 获取所有子分类
        let children = sqlx::query_as::<_, crate::models::CmsCategory>(
            r"
            SELECT id, parent_id, name, slug, description, icon, sort_order,
                   seo_title, seo_keywords, seo_description, status, allow_attachment,
                   created_at, updated_at
            FROM cms_category
            WHERE parent_id = $1
            " ,
        )
        .bind(id)
        .fetch_all(self.state.repository.category.pool())
        .await
        .map_err(|e| Status::internal(format!("查询子分类失败: {e}" )))?;

        for child in children {
            Box::pin(self.force_delete_category(child.id)).await?;
        }

        // 删除当前分类
        self.state
            .repository
            .category
            .delete(id)
            .await
            .map_err(|e| Status::internal(format!("删除分类失败: {e}" )))?;

        Ok(())
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for CmsGrpcServer {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}" ).into() })?;
        let server = CmsServiceServer::new(CmsGrpcServer::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}" , e);
            }
        });
        Ok(handle)
    }
}

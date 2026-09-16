use std::collections::HashMap;
use tonic::{Request, Response, Status};

use grpc_proto::feedback::feedback_service_server::FeedbackService;
use grpc_proto::feedback::{
    AddReplyRequest, AddReplyResponse, CreateFeedbackRequest, CreateFeedbackResponse,
    DeleteFeedbackRequest, DeleteFeedbackResponse, Feedback, FeedbackPriority, FeedbackStatus,
    FeedbackType, GetFeedbackRequest, GetFeedbackResponse, GetStatsRequest, GetStatsResponse,
    ListFeedbacksRequest, ListFeedbacksResponse, ListRepliesRequest, ListRepliesResponse,
    UpdateFeedbackRequest, UpdateFeedbackResponse, FeedbackStats,
};

use crate::grpc_handlers::FeedbackGrpcService;

fn domain_type_to_proto_type(t: &str) -> i32 {
    match t.to_lowercase().as_str() {
        "bug" => FeedbackType::Bug as i32,
        "feature" | "suggestion" => FeedbackType::Feature as i32,
        "question" => FeedbackType::Question as i32,
        "complaint" => FeedbackType::Complaint as i32,
        _ => FeedbackType::Other as i32,
    }
}

fn domain_status_to_proto_status(s: &str) -> i32 {
    match s.to_lowercase().as_str() {
        "pending" => FeedbackStatus::Open as i32,
        "processing" => FeedbackStatus::InProgress as i32,
        "resolved" => FeedbackStatus::Resolved as i32,
        "closed" | "rejected" => FeedbackStatus::Closed as i32,
        _ => FeedbackStatus::Open as i32,
    }
}

fn feedback_to_proto(f: crate::grpc_handlers::FeedbackInfo) -> Feedback {
    Feedback {
        id: f.id,
        user_id: f.user_id,
        username: f.user_name.unwrap_or_default(),
        r#type: domain_type_to_proto_type(&f.r#type),
        status: domain_status_to_proto_status(&f.status),
        priority: FeedbackPriority::Medium as i32,
        title: f.title,
        content: f.content,
        attachments: vec![],
        device_info: String::new(),
        app_version: String::new(),
        extra: HashMap::new(),
        assigned_to: f.handler_id.unwrap_or(0),
        resolved_at: f.handler_time.map_or(0, |t| t.timestamp()),
        created_at: f.created_at.timestamp(),
        updated_at: f.updated_at.timestamp(),
    }
}

#[tonic::async_trait]
impl FeedbackService for FeedbackGrpcService {
    async fn list_feedbacks(
        &self,
        request: Request<ListFeedbacksRequest>,
    ) -> Result<Response<ListFeedbacksResponse>, Status> {
        let req = request.into_inner();
        let page = if req.page > 0 { Some(req.page) } else { None };
        let page_size = if req.page_size > 0 { Some(req.page_size) } else { None };
        let keyword = if req.keyword.is_empty() { None } else { Some(req.keyword) };

        let result = crate::grpc_handlers::list_feedback(
            self.state().clone(),
            page,
            page_size,
            None,
            None,
            keyword,
        )
        .await?;

        let feedbacks: Vec<Feedback> = result.list.into_iter().map(feedback_to_proto).collect();

        Ok(Response::new(ListFeedbacksResponse {
            feedbacks,
            total: result.total,
        }))
    }

    async fn get_feedback(
        &self,
        request: Request<GetFeedbackRequest>,
    ) -> Result<Response<GetFeedbackResponse>, Status> {
        let req = request.into_inner();
        let result = crate::grpc_handlers::get_feedback(self.state().clone(), req.id).await?;

        Ok(Response::new(GetFeedbackResponse {
            feedback: result.map(feedback_to_proto),
        }))
    }

    async fn create_feedback(
        &self,
        request: Request<CreateFeedbackRequest>,
    ) -> Result<Response<CreateFeedbackResponse>, Status> {
        let req = request.into_inner();
        let type_str = match req.r#type {
            x if x == FeedbackType::Bug as i32 => "bug" ,
            x if x == FeedbackType::Feature as i32 => "suggestion" ,
            x if x == FeedbackType::Complaint as i32 => "complaint" ,
            _ => "other" ,
        };

        let title = req.title.clone();
        let id = crate::grpc_handlers::create_feedback(
            self.state().clone(),
            req.user_id,
            type_str.to_string(),
            req.title,
            req.content,
            None,
        )
        .await?;

        Ok(Response::new(CreateFeedbackResponse {
            id,
            title,
        }))
    }

    async fn update_feedback(
        &self,
        request: Request<UpdateFeedbackRequest>,
    ) -> Result<Response<UpdateFeedbackResponse>, Status> {
        let req = request.into_inner();
        let status_str = match req.status {
            x if x == FeedbackStatus::Open as i32 => "pending" ,
            x if x == FeedbackStatus::Resolved as i32 => "resolved" ,
            x if x == FeedbackStatus::Closed as i32 => "closed" ,
            _ => return Err(Status::invalid_argument("invalid status" )),
        };

        crate::grpc_handlers::handle_feedback(
            self.state().clone(),
            req.id,
            status_str.to_string(),
            req.reply_content,
        )
        .await?;

        Ok(Response::new(UpdateFeedbackResponse {
            id: req.id,
            status: req.status,
        }))
    }

    async fn delete_feedback(
        &self,
        request: Request<DeleteFeedbackRequest>,
    ) -> Result<Response<DeleteFeedbackResponse>, Status> {
        let req = request.into_inner();
        let success = crate::grpc_handlers::delete_feedback(self.state().clone(), req.id).await?;

        Ok(Response::new(DeleteFeedbackResponse { success }))
    }

    async fn add_reply(
        &self,
        request: Request<AddReplyRequest>,
    ) -> Result<Response<AddReplyResponse>, Status> {
        let req = request.into_inner();
        let success = crate::grpc_handlers::reply_feedback(
            self.state().clone(),
            req.feedback_id,
            req.content,
        )
        .await?;

        Ok(Response::new(AddReplyResponse {
            id: 0,
            success,
        }))
    }

    async fn list_replies(
        &self,
        _request: Request<ListRepliesRequest>,
    ) -> Result<Response<ListRepliesResponse>, Status> {
        Ok(Response::new(ListRepliesResponse {
            replies: vec![],
        }))
    }

    async fn get_stats(
        &self,
        _request: Request<GetStatsRequest>,
    ) -> Result<Response<GetStatsResponse>, Status> {
        let start_date = None;
        let end_date = None;

        let stats = crate::grpc_handlers::get_statistics(
            self.state().clone(),
            start_date,
            end_date,
        )
        .await?;

        Ok(Response::new(GetStatsResponse {
            stats: Some(FeedbackStats {
                total_count: stats.total,
                open_count: stats.pending,
                in_progress_count: stats.processing,
                resolved_count: stats.resolved,
                closed_count: stats.rejected,
                by_type: HashMap::new(),
                by_priority: HashMap::new(),
            }),
        }))
    }
}

    // ============ 公告管理 ============

    async fn list_announcements(
        &self,
        request: Request<ListAnnouncementsRequest>,
    ) -> Result<Response<ListAnnouncementsResponse>, Status> {
        let req = request.into_inner();
        let is_active = if req.is_active == 0 { None } else { Some(req.is_active == 1) };
        let result = self.state.announcement_repository.list(req.page.max(1), req.page_size.clamp(1, 100), is_active)
            .await.map_err(|e| Status::internal(e.to_string()))?;

        let announcements: Vec<AnnouncementInfo> = result.announcements.into_iter().map(|a| AnnouncementInfo {
            id: a.id, title: a.title, content: String::new(), r#type: a.announcement_type,
            priority: a.priority, is_pinned: a.is_pinned, is_active: a.is_active,
            created_by: a.created_by_name.unwrap_or_default(),
            start_time: a.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
            end_time: a.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
            created_at: a.created_at.timestamp(), updated_at: 0,
        }).collect();

        Ok(Response::new(ListAnnouncementsResponse { announcements, total: result.total }))
    }

    async fn get_announcement(
        &self,
        request: Request<GetAnnouncementRequest>,
    ) -> Result<Response<GetAnnouncementResponse>, Status> {
        let req = request.into_inner();
        let a = self.state.announcement_repository.find_by_id(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        match a {
            Some(a) => Ok(Response::new(GetAnnouncementResponse {
                announcement: Some(AnnouncementInfo {
                    id: a.id, title: a.title, content: a.content, r#type: a.announcement_type,
                    priority: a.priority, is_pinned: a.is_pinned, is_active: a.is_active,
                    created_by: a.created_by_name.unwrap_or_default(),
                    start_time: a.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                    end_time: a.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                    created_at: a.created_at.timestamp(), updated_at: a.updated_at.timestamp(),
                }),
            })),
            None => Err(Status::not_found("公告不存在" )),
        }
    }

    async fn create_announcement(
        &self,
        request: Request<CreateAnnouncementRequest>,
    ) -> Result<Response<CreateAnnouncementResponse>, Status> {
        let req = request.into_inner();
        let created_by: Option<i64> = req.created_by.parse().ok();
        let id = self.state.announcement_repository.create(
            CreateAnnouncementParams {
                title: &req.title,
                content: &req.content,
                announcement_type: &req.r#type,
                priority: req.priority,
                is_pinned: req.is_pinned,
                is_active: req.is_active,
                start_time: None,
                end_time: None,
                created_by,
            }
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateAnnouncementResponse {
            announcement: Some(AnnouncementInfo {
                id, title: req.title, content: req.content, r#type: req.r#type,
                priority: req.priority, is_pinned: req.is_pinned, is_active: req.is_active,
                created_by: req.created_by, start_time: String::new(), end_time: String::new(),
                created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp(),
            }),
        }))
    }

    async fn update_announcement(
        &self,
        request: Request<UpdateAnnouncementRequest>,
    ) -> Result<Response<UpdateAnnouncementResponse>, Status> {
        let req = request.into_inner();
        let exists = self.state.announcement_repository.find_by_id(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        if exists.is_none() { return Err(Status::not_found("公告不存在" )); }

        let title = if req.title.is_empty() { None } else { Some(req.title) };
        let content = if req.content.is_empty() { None } else { Some(req.content) };
        let ann_type = if req.r#type.is_empty() { None } else { Some(req.r#type) };
        let priority_val = if req.priority == 0 { None } else { Some(req.priority) };

        self.state.announcement_repository.update(
            UpdateAnnouncementParams {
                id: req.id,
                title,
                content,
                ann_type,
                priority_val,
                is_pinned: Some(req.is_pinned),
                is_active: Some(req.is_active),
                start_time: None,
                end_time: None,
            }
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateAnnouncementResponse { success: true }))
    }

    async fn delete_announcement(
        &self,
        request: Request<DeleteAnnouncementRequest>,
    ) -> Result<Response<DeleteAnnouncementResponse>, Status> {
        let req = request.into_inner();
        let success = self.state.announcement_repository.delete(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        if !success { return Err(Status::not_found("公告不存在" )); }
        Ok(Response::new(DeleteAnnouncementResponse { success: true }))
    }

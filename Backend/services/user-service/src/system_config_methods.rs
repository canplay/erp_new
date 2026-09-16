    // ============ 系统配置 ============

    async fn list_system_configs(
        &self,
        request: Request<ListSystemConfigsRequest>,
    ) -> Result<Response<ListSystemConfigsResponse>, Status> {
        let req = request.into_inner();
        let category = if req.category.is_empty() { None } else { Some(req.category.as_str()) };
        let configs = self.state.announcement_repository.list_system_configs(category).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let config_infos: Vec<SystemConfigInfo> = configs.into_iter().map(|c| SystemConfigInfo {
            id: c.id, category: c.category, key: c.config_key, value: c.config_value.unwrap_or_default(),
            r#type: c.value_type, label: c.label, description: c.description.unwrap_or_default(),
            sort: c.sort_order, status: c.status,
            created_at: c.created_at.timestamp(), updated_at: c.updated_at.timestamp(),
        }).collect();

        Ok(Response::new(ListSystemConfigsResponse { configs: config_infos }))
    }

    async fn update_system_config(
        &self,
        request: Request<UpdateSystemConfigRequest>,
    ) -> Result<Response<UpdateSystemConfigResponse>, Status> {
        let req = request.into_inner();
        let success = self.state.announcement_repository.update_config(&req.key, &req.value).await
            .map_err(|e| Status::internal(e.to_string()))?;
        if !success { return Err(Status::not_found("配置不存在" )); }
        Ok(Response::new(UpdateSystemConfigResponse { success: true }))
    }

    async fn batch_update_system_configs(
        &self,
        request: Request<BatchUpdateSystemConfigsRequest>,
    ) -> Result<Response<BatchUpdateSystemConfigsResponse>, Status> {
        let req = request.into_inner();
        let pairs: Vec<(String, String)> = req.configs.into_iter().map(|c| (c.key, c.value)).collect();
        self.state.announcement_repository.batch_update_system_configs(&pairs).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchUpdateSystemConfigsResponse { success: true }))
    }

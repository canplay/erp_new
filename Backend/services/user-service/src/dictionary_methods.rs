    // ============ 数据字典类型 ============

    async fn list_dictionary_types(
        &self,
        request: Request<ListDictionaryTypesRequest>,
    ) -> Result<Response<ListDictionaryTypesResponse>, Status> {
        let req = request.into_inner();
        let page = req.page.max(1);
        let page_size = req.page_size.clamp(1, 100);
        let keyword = if req.keyword.is_empty() {
            None
        } else {
            Some(req.keyword.as_str())
        };
        let status = if req.status == 0 { None } else { Some(req.status) };

        let result = self
            .state
            .announcement_repository
            .list_dictionary_types(page, page_size, keyword, status)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let types: Vec<DictionaryTypeInfo> = result
            .types
            .into_iter()
            .map(|t| DictionaryTypeInfo {
                id: t.id,
                code: t.code,
                name: t.name,
                description: t.description.unwrap_or_default(),
                sort: t.sort,
                status: t.status,
                created_at: t.created_at.timestamp(),
                updated_at: t.updated_at.timestamp(),
            })
            .collect();

        Ok(Response::new(ListDictionaryTypesResponse {
            types,
            total: result.total,
        }))
    }

    async fn create_dictionary_type(
        &self,
        request: Request<CreateDictionaryTypeRequest>,
    ) -> Result<Response<CreateDictionaryTypeResponse>, Status> {
        let req = request.into_inner();

        if req.code.is_empty() || req.name.is_empty() {
            return Err(Status::invalid_argument("字典类型编码和名称不能为空" ));
        }

        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.as_str())
        };
        // sort 列在 struct 中是非 Option 类型，插入 NULL 会导致 SELECT panic
        let sort = Some(req.sort);

        let type_id = self
            .state
            .announcement_repository
            .create_dictionary_type(&req.code, &req.name, description, sort)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateDictionaryTypeResponse {
            r#type: Some(DictionaryTypeInfo {
                id: type_id,
                code: req.code,
                name: req.name,
                description: req.description,
                sort: sort.unwrap_or(0),
                status: 1,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
            }),
        }))
    }

    async fn update_dictionary_type(
        &self,
        request: Request<UpdateDictionaryTypeRequest>,
    ) -> Result<Response<UpdateDictionaryTypeResponse>, Status> {
        let req = request.into_inner();

        let existing = self
            .state
            .announcement_repository
            .find_dictionary_type_by_id(req.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if existing.is_none() {
            return Err(Status::not_found("字典类型不存在" ));
        }

        self.state
            .announcement_repository
            .update_dictionary_type(
                req.id,
                Some(req.name).filter(|s| !s.is_empty()),
                Some(req.description).filter(|s| !s.is_empty()),
                Some(req.sort).filter(|&s| s != 0),
                Some(req.status),
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateDictionaryTypeResponse { success: true }))
    }

    async fn delete_dictionary_type(
        &self,
        request: Request<DeleteDictionaryTypeRequest>,
    ) -> Result<Response<DeleteDictionaryTypeResponse>, Status> {
        let req = request.into_inner();

        let success = self
            .state
            .announcement_repository
            .delete_dictionary_type(req.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !success {
            return Err(Status::not_found("字典类型不存在" ));
        }

        Ok(Response::new(DeleteDictionaryTypeResponse { success: true }))
    }

    // ============ 数据字典项 ============

    async fn list_dictionary_items(
        &self,
        request: Request<ListDictionaryItemsRequest>,
    ) -> Result<Response<ListDictionaryItemsResponse>, Status> {
        let req = request.into_inner();
        let type_id = if req.type_id == 0 { None } else { Some(req.type_id) };
        let type_code = if req.type_code.is_empty() { None } else { Some(req.type_code.as_str()) };
        let keyword = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
        let status = if req.status == 0 { None } else { Some(req.status) };

        let result = self.state.announcement_repository
            .list_dictionary_items(type_id, type_code, keyword, status)
            .await.map_err(|e| Status::internal(e.to_string()))?;

        let items: Vec<DictionaryItemInfo> = result.items.into_iter().map(|item| DictionaryItemInfo {
            id: item.id, type_id: item.type_id, label: item.label, value: item.value,
            sort: item.sort, status: item.status, is_default: item.is_default,
            remark: item.remark.unwrap_or_default(),
            created_at: item.created_at.timestamp(), updated_at: item.updated_at.timestamp(),
        }).collect();

        Ok(Response::new(ListDictionaryItemsResponse { items, total: result.total }))
    }

    async fn create_dictionary_item(
        &self,
        request: Request<CreateDictionaryItemRequest>,
    ) -> Result<Response<CreateDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        // 不传 NULL：sort/status/is_default 列在 struct 中是非 Option 类型，
        // 插入 NULL 会导致后续 SELECT 时 ColumnDecode/UnexpectedNullError panic。
        let sort = Some(req.sort);
        let status = Some(req.status);
        let is_default = Some(req.is_default);
        let remark = if req.remark.is_empty() { None } else { Some(req.remark.as_str()) };

        let params = crate::repository::CreateDictionaryItemParams {
            type_id: req.type_id,
            label: &req.label,
            value: &req.value,
            sort,
            status,
            is_default,
            remark,
        };

        let item_id = self.state.announcement_repository
            .create_dictionary_item(params)
            .await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateDictionaryItemResponse {
            item: Some(DictionaryItemInfo {
                id: item_id, type_id: req.type_id, label: req.label, value: req.value,
                sort: sort.unwrap_or(0), status: status.unwrap_or(1), is_default: is_default.unwrap_or(false),
                remark: remark.unwrap_or("" ).to_string(),
                created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp(),
            }),
        }))
    }

    async fn update_dictionary_item(
        &self,
        request: Request<UpdateDictionaryItemRequest>,
    ) -> Result<Response<UpdateDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        let exists = self.state.announcement_repository.find_dictionary_item_by_id(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        if exists.is_none() { return Err(Status::not_found("字典项不存在" )); }

        let label = if req.label.is_empty() { None } else { Some(req.label) };
        let value = if req.value.is_empty() { None } else { Some(req.value) };
        let remark = if req.remark.is_empty() { None } else { Some(req.remark) };
        let sort_val = if req.sort == 0 { None } else { Some(req.sort) };
        let status_val = if req.status == 0 { None } else { Some(req.status) };
        let is_default_val = if req.is_default { Some(true) } else { None };

        let update_params = crate::repository::UpdateDictionaryItemParams {
            id: req.id,
            label: label.as_deref().unwrap_or("" ),
            value: value.as_deref().unwrap_or("" ),
            sort: sort_val,
            status: status_val,
            is_default: is_default_val,
            remark: remark.as_deref(),
        };

        self.state.announcement_repository.update_dictionary_item(update_params)
            .await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateDictionaryItemResponse { success: true }))
    }

    async fn delete_dictionary_item(
        &self,
        request: Request<DeleteDictionaryItemRequest>,
    ) -> Result<Response<DeleteDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        let success = self.state.announcement_repository.delete_dictionary_item(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        if !success { return Err(Status::not_found("字典项不存在" )); }
        Ok(Response::new(DeleteDictionaryItemResponse { success: true }))
    }

    async fn get_dictionary_type(
        &self,
        request: Request<GetDictionaryTypeRequest>,
    ) -> Result<Response<GetDictionaryTypeResponse>, Status> {
        let req = request.into_inner();
        // the repository has find_dictionary_type_by_id directly, use it
        let found = self.state.announcement_repository.find_dictionary_type_by_id(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        match found {
            Some(t) => Ok(Response::new(GetDictionaryTypeResponse {
                r#type: Some(DictionaryTypeInfo {
                    id: t.id, code: t.code, name: t.name,
                    description: t.description.unwrap_or_default(),
                    sort: t.sort, status: t.status,
                    created_at: t.created_at.timestamp(), updated_at: t.updated_at.timestamp(),
                }),
            })),
            None => Err(Status::not_found("字典类型不存在" )),
        }
    }

    async fn get_dictionary_item(
        &self,
        request: Request<GetDictionaryItemRequest>,
    ) -> Result<Response<GetDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        let found = self.state.announcement_repository.find_dictionary_item_by_id(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        match found {
            Some(item) => Ok(Response::new(GetDictionaryItemResponse {
                item: Some(DictionaryItemInfo {
                    id: item.id, type_id: item.type_id,
                    label: item.label, value: item.value,
                    sort: item.sort, status: item.status,
                    is_default: item.is_default,
                    remark: item.remark.unwrap_or_default(),
                    created_at: item.created_at.timestamp(),
                    updated_at: item.updated_at.timestamp(),
                }),
            })),
            None => Err(Status::not_found("字典项不存在" )),
        }
    }

    async fn import_users(
        &self,
        request: Request<ImportUsersRequest>,
    ) -> Result<Response<ImportUsersResponse>, Status> {
        let req = request.into_inner();
        // simple CSV parse, call user_repository.batch_create
        let result = self.state.user_repository.batch_create_from_csv(
            &req.data_base64, &req.format, &req.update_mode,
        ).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ImportUsersResponse {
            total: result.total as i32,
            success_count: result.success_count as i32,
            fail_count: result.fail_count as i32,
            errors: result.errors,
        }))
    }

    async fn export_users(
        &self,
        request: Request<ExportUsersRequest>,
    ) -> Result<Response<ExportUsersResponse>, Status> {
        let req = request.into_inner();
        let data = self.state.user_repository.export_to_csv(
            &req.keyword, req.status, &req.role, &req.format,
        ).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ExportUsersResponse {
            data: data.0,
            filename: data.1,
            format: req.format,
        }))
    }

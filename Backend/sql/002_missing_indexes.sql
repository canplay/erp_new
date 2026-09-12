-- =============================================================================
-- 002_missing_indexes.sql
-- Database Index Audit and Optimization
-- =============================================================================
-- Purpose: Add missing indexes for high-frequency query patterns
-- All indexes use IF NOT EXISTS for idempotency
-- =============================================================================

-- =============================================================================
-- 1. USERS TABLE - Upgrade email index to UNIQUE
-- =============================================================================
-- The existing idx_users_email is non-unique. Email should be unique to prevent
-- duplicate accounts and speed up login lookups.
-- Note: Drop the old non-unique index first, then create unique.

DROP INDEX IF EXISTS "idx_users_email";
CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_email_unique" ON "public"."users" USING btree ("email" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_users_email_unique" IS '用户邮箱唯一索引 - 防止重复注册，加速登录查询';

-- =============================================================================
-- 2. AUDIT_LOGS TABLE - Composite index for tenant + time range queries
-- =============================================================================
-- Audit logs are frequently queried by tenant with time-based filtering
-- (e.g., "show all audit logs for tenant X in the last 7 days").

CREATE INDEX IF NOT EXISTS "idx_audit_logs_tenant_created" ON "public"."audit_logs" USING btree ("tenant_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);
COMMENT ON INDEX "public"."idx_audit_logs_tenant_created" IS '审计日志租户+时间复合索引 - 支持按租户和时间范围查询审计记录';

-- =============================================================================
-- 3. SYS_LOGIN_LOGS TABLE - Composite index for user login history
-- =============================================================================
-- Login history queries often filter by user_id with time ordering
-- (e.g., "show recent logins for user X").

CREATE INDEX IF NOT EXISTS "idx_sys_login_logs_user_created" ON "public"."sys_login_logs" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);
COMMENT ON INDEX "public"."idx_sys_login_logs_user_created" IS '登录日志用户+时间复合索引 - 支持查询用户登录历史';

-- =============================================================================
-- 4. FEEDBACKS TABLE - Indexes for common query patterns
-- =============================================================================
-- Feedbacks are queried by user (to see own feedbacks) and by status
-- (for admin dashboards showing pending feedbacks).

CREATE INDEX IF NOT EXISTS "idx_feedbacks_user_id" ON "public"."feedbacks" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_feedbacks_user_id" IS '反馈用户索引 - 支持查询用户提交的反馈';

CREATE INDEX IF NOT EXISTS "idx_feedbacks_status" ON "public"."feedbacks" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_feedbacks_status" IS '反馈状态索引 - 支持按状态筛选反馈（待处理/已处理等）';

CREATE INDEX IF NOT EXISTS "idx_feedbacks_status_created" ON "public"."feedbacks" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST, "created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);
COMMENT ON INDEX "public"."idx_feedbacks_status_created" IS '反馈状态+时间复合索引 - 支持按状态和时间排序查询';

-- =============================================================================
-- 5. FILES TABLE (tenant-service) - Index for tenant file listings
-- =============================================================================
-- The files table tracks tenant file storage usage. Queries filter by tenant_id.

CREATE INDEX IF NOT EXISTS "idx_files_tenant_id" ON "public"."files" USING btree ("tenant_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_files_tenant_id" IS '租户文件索引 - 支持按租户查询文件统计';

-- =============================================================================
-- 6. TENANT_USERS TABLE - Additional indexes for user lookups
-- =============================================================================
-- While tenant_users has a UNIQUE(tenant_id, user_id) constraint, we need
-- an index on user_id alone for reverse lookups (find all tenants for a user).

CREATE INDEX IF NOT EXISTS "idx_tenant_users_user_id" ON "public"."tenant_users" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_tenant_users_user_id" IS '租户用户反向索引 - 支持查询用户所属的所有租户';

CREATE INDEX IF NOT EXISTS "idx_tenant_users_status" ON "public"."tenant_users" USING btree ("status" "pg_catalog"."int4_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_tenant_users_status" IS '租户用户状态索引 - 支持按状态筛选租户成员';

-- =============================================================================
-- 7. SYS_FEEDBACK TABLE - Composite index for user + status queries
-- =============================================================================
-- Admin dashboards query feedbacks by status; users query by their own user_id.

CREATE INDEX IF NOT EXISTS "idx_sys_feedback_user_status" ON "public"."sys_feedback" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_sys_feedback_user_status' IS '系统反馈用户+状态复合索引 - 支持查询用户反馈及状态筛选';

-- =============================================================================
-- 8. SCHEDULE_TASKS TABLE - Index for tenant task listings
-- =============================================================================
-- Scheduled tasks are queried by tenant with status filtering.

CREATE INDEX IF NOT EXISTS "idx_schedule_tasks_tenant_status" ON "public"."schedule_tasks" USING btree ("tenant_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_schedule_tasks_tenant_status" IS '定时任务租户+状态复合索引 - 支持按租户和状态查询定时任务';

-- =============================================================================
-- 9. DICTIONARY_ITEMS TABLE - Index for type-based lookups
-- =============================================================================
-- Dictionary items are frequently queried by type_id to get all items of a type.

CREATE INDEX IF NOT EXISTS "idx_dictionary_items_type_id" ON "public"."dictionary_items" USING btree ("type_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_dictionary_items_type_id" IS '字典项类型索引 - 支持按字典类型查询所有字典项';

-- =============================================================================
-- 10. TASK_RECORDS TABLE - Index for workflow instance task lookups
-- =============================================================================
-- Task records are queried by instance_id to get all tasks in a workflow instance.

CREATE INDEX IF NOT EXISTS "idx_task_records_instance_id" ON "public"."task_records" USING btree ("instance_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_task_records_instance_id' IS '任务记录实例索引 - 支持按工作流实例查询任务记录';

-- =============================================================================
-- 11. TASK_EXECUTIONS TABLE - Index for task execution lookups
-- =============================================================================
-- Task executions are queried by task_id to get execution history.

CREATE INDEX IF NOT EXISTS "idx_task_executions_task_id" ON "public"."task_executions" USING btree ("task_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_task_executions_task_id" IS '任务执行索引 - 支持按任务查询执行记录';

-- =============================================================================
-- 12. SYS_MESSAGE_USER TABLE - Index for message lookups
-- =============================================================================
-- Message-user associations are queried by message_id for broadcast messages.

CREATE INDEX IF NOT EXISTS "idx_sys_message_user_message_id" ON "public"."sys_message_user" USING btree ("message_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_sys_message_user_message_id" IS '消息用户关联索引 - 支持按消息查询接收用户';

-- =============================================================================
-- 13. USER_DEPARTMENTS TABLE - Index for department lookups
-- =============================================================================
-- User-department associations are queried by department_id to find department members.

CREATE INDEX IF NOT EXISTS "idx_user_departments_department_id" ON "public"."user_departments" USING btree ("department_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_user_departments_department_id" IS '用户部门关联索引 - 支持按部门查询所有成员';

-- =============================================================================
-- 14. API_USAGE_LOGS TABLE - Composite index for tenant usage analytics
-- =============================================================================
-- API usage logs are queried by tenant with time ranges for billing/analytics.

CREATE INDEX IF NOT EXISTS "idx_api_usage_logs_tenant_created" ON "public"."api_usage_logs" USING btree ("tenant_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);
COMMENT ON INDEX "public"."idx_api_usage_logs_tenant_created" IS 'API用量租户+时间复合索引 - 支持按租户和时间范围统计API用量';

-- =============================================================================
-- 15. ANNOUNCEMENTS TABLE - Index for active announcements
-- =============================================================================
-- Announcements are queried by active status and time range for display.

CREATE INDEX IF NOT EXISTS "idx_announcements_active_time" ON "public"."announcements" USING btree ("is_active" "pg_catalog"."bool_ops" ASC NULLS LAST, "start_time" "pg_catalog"."timestamptz_ops" ASC NULLS LAST, "end_time" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_announcements_active_time" IS '公告活跃状态+时间复合索引 - 支持查询当前有效的公告';

-- =============================================================================
-- 16. TENANTS TABLE - Index for code lookups
-- =============================================================================
-- Tenants are looked up by code in multi-tenant routing.

CREATE INDEX IF NOT EXISTS "idx_tenants_code" ON "public"."tenants" USING btree ("code" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_tenants_code" IS '租户编码索引 - 支持按编码快速查找租户';

-- =============================================================================
-- 17. TENANTS TABLE - Index for status filtering
-- =============================================================================
-- Admin dashboards filter tenants by status (active/suspended/expired).

CREATE INDEX IF NOT EXISTS "idx_tenants_status" ON "public"."tenants" USING btree ("status" "pg_catalog"."int4_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_tenants_status" IS '租户状态索引 - 支持按状态筛选租户';

-- =============================================================================
-- 18. WORKFLOW_INSTANCES TABLE - Composite index for status + time
-- =============================================================================
-- Workflow dashboards query by status with time ordering.

CREATE INDEX IF NOT EXISTS "idx_workflow_instances_status_started" ON "public"."workflow_instances" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST, "started_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);
COMMENT ON INDEX "public"."idx_workflow_instances_status_started" IS '工作流实例状态+时间复合索引 - 支持按状态和时间排序查询';

-- =============================================================================
-- 19. CMS_ARTICLE TABLE - Composite index for category + status
-- =============================================================================
-- Article listings filter by category and status.

CREATE INDEX IF NOT EXISTS "idx_cms_article_category_status" ON "public"."cms_article" USING btree ("category_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "status" "pg_catalog"."int4_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_cms_article_category_status' IS '文章分类+状态复合索引 - 支持按分类和状态筛选文章';

-- =============================================================================
-- 20. SESSIONS TABLE - Composite index for cleanup queries
-- =============================================================================
-- Session cleanup jobs query by user_id and expires_at.

CREATE INDEX IF NOT EXISTS "idx_sessions_user_expires" ON "public"."sessions" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "expires_at" "pg_catalog"."timestamp_ops" ASC NULLS LAST);
COMMENT ON INDEX "public"."idx_sessions_user_expires' IS '会话用户+过期时间复合索引 - 支持会话清理和过期查询';

-- =============================================================================
-- END OF FILE
-- Total indexes added: 20
-- =============================================================================

-- =============================================================================
-- MyAI Backend Database Schema - Reset (DESTRUCTIVE)
-- =============================================================================
-- WARNING: This file contains DROP TABLE IF EXISTS statements!
-- It WILL DELETE all existing data.
-- Only use for database reset or initial deployment.
-- =============================================================================

-- 文件分类枚举
DROP TYPE IF EXISTS "public"."file_category";
-- =============================================================================

DROP SEQUENCE IF EXISTS "public"."announcements_id_seq";

DROP SEQUENCE IF EXISTS "public"."api_key_usage_logs_id_seq";

DROP SEQUENCE IF EXISTS "public"."cms_article_id_seq";

DROP SEQUENCE IF EXISTS "public"."cms_category_id_seq";

DROP SEQUENCE IF EXISTS "public"."cms_comment_id_seq";

DROP SEQUENCE IF EXISTS "public"."departments_id_seq";

DROP SEQUENCE IF EXISTS "public"."permissions_id_seq";

DROP SEQUENCE IF EXISTS "public"."role_permissions_id_seq";

DROP SEQUENCE IF EXISTS "public"."role_templates_id_seq";

DROP SEQUENCE IF EXISTS "public"."roles_id_seq";

DROP SEQUENCE IF EXISTS "public"."sessions_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_feedback_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_files_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_ip_whitelist_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_login_device_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_login_logs_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_message_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_message_user_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_operation_logs_id_seq";

DROP SEQUENCE IF EXISTS "public"."sys_sensitive_audits_id_seq";

DROP SEQUENCE IF EXISTS "public"."system_configs_id_seq";

DROP SEQUENCE IF EXISTS "public"."user_departments_id_seq";

DROP SEQUENCE IF EXISTS "public"."user_roles_id_seq";

DROP SEQUENCE IF EXISTS "public"."users_id_seq";

DROP SEQUENCE IF EXISTS "public"."tow_car_id_seq";

DROP SEQUENCE IF EXISTS "public"."tow_car_class_id_seq";

DROP SEQUENCE IF EXISTS "public"."tow_car_type_id_seq";

DROP SEQUENCE IF EXISTS "public"."tow_car_color_id_seq";

DROP SEQUENCE IF EXISTS "public"."tow_dc_type_id_seq";

DROP SEQUENCE IF EXISTS "public"."tow_dc_causes_id_seq";

DROP SEQUENCE IF EXISTS "public"."message_templates_id_seq";
-- 用户主表
DROP TABLE IF EXISTS "public"."users";
-- 部门表
DROP TABLE IF EXISTS "public"."departments";
-- 会话表
DROP TABLE IF EXISTS "public"."sessions";
-- 登录日志表
DROP TABLE IF EXISTS "public"."sys_login_logs";
-- 登录设备表
DROP TABLE IF EXISTS "public"."sys_login_device";
-- 用户角色关联表
DROP TABLE IF EXISTS "public"."user_roles";
-- 用户部门关联表
DROP TABLE IF EXISTS "public"."user_departments";
-- 角色表
DROP TABLE IF EXISTS "public"."roles";
-- 权限菜单表
DROP TABLE IF EXISTS "public"."permissions";
-- 角色权限关联表
DROP TABLE IF EXISTS "public"."role_permissions";
-- 角色功能权限表
DROP TABLE IF EXISTS "public"."role_function_permissions";
-- 角色数据权限表
DROP TABLE IF EXISTS "public"."role_data_permissions";
-- 角色字段权限表
DROP TABLE IF EXISTS "public"."role_field_permissions";
-- 角色模板表
DROP TABLE IF EXISTS "public"."role_templates";
-- 权限继承表
DROP TABLE IF EXISTS "public"."permission_inheritances";
-- 权限过期记录表
DROP TABLE IF EXISTS "public"."permission_expiry";
-- 权限过期配置表
DROP TABLE IF EXISTS "public"."permission_expiries";
-- 权限变更日志表
DROP TABLE IF EXISTS "public"."permission_change_logs";
-- 数据权限规则表
DROP TABLE IF EXISTS "public"."data_permission_rules";
-- 字段权限配置表
DROP TABLE IF EXISTS "public"."field_permission_configs";
-- API权限定义表
DROP TABLE IF EXISTS "public"."api_permissions";
-- CMS文章分类表
DROP TABLE IF EXISTS "public"."cms_category";
-- CMS文章表
DROP TABLE IF EXISTS "public"."cms_article";
-- CMS文章评论表
DROP TABLE IF EXISTS "public"."cms_comment";
-- 站内信消息表
DROP TABLE IF EXISTS "public"."sys_message";
-- 用户消息关联表
DROP TABLE IF EXISTS "public"."sys_message_user";
-- 系统公告表
DROP TABLE IF EXISTS "public"."announcements";
-- 消息模板表
DROP TABLE IF EXISTS "public"."message_templates";
-- 字典类型表
DROP TABLE IF EXISTS "public"."dictionary_types";
-- 字典项表
DROP TABLE IF EXISTS "public"."dictionary_items";
-- 意见反馈表
DROP TABLE IF EXISTS "public"."sys_feedback";
-- 工作流定义表
DROP TABLE IF EXISTS "public"."workflows";
-- 工作流节点表
DROP TABLE IF EXISTS "public"."workflow_nodes";
-- 工作流连线表
DROP TABLE IF EXISTS "public"."workflow_edges";
-- 工作流实例表
DROP TABLE IF EXISTS "public"."workflow_instances";
-- 任务记录表
DROP TABLE IF EXISTS "public"."task_records";
-- 系统操作日志表
DROP TABLE IF EXISTS "public"."sys_operation_logs";
-- 敏感操作审计表
DROP TABLE IF EXISTS "public"."sys_sensitive_audits";
-- IP白名单表
DROP TABLE IF EXISTS "public"."sys_ip_whitelist";
-- 报表定义表
DROP TABLE IF EXISTS "public"."reports";
-- 报表任务表
DROP TABLE IF EXISTS "public"."report_tasks";
-- 调度任务表
DROP TABLE IF EXISTS "public"."schedule_tasks";
-- 任务执行记录表
DROP TABLE IF EXISTS "public"."task_executions";
-- 数据源配置表
DROP TABLE IF EXISTS "public"."data_sources";
-- API密钥表
DROP TABLE IF EXISTS "public"."api_keys";
-- API密钥使用记录表
DROP TABLE IF EXISTS "public"."api_key_usage_logs";
-- 拖车车辆表
DROP TABLE IF EXISTS "public"."tow_car";
-- 车辆分类表
DROP TABLE IF EXISTS "public"."tow_car_class";
-- 车辆类型表
DROP TABLE IF EXISTS "public"."tow_car_type";
-- 车辆颜色表
DROP TABLE IF EXISTS "public"."tow_car_color";
-- 扣押原因类型表
DROP TABLE IF EXISTS "public"."tow_dc_type";
-- 扣押原因明细表
DROP TABLE IF EXISTS "public"."tow_dc_causes";
-- 电动自行车用户表 (旧表保留用于数据迁移，新代码使用 public.users)
DROP TABLE IF EXISTS "public"."ebike_user";
-- 需要兼容查询时请直接使用 public.users (含 ebike_user 迁移字段)。
DROP VIEW IF EXISTS "public"."user";
-- 电动自行车车辆表
DROP TABLE IF EXISTS "public"."ebike_car";
-- 电动自行车车辆历史表（2021年6月快照）
DROP TABLE IF EXISTS "public"."ebike_car_history_2021_6";
-- 电动自行车存储表
DROP TABLE IF EXISTS "public"."ebike_storage";
-- 电动自行车存储历史表（2021年6月快照）
DROP TABLE IF EXISTS "public"."ebike_storage_history_2021_6";
-- 电动自行车订单表（2021年6月快照）
DROP TABLE IF EXISTS "public"."ebike_order_2021_6";
-- 电动自行车配置选项表
DROP TABLE IF EXISTS "public"."ebike_options";
-- 运营商配额表
DROP TABLE IF EXISTS "public"."operators";
-- 违停记录表
DROP TABLE IF EXISTS "public"."violations";
-- 系统配置表
DROP TABLE IF EXISTS "public"."system_configs";
-- API 调用日志表
DROP SEQUENCE IF EXISTS "public"."sys_api_call_logs_id_seq";

DROP TABLE IF EXISTS "public"."sys_api_call_logs";
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_lpr_pass_records_updated_at ON public.lpr_pass_records;

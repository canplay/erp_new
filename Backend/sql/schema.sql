-- =============================================================================
-- MyAI Backend Database Schema
-- WARNING: This file contains DROP TABLE IF EXISTS statements!
-- It WILL DELETE all existing data.
-- Only use for initial deployment or database reset.
-- For existing databases with data, migrate manually.
-- All CREATE/INSERT use IF NOT EXISTS / NOT EXISTS for idempotent execution.
-- =============================================================================

/* 安全加固: 创建应用专用角色 (修复: 原使用 postgres 超级用户连库) */
DO $$
BEGIN
    -- 创建应用角色(如果不存在)
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'myai_app') THEN
        CREATE ROLE myai_app LOGIN PASSWORD :'MYAI_APP_PASSWORD';
    END IF;
    -- 授予 schema 使用权限
    GRANT USAGE ON SCHEMA public TO myai_app;
    GRANT USAGE ON SCHEMA socialops TO myai_app;
END $$;

-- 注意: 表级权限在每张表创建后通过 ALTER TABLE ... OWNER TO myai_app 设置

/*
===============================================================================
  MyAI Backend Database Schema (Merged)
  PostgreSQL 18+
  Generated: 2026-06-10
  Version: 4.0 (All services + E-bike tables)
===============================================================================

  Service Mapping (Backend Microservices):
  ─────────────────────────────────────────────────────────────────────────────
  [api-gateway]          端口 8090  - API 网关，统一入口
  [auth-service]         端口 8081/9091 - 认证授权、RBAC 权限
  [user-service]         端口 8080/9090 - 用户管理、组织架构
  [cms-service]          端口 8082/9082 - 内容管理、文章评论
  [messaging-service]    端口 8083/9083 - 站内信、公告通知
  [file-service]         端口 8086/9092 - 文件存储管理
  [tenant-service]       端口 8087/9095 - 多租户管理
  [workflow-service]     端口 8088/9088 - 工作流引擎
  [feedback-service]     端口 8085/9085 - 意见反馈
  [audit-service]        端口 8089/9010 - 操作审计、安全
  [api-key-service]      端口 8091/9014 - API Key 管理
  [hik-service]          端口 8092/9096 - 海康设备集成
  [ctp-service]          端口 8094     - CTP 平板锁协议对接（无 gRPC）
  [lpr-service]          端口 8097/9099 - 车牌识别服务（Vz 相机回调）
  [xlt-service]          端口 8095     - 信路通 MQTT 停车协议对接
  [tow-service]          端口 0/9086   - 拖车服务
  [pay-service]          端口 8093/9097 - 支付服务
  [clean-service]        端口 8095/9087 - 数据清理、报表
  [ebike-service]        端口 8096/9098 - 电动自行车管理（车辆、订单、存储）
===============================================================================

  Database Architecture:
  ─────────────────────────────────────────────────────────────────────────────
  Mode 1: Shared Database (Default)
    All services share a single database via DATABASE_URL.
    Tables use the "public" schema with service-prefixed naming.

  Mode 2: Per-Service Database
    Each service connects to its own database via {SERVICE_NAME}_DB_URL
    (e.g., USER_SERVICE_DB_URL, AUTH_SERVICE_DB_URL).

  Mode 3: Schema-Based Isolation (SERVICE_SCHEMA)
    Each service operates within its own PostgreSQL schema.
    Set SERVICE_SCHEMA=<service_name> env var to enable.
    Tables are created under the service-specific schema instead of "public".

  Table Service Mapping:
  ─────────────────────────────────────────────────────────────────────────────
  ● user-service (用户/组织)
    ├── users                    - 用户主表
    ├── departments              - 部门表
    ├── user_roles               - 用户角色关联
    ├── user_departments         - 用户部门关联
    ├── sessions                 - 会话管理
    ├── sys_login_logs           - 登录日志
    └── sys_login_device         - 登录设备

  ● auth-service (认证授权/RBAC)
    ├── roles                    - 角色表
    ├── permissions              - 权限菜单表
    ├── role_permissions         - 角色权限关联
    ├── role_function_permissions  - 角色功能权限
    ├── role_data_permissions    - 角色数据权限
    ├── role_field_permissions   - 角色字段权限
    ├── role_templates           - 角色模板
    ├── permission_inheritances  - 权限继承
    ├── permission_expiry        - 权限过期记录
    ├── permission_expiries      - 权限过期配置
    ├── permission_change_logs   - 权限变更日志
    ├── data_permission_rules    - 数据权限规则
    ├── field_permission_configs - 字段权限配置
    └── api_permissions          - API 权限定义

  ● cms-service (内容管理)
    ├── cms_article              - 文章表
    ├── cms_category             - 文章分类
    └── cms_comment              - 文章评论

  ● messaging-service (消息通知)
    ├── sys_message              - 站内信
    ├── sys_message_user         - 用户消息关联
    └── announcements            - 系统公告

  ● file-service (文件管理)
    └── sys_files                - 文件存储

  ● feedback-service (意见反馈)
    └── sys_feedback             - 反馈表

  ● workflow-service (工作流)
    ├── workflows                - 工作流定义
    ├── workflow_nodes           - 工作流节点
    ├── workflow_edges           - 工作流连线
    ├── workflow_instances       - 工作流实例
    └── task_records             - 任务记录

  ● audit-service (审计安全)
    ├── sys_operation_logs       - 操作日志
    ├── sys_sensitive_audits    - 敏感操作审计
    └── sys_ip_whitelist         - IP 白名单

  ● clean-service (数据清理/报表)
    ├── reports                  - 报表定义
    ├── report_tasks             - 报表任务
    ├── schedule_tasks           - 调度任务
    ├── task_executions          - 任务执行记录
    └── data_sources             - 数据源配置

  ● hik-service (设备集成)
    └── data_sources             - 海康设备数据源

  ● api-key-service (API 密钥管理)
    ├── api_keys                 - API 密钥表
    └── api_key_usage_logs       - API 密钥使用记录

  ● tow-service (拖车服务)
    ├── tow_car                  - 车辆表
    ├── tow_car_class            - 车辆分类表（收费标准）
    ├── tow_car_type             - 车辆类型表
    ├── tow_car_color            - 车辆颜色表
    ├── tow_dc_type              - 扣押原因类型表
    └── tow_dc_causes            - 扣押原因明细表

  ● ebike-service (电动自行车服务)
    ├── car                      - 车辆表（主表）
    ├── car_history_2021_6       - 车辆历史表（2021年6月快照）
    ├── storage                  - 存储表
    ├── storage_history_2021_6   - 存储历史表（2021年6月快照）
    ├── order_2021_6             - 订单表（2021年6月快照）
    ├── options                  - 配置选项表
    └── user                     - 用户表（ebike专用）

  ● shared (多服务共用)
    └── system_configs           - 系统配置
    └── sys_api_call_logs        - API 调用日志

  Schema Isolation Usage:
  ─────────────────────────────────────────────────────────────────────────────
  -- Enable per-service schema isolation:
  --   SET SERVICE_SCHEMA = 'user_service';
  -- Tables will be created under the 'user_service' schema instead of 'public'.
  --
  -- Enable per-service database:
  --   SET USER_SERVICE_DB_URL = 'postgres://user:***@host:5432/user_db';
  --
  -- Connection pool tuning per service:
  --   SET DB_POOL_MAX_CONNECTIONS = 20
  --   SET DB_POOL_MIN_CONNECTIONS = 5
  --   SET DB_POOL_CONNECT_TIMEOUT = 30
  --   SET DB_POOL_IDLE_TIMEOUT = 600
  --   SET DB_POOL_MAX_LIFETIME = 1800
===============================================================================
*/

-- =============================================================================
-- =============================================================================
-- (清理) 历史遗留的 per-service schema 空壳已移除:
-- 所有服务表统一建在 public schema(见下方 CREATE TABLE "public".*)。
-- 唯一例外: socialops schema 由 social-ops-service 显式引用(socialops.xxx),
-- 保留于文件末尾。
-- =============================================================================

-- =============================================================================
-- Type structures (枚举类型定义)
-- =============================================================================

-- 文件分类枚举
DROP TYPE IF EXISTS "public"."file_category";
CREATE TYPE "public"."file_category" AS ENUM (
  'general',   -- 通用文件
  'image',     -- 图片文件
  'document',  -- 文档文件
  'video',     -- 视频文件
  'audio',     -- 音频文件
  'avatar',    -- 头像
  'attachment' -- 附件
);

-- =============================================================================
-- Sequence structures (序列定义)
-- =============================================================================

DROP SEQUENCE IF EXISTS "public"."announcements_id_seq";
CREATE SEQUENCE "public"."announcements_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."api_key_usage_logs_id_seq";
CREATE SEQUENCE "public"."api_key_usage_logs_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."cms_article_id_seq";
CREATE SEQUENCE "public"."cms_article_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."cms_category_id_seq";
CREATE SEQUENCE "public"."cms_category_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."cms_comment_id_seq";
CREATE SEQUENCE "public"."cms_comment_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."departments_id_seq";
CREATE SEQUENCE "public"."departments_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."permissions_id_seq";
CREATE SEQUENCE "public"."permissions_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."role_permissions_id_seq";
CREATE SEQUENCE "public"."role_permissions_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."role_templates_id_seq";
CREATE SEQUENCE "public"."role_templates_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."roles_id_seq";
CREATE SEQUENCE "public"."roles_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sessions_id_seq";
CREATE SEQUENCE "public"."sessions_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_feedback_id_seq";
CREATE SEQUENCE "public"."sys_feedback_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_files_id_seq";
CREATE SEQUENCE "public"."sys_files_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_ip_whitelist_id_seq";
CREATE SEQUENCE "public"."sys_ip_whitelist_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_login_device_id_seq";
CREATE SEQUENCE "public"."sys_login_device_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_login_logs_id_seq";
CREATE SEQUENCE "public"."sys_login_logs_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_message_id_seq";
CREATE SEQUENCE "public"."sys_message_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_message_user_id_seq";
CREATE SEQUENCE "public"."sys_message_user_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_operation_logs_id_seq";
CREATE SEQUENCE "public"."sys_operation_logs_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."sys_sensitive_audits_id_seq";
CREATE SEQUENCE "public"."sys_sensitive_audits_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."system_configs_id_seq";
CREATE SEQUENCE "public"."system_configs_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."user_departments_id_seq";
CREATE SEQUENCE "public"."user_departments_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."user_roles_id_seq";
CREATE SEQUENCE "public"."user_roles_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."users_id_seq";
CREATE SEQUENCE "public"."users_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."tow_car_id_seq";
CREATE SEQUENCE "public"."tow_car_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."tow_car_class_id_seq";
CREATE SEQUENCE "public"."tow_car_class_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."tow_car_type_id_seq";
CREATE SEQUENCE "public"."tow_car_type_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."tow_car_color_id_seq";
CREATE SEQUENCE "public"."tow_car_color_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."tow_dc_type_id_seq";
CREATE SEQUENCE "public"."tow_dc_type_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."tow_dc_causes_id_seq";
CREATE SEQUENCE "public"."tow_dc_causes_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP SEQUENCE IF EXISTS "public"."message_templates_id_seq";
CREATE SEQUENCE "public"."message_templates_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

-- =============================================================================
-- Table structures: user-service (用户服务)
-- =============================================================================

-- 用户主表
DROP TABLE IF EXISTS "public"."users";
CREATE TABLE "public"."users" (
  "id" int8 NOT NULL DEFAULT nextval('users_id_seq'::regclass),
  "username" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "nickname" varchar(100) COLLATE "pg_catalog"."default",
  "password_hash" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "avatar" varchar(500) COLLATE "pg_catalog"."default",
  "phone" varchar(20) COLLATE "pg_catalog"."default",
  "email" varchar(100) COLLATE "pg_catalog"."default",
  "gender" int4 DEFAULT 0,
  "address" varchar(500) COLLATE "pg_catalog"."default",
  "role" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'user'::character varying,
  "status" int4 DEFAULT 1,
  "must_change_password" bool DEFAULT TRUE,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."users" IS '用户主表 - 存储系统用户基本信息，包括登录凭证和个人资料';
COMMENT ON COLUMN "public"."users"."username" IS '用户名 - 用于登录系统的唯一标识';
COMMENT ON COLUMN "public"."users"."nickname" IS '昵称 - 用户显示名称，可与用户名不同';
COMMENT ON COLUMN "public"."users"."password_hash" IS '密码哈希 - 使用 bcrypt/argon2 等算法加密存储';
COMMENT ON COLUMN "public"."users"."avatar" IS '头像URL - 用户头像图片地址';
COMMENT ON COLUMN "public"."users"."phone" IS '手机号 - 用户联系电话';
COMMENT ON COLUMN "public"."users"."email" IS '邮箱 - 用户电子邮箱地址';
COMMENT ON COLUMN "public"."users"."gender" IS '性别: 0-未知, 1-男, 2-女';
COMMENT ON COLUMN "public"."users"."address" IS '地址 - 用户居住或办公地址';
COMMENT ON COLUMN "public"."users"."role" IS '用户角色: admin-管理员, user-普通用户';
COMMENT ON COLUMN "public"."users"."status" IS '状态: 1-正常, 0-禁用';
COMMENT ON COLUMN "public"."users"."created_at" IS '创建时间 - 账户创建时间戳';
COMMENT ON COLUMN "public"."users"."updated_at" IS '更新时间 - 最后一次资料更新时间戳';

-- 部门表
DROP TABLE IF EXISTS "public"."departments";
CREATE TABLE "public"."departments" (
  "id" int8 NOT NULL DEFAULT nextval('departments_id_seq'::regclass),
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "code" varchar(50) COLLATE "pg_catalog"."default",
  "parent_id" int8,
  "level" int4 DEFAULT 0,
  "sort_order" int4 DEFAULT 0,
  "leader_id" int8,
  "description" text COLLATE "pg_catalog"."default",
  "status" int4 DEFAULT 1,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."departments" IS '部门表 - 组织架构管理，支持树形结构的多级部门';
COMMENT ON COLUMN "public"."departments"."name" IS '部门名称 - 部门的显示名称';
COMMENT ON COLUMN "public"."departments"."code" IS '部门编码 - 部门的唯一标识代码';
COMMENT ON COLUMN "public"."departments"."parent_id" IS '父部门ID - 上级部门ID，用于构建树形结构';
COMMENT ON COLUMN "public"."departments"."level" IS '部门层级 - 部门的层级深度，0为顶级，最大5层';
COMMENT ON COLUMN "public"."departments"."sort_order" IS '排序号 - 同级部门的排序顺序';
COMMENT ON COLUMN "public"."departments"."leader_id" IS '部门负责人ID - 部门主管的用户ID';
COMMENT ON COLUMN "public"."departments"."description" IS '部门描述 - 部门的详细说明';
COMMENT ON COLUMN "public"."departments"."status" IS '状态: 1-正常, 0-禁用';

-- 会话表
DROP TABLE IF EXISTS "public"."sessions";
CREATE TABLE "public"."sessions" (
  "id" int8 NOT NULL DEFAULT nextval('sessions_id_seq'::regclass),
  "user_id" int8 NOT NULL,
  "refresh_token_hash" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "device_info" varchar(500) COLLATE "pg_catalog"."default",
  "ip_address" varchar(50) COLLATE "pg_catalog"."default",
  "expires_at" timestamp(6) NOT NULL,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sessions" IS '会话表 - 用户登录会话管理，用于JWT refresh token管理';
COMMENT ON COLUMN "public"."sessions"."user_id" IS '用户ID - 关联的用户ID';
COMMENT ON COLUMN "public"."sessions"."refresh_token_hash" IS '刷新令牌哈希 - JWT refresh token的哈希值，用于会话续期';
COMMENT ON COLUMN "public"."sessions"."device_info" IS '设备信息 - 登录设备详情（浏览器、操作系统等）';
COMMENT ON COLUMN "public"."sessions"."ip_address" IS 'IP地址 - 用户登录时的IP地址';
COMMENT ON COLUMN "public"."sessions"."expires_at" IS '过期时间 - 会话过期时间点';
COMMENT ON COLUMN "public"."sessions"."created_at" IS '创建时间 - 会话创建时间戳';

-- 登录日志表
DROP TABLE IF EXISTS "public"."sys_login_logs";
CREATE TABLE "public"."sys_login_logs" (
  "id" int8 NOT NULL DEFAULT nextval('sys_login_logs_id_seq'::regclass),
  "user_id" int8,
  "username" varchar(100) COLLATE "pg_catalog"."default",
  "ip_address" varchar(50) COLLATE "pg_catalog"."default",
  "user_agent" text COLLATE "pg_catalog"."default",
  "login_location" varchar(255) COLLATE "pg_catalog"."default",
  "login_status" int2,
  "fail_reason" varchar(255) COLLATE "pg_catalog"."default",
  "login_type" varchar(20) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_login_logs" IS '系统登录日志表 - 记录用户登录历史，用于安全审计和异常检测';
COMMENT ON COLUMN "public"."sys_login_logs"."user_id" IS '用户ID - 登录的用户ID（失败时可能为空）';
COMMENT ON COLUMN "public"."sys_login_logs"."username" IS '用户名 - 登录时使用的用户名';
COMMENT ON COLUMN "public"."sys_login_logs"."ip_address" IS 'IP地址 - 登录来源IP地址';
COMMENT ON COLUMN "public"."sys_login_logs"."user_agent" IS 'User-Agent - 浏览器/客户端标识';
COMMENT ON COLUMN "public"."sys_login_logs"."login_location" IS '登录地点 - 根据IP解析的地理位置';
COMMENT ON COLUMN "public"."sys_login_logs"."login_status" IS '登录状态: 1-成功, 2-失败';
COMMENT ON COLUMN "public"."sys_login_logs"."fail_reason" IS '失败原因 - 登录失败的具体原因';
COMMENT ON COLUMN "public"."sys_login_logs"."login_type" IS '登录方式: password-密码, sms-短信, oauth-第三方登录';

-- 登录设备表
DROP TABLE IF EXISTS "public"."sys_login_device";
CREATE TABLE "public"."sys_login_device" (
  "id" int8 NOT NULL DEFAULT nextval('sys_login_device_id_seq'::regclass),
  "user_id" int8 NOT NULL,
  "device_id" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "device_type" varchar(30) COLLATE "pg_catalog"."default" DEFAULT 'unknown'::character varying,
  "device_name" varchar(100) COLLATE "pg_catalog"."default",
  "browser" varchar(100) COLLATE "pg_catalog"."default",
  "os" varchar(100) COLLATE "pg_catalog"."default",
  "ip_address" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "ip_location" varchar(200) COLLATE "pg_catalog"."default",
  "login_time" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "last_active_time" timestamptz(6),
  "logout_time" timestamptz(6),
  "is_active" int2 NOT NULL DEFAULT 1,
  "is_trusted" int2 NOT NULL DEFAULT 0,
  "user_agent" text COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_login_device" IS '用户登录设备表 - 管理用户可信设备，支持设备认证和异常检测';
COMMENT ON COLUMN "public"."sys_login_device"."user_id" IS '用户ID - 关联的用户ID';
COMMENT ON COLUMN "public"."sys_login_device"."device_id" IS '设备ID - 唯一设备标识符';
COMMENT ON COLUMN "public"."sys_login_device"."device_type" IS '设备类型: desktop-桌面端, mobile-移动端, tablet-平板, other-其他';
COMMENT ON COLUMN "public"."sys_login_device"."device_name" IS '设备名称 - 用户自定义的设备名称';
COMMENT ON COLUMN "public"."sys_login_device"."browser" IS '浏览器 - 使用的浏览器名称和版本';
COMMENT ON COLUMN "public"."sys_login_device"."os" IS '操作系统 - 设备运行的操作系统';
COMMENT ON COLUMN "public"."sys_login_device"."ip_address" IS 'IP地址 - 设备登录时的IP地址';
COMMENT ON COLUMN "public"."sys_login_device"."ip_location" IS 'IP位置 - IP地址对应的地理位置';
COMMENT ON COLUMN "public"."sys_login_device"."login_time" IS '登录时间 - 首次登录时间';
COMMENT ON COLUMN "public"."sys_login_device"."last_active_time" IS '最后活跃时间 - 最后一次活跃操作时间';
COMMENT ON COLUMN "public"."sys_login_device"."logout_time" IS '登出时间 - 用户主动登出的时间';
COMMENT ON COLUMN "public"."sys_login_device"."is_active" IS '是否活跃: 1-活跃, 0-已登出';
COMMENT ON COLUMN "public"."sys_login_device"."is_trusted" IS '是否可信设备: 0-不可信, 1-可信';
COMMENT ON COLUMN "public"."sys_login_device"."user_agent" IS '完整User-Agent字符串';

-- 用户角色关联表
DROP TABLE IF EXISTS "public"."user_roles";
CREATE TABLE "public"."user_roles" (
  "id" int8 NOT NULL DEFAULT nextval('user_roles_id_seq'::regclass),
  "user_id" int8 NOT NULL,
  "role_id" int8 NOT NULL,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."user_roles" IS '用户角色关联表 - 用户与角色的多对多关系，支持一个用户拥有多个角色';
COMMENT ON COLUMN "public"."user_roles"."user_id" IS '用户ID - 关联的用户ID';
COMMENT ON COLUMN "public"."user_roles"."role_id" IS '角色ID - 关联的角色ID';
COMMENT ON COLUMN "public"."user_roles"."created_at" IS '关联时间 - 角色分配给用户的时间';

-- 用户部门关联表
DROP TABLE IF EXISTS "public"."user_departments";
CREATE TABLE "public"."user_departments" (
  "id" int8 NOT NULL DEFAULT nextval('user_departments_id_seq'::regclass),
  "user_id" int8 NOT NULL,
  "department_id" int8 NOT NULL,
  "is_primary" bool DEFAULT true,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."user_departments" IS '用户部门关联表 - 用户与部门的多对多关系，支持用户属于多个部门';
COMMENT ON COLUMN "public"."user_departments"."user_id" IS '用户ID - 关联的用户ID';
COMMENT ON COLUMN "public"."user_departments"."department_id" IS '部门ID - 关联的部门ID';
COMMENT ON COLUMN "public"."user_departments"."is_primary" IS '是否主部门: true-主部门(主要工作部门), false-兼职部门';

-- =============================================================================
-- Table structures: auth-service (认证授权服务)
-- =============================================================================

-- 角色表
DROP TABLE IF EXISTS "public"."roles";
CREATE TABLE "public"."roles" (
  "id" int8 NOT NULL DEFAULT nextval('roles_id_seq'::regclass),
  "name" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "code" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "role_type" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'custom'::character varying,
  "parent_id" int8,
  "level" int4 DEFAULT 0,
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "is_default" bool DEFAULT false,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."roles" IS '角色表 - RBAC 角色定义，支持角色继承和层级管理';
COMMENT ON COLUMN "public"."roles"."name" IS '角色名称 - 角色的显示名称，如"系统管理员"';
COMMENT ON COLUMN "public"."roles"."code" IS '角色编码 - 角色的唯一标识代码，如"admin"';
COMMENT ON COLUMN "public"."roles"."description" IS '角色描述 - 角色的详细说明和职责描述';
COMMENT ON COLUMN "public"."roles"."role_type" IS '角色类型: custom-自定义角色, system-系统内置角色';
COMMENT ON COLUMN "public"."roles"."parent_id" IS '父角色ID - 用于角色继承，上级角色ID';
COMMENT ON COLUMN "public"."roles"."level" IS '角色层级 - 角色的层级深度，0为最高级，最大3层';
COMMENT ON COLUMN "public"."roles"."sort_order" IS '排序号 - 角色列表的排序顺序';
COMMENT ON COLUMN "public"."roles"."status" IS '状态: 1-正常(可用), 0-禁用';
COMMENT ON COLUMN "public"."roles"."is_default" IS '是否默认角色: true-新用户默认分配, false-需手动分配';

-- 权限菜单表
DROP TABLE IF EXISTS "public"."permissions";
CREATE TABLE "public"."permissions" (
  "id" int8 NOT NULL DEFAULT nextval('permissions_id_seq'::regclass),
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "code" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "permission_type" varchar(20) COLLATE "pg_catalog"."default" NOT NULL,
  "parent_id" int8,
  "path" varchar(255) COLLATE "pg_catalog"."default",
  "method" varchar(10) COLLATE "pg_catalog"."default",
  "icon" varchar(50) COLLATE "pg_catalog"."default",
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."permissions" IS '权限菜单表 - 功能权限与菜单定义，支持多级菜单和按钮级别权限';
COMMENT ON COLUMN "public"."permissions"."name" IS '权限名称 - 权限的显示名称，如"用户列表"';
COMMENT ON COLUMN "public"."permissions"."code" IS '权限编码 - 权限的唯一标识，如"user:list"';
COMMENT ON COLUMN "public"."permissions"."permission_type" IS '权限类型: menu-菜单权限, button-按钮权限, api-API权限, data-数据权限';
COMMENT ON COLUMN "public"."permissions"."parent_id" IS '父权限ID - 用于构建菜单树';
COMMENT ON COLUMN "public"."permissions"."path" IS '访问路径 - 对应的API或页面路径';
COMMENT ON COLUMN "public"."permissions"."method" IS '请求方法 - HTTP方法(GET/POST/PUT/DELETE)，用于API权限';
COMMENT ON COLUMN "public"."permissions"."icon" IS '菜单图标 - 前端显示的图标标识';
COMMENT ON COLUMN "public"."permissions"."sort_order" IS '排序号 - 同级权限的排序顺序';
COMMENT ON COLUMN "public"."permissions"."status" IS '状态: 1-正常, 0-禁用';

-- 角色权限关联表
DROP TABLE IF EXISTS "public"."role_permissions";
CREATE TABLE "public"."role_permissions" (
  "id" int8 NOT NULL DEFAULT nextval('role_permissions_id_seq'::regclass),
  "role_id" int8 NOT NULL,
  "permission_id" int8 NOT NULL,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."role_permissions" IS '角色权限关联表 - 角色与权限的多对多关系，定义角色拥有的权限';
COMMENT ON COLUMN "public"."role_permissions"."role_id" IS '角色ID - 关联的角色ID';
COMMENT ON COLUMN "public"."role_permissions"."permission_id" IS '权限ID - 关联的权限ID';
COMMENT ON COLUMN "public"."role_permissions"."created_at" IS '授权时间 - 权限分配给角色的时间';

-- 角色功能权限表
DROP TABLE IF EXISTS "public"."role_function_permissions";
CREATE TABLE "public"."role_function_permissions" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL DEFAULT (gen_random_uuid())::text,
  "role_id" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "permission" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."role_function_permissions" IS '角色功能权限表 - 细粒度功能权限配置，用于控制角色的具体操作权限';
COMMENT ON COLUMN "public"."role_function_permissions"."role_id" IS '角色ID - 关联的角色ID';
COMMENT ON COLUMN "public"."role_function_permissions"."permission" IS '功能权限标识 - 具体的功能权限代码';

-- 角色数据权限表
DROP TABLE IF EXISTS "public"."role_data_permissions";
CREATE TABLE "public"."role_data_permissions" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL DEFAULT (gen_random_uuid())::text,
  "role_id" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "resource_type" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "data_scope" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'own'::character varying,
  "filter_expression" text COLLATE "pg_catalog"."default",
  "allowed_department_ids" jsonb,
  "allowed_user_ids" jsonb,
  "priority" int4 NOT NULL DEFAULT 0,
  "enabled" bool NOT NULL DEFAULT true,
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."role_data_permissions" IS '角色数据权限表 - 数据访问范围控制，限制角色能看到的数据范围';
COMMENT ON COLUMN "public"."role_data_permissions"."role_id" IS '角色ID - 关联的角色ID';
COMMENT ON COLUMN "public"."role_data_permissions"."resource_type" IS '资源类型 - 数据权限适用的资源类型，如"orders"、"reports"';
COMMENT ON COLUMN "public"."role_data_permissions"."data_scope" IS '数据范围: all-全部数据可见, own-仅本人数据, department-本部门数据, dept_with_children-本部门及下级数据, custom-自定义范围';
COMMENT ON COLUMN "public"."role_data_permissions"."filter_expression" IS '过滤表达式 - 自定义数据过滤条件(DSL)';
COMMENT ON COLUMN "public"."role_data_permissions"."allowed_department_ids" IS '允许的部门ID列表 - JSON数组，仅custom范围时使用';
COMMENT ON COLUMN "public"."role_data_permissions"."allowed_user_ids" IS '允许的用户ID列表 - JSON数组，仅custom范围时使用';
COMMENT ON COLUMN "public"."role_data_permissions"."priority" IS '优先级 - 多个规则时的优先级，数字越大越优先';
COMMENT ON COLUMN "public"."role_data_permissions"."enabled" IS '是否启用 - true-规则生效, false-规则禁用';

-- 角色字段权限表
DROP TABLE IF EXISTS "public"."role_field_permissions";
CREATE TABLE "public"."role_field_permissions" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL DEFAULT (gen_random_uuid())::text,
  "role_id" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "resource_type" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "field_name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "permission" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'read_write'::character varying,
  "mask_pattern" varchar(100) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."role_field_permissions" IS '角色字段权限表 - 字段级访问控制，控制角色对数据字段的操作权限';
COMMENT ON COLUMN "public"."role_field_permissions"."role_id" IS '角色ID - 关联的角色ID';
COMMENT ON COLUMN "public"."role_field_permissions"."resource_type" IS '资源类型 - 字段权限适用的资源类型';
COMMENT ON COLUMN "public"."role_field_permissions"."field_name" IS '字段名称 - 受控的具体字段名';
COMMENT ON COLUMN "public"."role_field_permissions"."permission" IS '字段权限: read_write-可读写, read_only-只读, hidden-隐藏字段, masked-脱敏显示, none-无权限';
COMMENT ON COLUMN "public"."role_field_permissions"."mask_pattern" IS '脱敏模式 - 脱敏显示时的格式模板';

-- 角色模板表
DROP TABLE IF EXISTS "public"."role_templates";
CREATE TABLE "public"."role_templates" (
  "id" int8 NOT NULL DEFAULT nextval('role_templates_id_seq'::regclass),
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "permissions" jsonb,
  "role_type" varchar(20) COLLATE "pg_catalog"."default",
  "is_system" bool DEFAULT false,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."role_templates" IS '角色模板表 - 预定义角色模板，包含常用权限组合，便于快速创建角色';
COMMENT ON COLUMN "public"."role_templates"."name" IS '模板名称 - 模板的显示名称';
COMMENT ON COLUMN "public"."role_templates"."description" IS '模板描述 - 模板的详细说明';
COMMENT ON COLUMN "public"."role_templates"."permissions" IS '权限列表 - JSON格式的权限ID数组';
COMMENT ON COLUMN "public"."role_templates"."role_type" IS '适用角色类型 - 模板适用的角色类型';
COMMENT ON COLUMN "public"."role_templates"."is_system" IS '是否系统模板 - true-系统预置不可删除, false-用户自定义';

-- 权限继承表
DROP TABLE IF EXISTS "public"."permission_inheritances";
CREATE TABLE "public"."permission_inheritances" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "parent_role_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "child_role_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "inherit_data_permissions" bool NOT NULL DEFAULT true,
  "inherit_field_permissions" bool NOT NULL DEFAULT true,
  "override_child_permissions" bool NOT NULL DEFAULT false,
  "priority" int4 NOT NULL DEFAULT 0,
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."permission_inheritances" IS '权限继承表 - 父子角色间的权限继承关系，子角色自动继承父角色权限';
COMMENT ON COLUMN "public"."permission_inheritances"."parent_role_id" IS '父角色ID - 权限来源的角色';
COMMENT ON COLUMN "public"."permission_inheritances"."child_role_id" IS '子角色ID - 继承权限的角色';
COMMENT ON COLUMN "public"."permission_inheritances"."inherit_data_permissions" IS '继承数据权限 - true-子角色继承父角色的数据权限规则';
COMMENT ON COLUMN "public"."permission_inheritances"."inherit_field_permissions" IS '继承字段权限 - true-子角色继承父角色的字段权限规则';
COMMENT ON COLUMN "public"."permission_inheritances"."override_child_permissions" IS '覆盖子权限 - true-父角色权限优先级高于子角色已有权限';
COMMENT ON COLUMN "public"."permission_inheritances"."priority" IS '优先级 - 继承规则优先级';

-- 权限过期记录表
DROP TABLE IF EXISTS "public"."permission_expiry";
CREATE TABLE "public"."permission_expiry" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "user_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "role_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "permission" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'function'::character varying,
  "expires_at" timestamptz(6) NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."permission_expiry" IS '权限过期记录表 - 用户临时权限的过期时间记录，支持限时授权';
COMMENT ON COLUMN "public"."permission_expiry"."user_id" IS '用户ID - 被授予临时权限的用户';
COMMENT ON COLUMN "public"."permission_expiry"."role_id" IS '角色ID - 临时的角色ID';
COMMENT ON COLUMN "public"."permission_expiry"."permission" IS '权限类型: function-功能权限, data-数据权限';
COMMENT ON COLUMN "public"."permission_expiry"."expires_at" IS '过期时间 - 临时权限的失效时间点';

-- 权限过期配置表
DROP TABLE IF EXISTS "public"."permission_expiries";
CREATE TABLE "public"."permission_expiries" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL DEFAULT (gen_random_uuid())::text,
  "user_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "role_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "permission_type" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'function'::character varying,
  "resource_id" varchar(36) COLLATE "pg_catalog"."default",
  "granted_at" timestamptz(6) NOT NULL,
  "expires_at" timestamptz(6) NOT NULL,
  "auto_revoke" bool NOT NULL DEFAULT true,
  "notify_before_expiry" bool NOT NULL DEFAULT true,
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."permission_expiries" IS '权限过期配置表 - 临时权限的有效期配置，支持自动撤销和到期提醒';
COMMENT ON COLUMN "public"."permission_expiries"."user_id" IS '用户ID - 被授权的用户ID';
COMMENT ON COLUMN "public"."permission_expiries"."role_id" IS '角色ID - 临时的角色ID';
COMMENT ON COLUMN "public"."permission_expiries"."permission_type" IS '权限类型: function-功能权限, data-数据权限, field-字段权限';
COMMENT ON COLUMN "public"."permission_expiries"."resource_id" IS '资源ID - 针对特定资源的权限';
COMMENT ON COLUMN "public"."permission_expiries"."granted_at" IS '授予时间 - 权限授予的时间点';
COMMENT ON COLUMN "public"."permission_expiries"."expires_at" IS '过期时间 - 权限失效的时间点';
COMMENT ON COLUMN "public"."permission_expiries"."auto_revoke" IS '到期自动撤销 - true-过期后自动移除权限, false-需手动撤销';
COMMENT ON COLUMN "public"."permission_expiries"."notify_before_expiry" IS '到期前通知 - true-过期前发送提醒通知';

-- 权限变更日志表
DROP TABLE IF EXISTS "public"."permission_change_logs";
CREATE TABLE "public"."permission_change_logs" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL DEFAULT (gen_random_uuid())::text,
  "role_name" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "operator" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "change_type" varchar(20) COLLATE "pg_catalog"."default" NOT NULL,
  "permission" varchar(200) COLLATE "pg_catalog"."default" NOT NULL,
  "old_value" text COLLATE "pg_catalog"."default",
  "new_value" text COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."permission_change_logs" IS '权限变更日志表 - 角色权限变更的审计记录，记录所有权限修改操作';
COMMENT ON COLUMN "public"."permission_change_logs"."role_name" IS '角色名称 - 权限被修改的角色名称';
COMMENT ON COLUMN "public"."permission_change_logs"."operator" IS '操作人 - 执行权限变更的管理员';
COMMENT ON COLUMN "public"."permission_change_logs"."change_type" IS '变更类型: add-添加权限, remove-移除权限, update-更新权限';
COMMENT ON COLUMN "public"."permission_change_logs"."permission" IS '权限标识 - 被修改的权限代码或名称';
COMMENT ON COLUMN "public"."permission_change_logs"."old_value" IS '变更前值 - 权限修改前的配置';
COMMENT ON COLUMN "public"."permission_change_logs"."new_value" IS '变更后值 - 权限修改后的配置';

-- 数据权限规则表
DROP TABLE IF EXISTS "public"."data_permission_rules";
CREATE TABLE "public"."data_permission_rules" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "role_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "resource_type" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "data_scope" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'own'::character varying,
  "filter_expression" text COLLATE "pg_catalog"."default",
  "allowed_department_ids" jsonb,
  "allowed_user_ids" jsonb,
  "priority" int4 NOT NULL DEFAULT 0,
  "enabled" bool NOT NULL DEFAULT true,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."data_permission_rules" IS '数据权限规则表 - 自定义数据过滤规则，用于复杂的数据权限场景';
COMMENT ON COLUMN "public"."data_permission_rules"."role_id" IS '角色ID - 规则所属的角色';
COMMENT ON COLUMN "public"."data_permission_rules"."resource_type" IS '资源类型 - 规则适用的数据类型';
COMMENT ON COLUMN "public"."data_permission_rules"."data_scope" IS '数据范围: all-全部, own-仅本人, department-本部门, department_and_child-本部门及下级, custom-自定义';
COMMENT ON COLUMN "public"."data_permission_rules"."filter_expression" IS '过滤表达式 - 自定义数据过滤条件';
COMMENT ON COLUMN "public"."data_permission_rules"."allowed_department_ids" IS '允许的部门 - JSON格式的部门ID列表';
COMMENT ON COLUMN "public"."data_permission_rules"."allowed_user_ids" IS '允许的用户 - JSON格式的用户ID列表';
COMMENT ON COLUMN "public"."data_permission_rules"."priority" IS '优先级 - 规则执行优先级';
COMMENT ON COLUMN "public"."data_permission_rules"."enabled" IS '是否启用 - true-规则生效';

-- 字段权限配置表
DROP TABLE IF EXISTS "public"."field_permission_configs";
CREATE TABLE "public"."field_permission_configs" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "role_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "resource_type" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "field_name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "permission" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'read_write'::character varying,
  "mask_pattern" varchar(100) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."field_permission_configs" IS '字段权限配置表 - 字段级访问控制配置，细粒度控制角色对字段的操作';
COMMENT ON COLUMN "public"."field_permission_configs"."role_id" IS '角色ID - 配置所属的角色';
COMMENT ON COLUMN "public"."field_permission_configs"."resource_type" IS '资源类型 - 配置适用的数据资源';
COMMENT ON COLUMN "public"."field_permission_configs"."field_name" IS '字段名称 - 受控的数据库字段名';
COMMENT ON COLUMN "public"."field_permission_configs"."permission" IS '权限级别: read_write-可读写, read_only-只读, hidden-隐藏, masked-脱敏, none-无权限';
COMMENT ON COLUMN "public"."field_permission_configs"."mask_pattern" IS '脱敏模式 - 数据脱敏的格式模板';

-- API权限定义表
DROP TABLE IF EXISTS "public"."api_permissions";
CREATE TABLE "public"."api_permissions" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL DEFAULT (gen_random_uuid())::text,
  "code" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "module" varchar(50) COLLATE "pg_catalog"."default",
  "category" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'api'::character varying,
  "sensitive" bool DEFAULT false,
  "status" int4 DEFAULT 1,
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."api_permissions" IS 'API权限定义表 - API 访问权限的细粒度控制，定义和保护API端点';
COMMENT ON COLUMN "public"."api_permissions"."code" IS '权限编码 - API权限的唯一标识';
COMMENT ON COLUMN "public"."api_permissions"."name" IS '权限名称 - API权限的显示名称';
COMMENT ON COLUMN "public"."api_permissions"."description" IS '权限描述 - API权限的详细说明';
COMMENT ON COLUMN "public"."api_permissions"."module" IS '所属模块: user-用户, role-角色, department-部门, cms-内容, message-消息, file-文件 等';
COMMENT ON COLUMN "public"."api_permissions"."category" IS '权限类别: api-接口权限, data-数据权限';
COMMENT ON COLUMN "public"."api_permissions"."sensitive" IS '是否敏感操作: true-敏感(如删除、导出), false-普通';
COMMENT ON COLUMN "public"."api_permissions"."status" IS '状态: 1-启用, 0-禁用';

-- =============================================================================
-- Table structures: cms-service (内容管理服务)
-- =============================================================================

-- CMS文章分类表
DROP TABLE IF EXISTS "public"."cms_category";
CREATE TABLE "public"."cms_category" (
  "id" int8 NOT NULL DEFAULT nextval('cms_category_id_seq'::regclass),
  "parent_id" int8 NOT NULL DEFAULT 0,
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "slug" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "description" varchar(500) COLLATE "pg_catalog"."default",
  "icon" varchar(100) COLLATE "pg_catalog"."default",
  "sort_order" int4 NOT NULL DEFAULT 0,
  "seo_title" varchar(200) COLLATE "pg_catalog"."default",
  "seo_keywords" varchar(500) COLLATE "pg_catalog"."default",
  "seo_description" varchar(1000) COLLATE "pg_catalog"."default",
  "status" int2 NOT NULL DEFAULT 1,
  "allow_attachment" int2 NOT NULL DEFAULT 1,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
);
COMMENT ON TABLE "public"."cms_category" IS 'CMS文章分类表 - 内容分类管理，支持树形结构和多级分类';
COMMENT ON COLUMN "public"."cms_category"."parent_id" IS '父分类ID - 上级分类ID，0为顶级';
COMMENT ON COLUMN "public"."cms_category"."name" IS '分类名称 - 分类的显示名称';
COMMENT ON COLUMN "public"."cms_category"."slug" IS '分类别名 - URL友好的唯一标识';
COMMENT ON COLUMN "public"."cms_category"."description" IS '分类描述 - 分类的详细说明';
COMMENT ON COLUMN "public"."cms_category"."icon" IS '分类图标 - 前端显示的图标';
COMMENT ON COLUMN "public"."cms_category"."sort_order" IS '排序号 - 同级分类的排序顺序';
COMMENT ON COLUMN "public"."cms_category"."seo_title" IS 'SEO标题 - 搜索引擎优化标题';
COMMENT ON COLUMN "public"."cms_category"."seo_keywords" IS 'SEO关键词 - 搜索引擎优化关键词';
COMMENT ON COLUMN "public"."cms_category"."seo_description" IS 'SEO描述 - 搜索引擎优化描述';
COMMENT ON COLUMN "public"."cms_category"."status" IS '状态: 1-启用, 0-禁用';
COMMENT ON COLUMN "public"."cms_category"."allow_attachment" IS '是否允许上传附件: 1-允许, 0-禁止';

-- CMS文章表
DROP TABLE IF EXISTS "public"."cms_article";
CREATE TABLE "public"."cms_article" (
  "id" int8 NOT NULL DEFAULT nextval('cms_article_id_seq'::regclass),
  "category_id" int8 NOT NULL,
  "title" varchar(300) COLLATE "pg_catalog"."default" NOT NULL,
  "slug" varchar(300) COLLATE "pg_catalog"."default",
  "summary" varchar(1000) COLLATE "pg_catalog"."default",
  "content" text COLLATE "pg_catalog"."default" NOT NULL,
  "content_type" int4 NOT NULL DEFAULT 0,
  "cover_image" varchar(500) COLLATE "pg_catalog"."default",
  "author_id" int8 NOT NULL,
  "author_name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "tags" text,
  "view_count" int8 NOT NULL DEFAULT 0,
  "like_count" int8 NOT NULL DEFAULT 0,
  "comment_count" int8 NOT NULL DEFAULT 0,
  "share_count" int8 NOT NULL DEFAULT 0,
  "is_featured" bool NOT NULL DEFAULT false,
  "is_top" bool NOT NULL DEFAULT false,
  "is_draft" bool NOT NULL DEFAULT false,
  "status" int4 NOT NULL DEFAULT 1,
  "reject_reason" varchar(500) COLLATE "pg_catalog"."default",
  "published_at" timestamptz(6),
  "source" varchar(100) COLLATE "pg_catalog"."default",
  "source_url" varchar(500) COLLATE "pg_catalog"."default",
  "attachments" jsonb,
  "seo_title" varchar(200) COLLATE "pg_catalog"."default",
  "seo_keywords" varchar(500) COLLATE "pg_catalog"."default",
  "seo_description" varchar(1000) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
);
COMMENT ON TABLE "public"."cms_article" IS 'CMS文章表 - 内容管理核心表，存储所有文章内容';
COMMENT ON COLUMN "public"."cms_article"."category_id" IS '分类ID - 文章所属的分类';
COMMENT ON COLUMN "public"."cms_article"."title" IS '文章标题 - 文章的标题';
COMMENT ON COLUMN "public"."cms_article"."slug" IS '文章别名 - URL友好的唯一标识';
COMMENT ON COLUMN "public"."cms_article"."summary" IS '文章摘要 - 文章的简短描述';
COMMENT ON COLUMN "public"."cms_article"."content" IS '文章内容 - 文章正文内容';
COMMENT ON COLUMN "public"."cms_article"."content_type" IS '内容类型: richText-富文本, markdown-Markdown格式';
COMMENT ON COLUMN "public"."cms_article"."cover_image" IS '封面图片 - 文章封面图片URL';
COMMENT ON COLUMN "public"."cms_article"."author_id" IS '作者ID - 文章作者的用户ID';
COMMENT ON COLUMN "public"."cms_article"."author_name" IS '作者名称 - 文章作者显示名称';
COMMENT ON COLUMN "public"."cms_article"."tags" IS '标签 - JSON格式的标签数组';
COMMENT ON COLUMN "public"."cms_article"."view_count" IS '浏览次数 - 文章被浏览的次数';
COMMENT ON COLUMN "public"."cms_article"."like_count" IS '点赞次数 - 文章被点赞的次数';
COMMENT ON COLUMN "public"."cms_article"."comment_count" IS '评论次数 - 文章收到的评论数';
COMMENT ON COLUMN "public"."cms_article"."share_count" IS '分享次数 - 文章被分享的次数';
COMMENT ON COLUMN "public"."cms_article"."is_featured" IS '是否精选: 0-否, 1-是';
COMMENT ON COLUMN "public"."cms_article"."is_top" IS '是否置顶: 0-否, 1-是';
COMMENT ON COLUMN "public"."cms_article"."is_draft" IS '是否草稿: 0-已发布, 1-草稿';
COMMENT ON COLUMN "public"."cms_article"."status" IS '文章状态: draft-草稿, pending-待审核, published-已发布, rejected-已拒绝, archived-已归档';
COMMENT ON COLUMN "public"."cms_article"."reject_reason" IS '拒绝原因 - 审核拒绝的原因';
COMMENT ON COLUMN "public"."cms_article"."published_at" IS '发布时间 - 文章正式发布的时间';
COMMENT ON COLUMN "public"."cms_article"."source" IS '文章来源 - 如"原创"、"转载"';
COMMENT ON COLUMN "public"."cms_article"."source_url" IS '原文链接 - 转载文章的原始链接';
COMMENT ON COLUMN "public"."cms_article"."attachments" IS '附件 - JSON格式的附件列表';

-- CMS文章评论表
DROP TABLE IF EXISTS "public"."cms_comment";
CREATE TABLE "public"."cms_comment" (
  "id" int8 NOT NULL DEFAULT nextval('cms_comment_id_seq'::regclass),
  "article_id" int8 NOT NULL,
  "parent_id" int8,
  "user_id" int8 NOT NULL,
  "user_name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "user_avatar" varchar(500) COLLATE "pg_catalog"."default",
  "content" text COLLATE "pg_catalog"."default" NOT NULL,
  "like_count" int4 NOT NULL DEFAULT 0,
  "reply_count" int4 NOT NULL DEFAULT 0,
  "status" varchar(20) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'published'::character varying,
  "ip_address" varchar(50) COLLATE "pg_catalog"."default",
  "user_agent" text COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
);
COMMENT ON TABLE "public"."cms_comment" IS 'CMS文章评论表 - 用户评论与回复，支持嵌套评论';
COMMENT ON COLUMN "public"."cms_comment"."article_id" IS '文章ID - 关联的文章ID';
COMMENT ON COLUMN "public"."cms_comment"."parent_id" IS '父评论ID - 回复的评论ID，NULL为顶级评论';
COMMENT ON COLUMN "public"."cms_comment"."user_id" IS '用户ID - 评论者的用户ID';
COMMENT ON COLUMN "public"."cms_comment"."user_name" IS '用户名 - 评论者显示名称';
COMMENT ON COLUMN "public"."cms_comment"."user_avatar" IS '用户头像 - 评论者头像URL';
COMMENT ON COLUMN "public"."cms_comment"."content" IS '评论内容 - 评论正文';
COMMENT ON COLUMN "public"."cms_comment"."like_count" IS '点赞次数 - 评论被点赞数';
COMMENT ON COLUMN "public"."cms_comment"."reply_count" IS '回复次数 - 评论收到的回复数';
COMMENT ON COLUMN "public"."cms_comment"."status" IS '状态: published-已发布, pending-待审核, deleted-已删除';
COMMENT ON COLUMN "public"."cms_comment"."ip_address" IS 'IP地址 - 评论者的IP地址';
COMMENT ON COLUMN "public"."cms_comment"."user_agent" IS 'User-Agent - 评论者的浏览器信息';

-- =============================================================================
-- Table structures: messaging-service (消息通知服务)
-- =============================================================================

-- 站内信消息表
DROP TABLE IF EXISTS "public"."sys_message";
CREATE TABLE "public"."sys_message" (
  "id" int8 NOT NULL DEFAULT nextval('sys_message_id_seq'::regclass),
  "type" varchar(20) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'system'::character varying,
  "title" varchar(200) COLLATE "pg_catalog"."default" NOT NULL,
  "content" text COLLATE "pg_catalog"."default" NOT NULL,
  "sender_id" int8,
  "sender_name" varchar(100) COLLATE "pg_catalog"."default",
  "priority" int2 NOT NULL DEFAULT 0,
  "attachment_urls" jsonb,
  "target_type" varchar(20) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'all'::character varying,
  "target_ids" jsonb,
  "expire_time" timestamptz(6),
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
);
COMMENT ON TABLE "public"."sys_message" IS '站内信消息表 - 系统消息与通知，支持定向和广播消息';
COMMENT ON COLUMN "public"."sys_message"."type" IS '消息类型: system-系统通知, user-用户消息, announcement-公告';
COMMENT ON COLUMN "public"."sys_message"."title" IS '消息标题 - 消息的主题';
COMMENT ON COLUMN "public"."sys_message"."content" IS '消息内容 - 消息正文';
COMMENT ON COLUMN "public"."sys_message"."sender_id" IS '发送者ID - 消息发送者的用户ID';
COMMENT ON COLUMN "public"."sys_message"."sender_name" IS '发送者名称 - 消息发送者显示名称';
COMMENT ON COLUMN "public"."sys_message"."priority" IS '优先级: 0-普通, 1-重要, 2-紧急';
COMMENT ON COLUMN "public"."sys_message"."attachment_urls" IS '附件URLs - JSON格式的附件URL列表';
COMMENT ON COLUMN "public"."sys_message"."target_type" IS '发送目标类型: all-全部用户, dept-指定部门, role-指定角色, user-指定用户';
COMMENT ON COLUMN "public"."sys_message"."target_ids" IS '目标ID列表 - JSON格式的目标ID数组';
COMMENT ON COLUMN "public"."sys_message"."expire_time" IS '过期时间 - 消息过期时间，过期后不再显示';

-- 用户消息关联表
DROP TABLE IF EXISTS "public"."sys_message_user";
CREATE TABLE "public"."sys_message_user" (
  "id" int8 NOT NULL DEFAULT nextval('sys_message_user_id_seq'::regclass),
  "message_id" int8 NOT NULL,
  "user_id" int8 NOT NULL,
  "is_read" int2 NOT NULL DEFAULT 0,
  "read_time" timestamptz(6),
  "is_starred" int2 NOT NULL DEFAULT 0,
  "is_deleted" int2 NOT NULL DEFAULT 0,
  "is_archived" int2 NOT NULL DEFAULT 0,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_message_user" IS '用户消息关联表 - 消息的已读/星标状态，追踪用户对消息的操作';
COMMENT ON COLUMN "public"."sys_message_user"."message_id" IS '消息ID - 关联的消息ID';
COMMENT ON COLUMN "public"."sys_message_user"."user_id" IS '用户ID - 消息接收者ID';
COMMENT ON COLUMN "public"."sys_message_user"."is_read" IS '是否已读: 0-未读, 1-已读';
COMMENT ON COLUMN "public"."sys_message_user"."read_time" IS '阅读时间 - 用户阅读消息的时间';
COMMENT ON COLUMN "public"."sys_message_user"."is_starred" IS '是否星标: 0-否, 1-是';
COMMENT ON COLUMN "public"."sys_message_user"."is_deleted" IS '是否删除: 0-否, 1-是（软删除）';
COMMENT ON COLUMN "public"."sys_message_user"."is_archived" IS '是否归档: 0-否, 1-是';

-- 系统公告表
DROP TABLE IF EXISTS "public"."announcements";
CREATE TABLE "public"."announcements" (
  "id" int8 NOT NULL DEFAULT nextval('announcements_id_seq'::regclass),
  "title" varchar(200) COLLATE "pg_catalog"."default" NOT NULL,
  "content" text COLLATE "pg_catalog"."default" NOT NULL,
  "announcement_type" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'info'::character varying,
  "priority" int4 DEFAULT 0,
  "is_pinned" bool DEFAULT false,
  "is_active" bool DEFAULT true,
  "start_time" timestamptz(6),
  "end_time" timestamptz(6),
  "created_by" int8,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."announcements" IS '系统公告表 - 系统公告与通知，支持定时发布和置顶';
COMMENT ON COLUMN "public"."announcements"."title" IS '公告标题 - 公告的主题';
COMMENT ON COLUMN "public"."announcements"."content" IS '公告内容 - 公告正文内容';
COMMENT ON COLUMN "public"."announcements"."announcement_type" IS '公告类型: info-通知, warning-警告, success-成功, error-错误';
COMMENT ON COLUMN "public"."announcements"."priority" IS '优先级: 0-普通, 1-高';
COMMENT ON COLUMN "public"."announcements"."is_pinned" IS '是否置顶 - true-置顶显示在顶部';
COMMENT ON COLUMN "public"."announcements"."is_active" IS '是否激活 - true-激活显示, false-暂停显示';
COMMENT ON COLUMN "public"."announcements"."start_time" IS '开始时间 - 公告开始展示的时间';
COMMENT ON COLUMN "public"."announcements"."end_time" IS '结束时间 - 公告结束展示的时间';
COMMENT ON COLUMN "public"."announcements"."created_by" IS '创建者ID - 公告创建者的用户ID';

-- 消息模板表
DROP TABLE IF EXISTS "public"."message_templates";
CREATE TABLE "public"."message_templates" (
  "id" int8 NOT NULL DEFAULT nextval('message_templates_id_seq'::regclass),
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "template_type" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "title_template" varchar(200) COLLATE "pg_catalog"."default" NOT NULL,
  "content_template" text COLLATE "pg_catalog"."default" NOT NULL,
  "variables" jsonb DEFAULT '[]'::jsonb,
  "is_active" bool DEFAULT true,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."message_templates" IS '消息模板表 - 消息模板管理，支持变量替换的消息模板配置';
COMMENT ON COLUMN "public"."message_templates"."name" IS '模板名称 - 模板的显示名称';
COMMENT ON COLUMN "public"."message_templates"."template_type" IS '模板类型: sms-短信, email-邮件, push-推送, notification-站内通知';
COMMENT ON COLUMN "public"."message_templates"."title_template" IS '标题模板 - 消息标题模板，支持变量占位符如 ${username}';
COMMENT ON COLUMN "public"."message_templates"."content_template" IS '内容模板 - 消息内容模板，支持变量占位符';
COMMENT ON COLUMN "public"."message_templates"."variables" IS '变量列表 - JSON格式支持的变量定义';
COMMENT ON COLUMN "public"."message_templates"."is_active" IS '是否启用 - true-启用使用, false-禁用';

-- =============================================================================
-- Table structures: dictionary (数据字典)
-- =============================================================================

-- 字典类型表
DROP TABLE IF EXISTS "public"."dictionary_types";
CREATE SEQUENCE IF NOT EXISTS "public"."dictionary_types_id_seq"
  START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
CREATE TABLE "public"."dictionary_types" (
  "id" int8 NOT NULL DEFAULT nextval('dictionary_types_id_seq'::regclass),
  "code" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "sort" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
ALTER SEQUENCE "public"."dictionary_types_id_seq" OWNED BY "public"."dictionary_types"."id";
ALTER TABLE ONLY "public"."dictionary_types" ALTER COLUMN "id" SET DEFAULT nextval('dictionary_types_id_seq'::regclass);
ALTER TABLE ONLY "public"."dictionary_types" ADD CONSTRAINT "dictionary_types_pkey" PRIMARY KEY ("id");
ALTER TABLE ONLY "public"."dictionary_types" ADD CONSTRAINT "dictionary_types_code_key" UNIQUE ("code");
COMMENT ON TABLE "public"."dictionary_types" IS '字典类型表 - 数据字典类型定义';
COMMENT ON COLUMN "public"."dictionary_types"."code" IS '类型编码 - 唯一标识，如 gender, status';
COMMENT ON COLUMN "public"."dictionary_types"."name" IS '类型名称 - 显示名称，如 性别, 状态';
COMMENT ON COLUMN "public"."dictionary_types"."sort" IS '排序 - 数值越小越靠前';
COMMENT ON COLUMN "public"."dictionary_types"."status" IS '状态 - 1-启用, 0-禁用';

-- 字典项表
DROP TABLE IF EXISTS "public"."dictionary_items";
CREATE SEQUENCE IF NOT EXISTS "public"."dictionary_items_id_seq"
  START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
CREATE TABLE "public"."dictionary_items" (
  "id" int8 NOT NULL DEFAULT nextval('dictionary_items_id_seq'::regclass),
  "type_id" int8 NOT NULL,
  "label" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "value" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "sort" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "is_default" bool DEFAULT false,
  "remark" text COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
ALTER SEQUENCE "public"."dictionary_items_id_seq" OWNED BY "public"."dictionary_items"."id";
ALTER TABLE ONLY "public"."dictionary_items" ALTER COLUMN "id" SET DEFAULT nextval('dictionary_items_id_seq'::regclass);
ALTER TABLE ONLY "public"."dictionary_items" ADD CONSTRAINT "dictionary_items_pkey" PRIMARY KEY ("id");
ALTER TABLE ONLY "public"."dictionary_items" ADD CONSTRAINT "dictionary_items_type_id_fkey" FOREIGN KEY ("type_id") REFERENCES "public"."dictionary_types"("id") ON DELETE CASCADE;
COMMENT ON TABLE "public"."dictionary_items" IS '字典项表 - 数据字典的具体选项';
COMMENT ON COLUMN "public"."dictionary_items"."label" IS '显示标签 - 如 男, 女';
COMMENT ON COLUMN "public"."dictionary_items"."value" IS '存储值 - 如 1, 2';
COMMENT ON COLUMN "public"."dictionary_items"."is_default" IS '是否默认值';
COMMENT ON COLUMN "public"."dictionary_items"."remark" IS '备注';

-- =============================================================================
-- Table structures: file-service (文件管理服务)
-- =============================================================================
CREATE TABLE "public"."sys_files" (
  "id" int8 NOT NULL DEFAULT nextval('sys_files_id_seq'::regclass),
  "file_name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "original_name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "file_size" int8 NOT NULL,
  "mime_type" varchar(100) COLLATE "pg_catalog"."default",
  "md5" varchar(32) COLLATE "pg_catalog"."default",
  "storage_path" varchar(500) COLLATE "pg_catalog"."default" NOT NULL,
  "storage_type" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'local'::character varying,
  "bucket" varchar(100) COLLATE "pg_catalog"."default",
  "url" varchar(500) COLLATE "pg_catalog"."default",
  "category" varchar(50) COLLATE "pg_catalog"."default" DEFAULT 'general'::character varying,
  "created_by" int8,
  "tenant_id" int8,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
);
COMMENT ON TABLE "public"."sys_files" IS '系统文件表 - 文件存储管理，记录所有上传文件元数据';
COMMENT ON COLUMN "public"."sys_files"."file_name" IS '存储文件名 - 服务器存储的文件名（唯一）';
COMMENT ON COLUMN "public"."sys_files"."original_name" IS '原始文件名 - 用户上传时的原始文件名';
COMMENT ON COLUMN "public"."sys_files"."file_size" IS '文件大小 - 文件字节数';
COMMENT ON COLUMN "public"."sys_files"."mime_type" IS 'MIME类型 - 文件的MIME类型';
COMMENT ON COLUMN "public"."sys_files"."md5" IS 'MD5哈希 - 文件内容的MD5校验值';
COMMENT ON COLUMN "public"."sys_files"."storage_path" IS '存储路径 - 文件在存储系统中的路径';
COMMENT ON COLUMN "public"."sys_files"."storage_type" IS '存储类型: local-本地存储, oss-阿里云OSS, s3-AWS S3, minio-MinIO';
COMMENT ON COLUMN "public"."sys_files"."bucket" IS '存储桶 - 云存储的桶名';
COMMENT ON COLUMN "public"."sys_files"."url" IS '访问URL - 文件的访问URL';
COMMENT ON COLUMN "public"."sys_files"."category" IS '文件分类: general-通用, image-图片, document-文档, video-视频, audio-音频, avatar-头像, attachment-附件';
COMMENT ON COLUMN "public"."sys_files"."created_by" IS '上传者ID - 文件上传者的用户ID';
COMMENT ON COLUMN "public"."sys_files"."tenant_id" IS '租户ID - 文件所属的租户ID';

-- =============================================================================
-- Table structures: feedback-service (意见反馈服务)
-- =============================================================================

-- 意见反馈表
DROP TABLE IF EXISTS "public"."sys_feedback";
CREATE TABLE "public"."sys_feedback" (
  "id" int8 NOT NULL DEFAULT nextval('sys_feedback_id_seq'::regclass),
  "user_id" int8 NOT NULL,
  "user_name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "type" varchar(30) COLLATE "pg_catalog"."default" NOT NULL,
  "title" varchar(200) COLLATE "pg_catalog"."default" NOT NULL,
  "content" text COLLATE "pg_catalog"."default" NOT NULL,
  "attachments" jsonb,
  "contact" varchar(100) COLLATE "pg_catalog"."default",
  "status" varchar(20) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'pending'::character varying,
  "handler_id" int8,
  "handler_name" varchar(100) COLLATE "pg_catalog"."default",
  "handler_reply" text COLLATE "pg_catalog"."default",
  "handler_time" timestamptz(6),
  "rating" int2,
  "handler_attachments" jsonb,
  "client_info" jsonb,
  "page_url" varchar(500) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
);
COMMENT ON TABLE "public"."sys_feedback" IS '意见反馈表 - 用户反馈与投诉管理，支持处理流程和满意度评价';
COMMENT ON COLUMN "public"."sys_feedback"."user_id" IS '用户ID - 提交反馈的用户ID';
COMMENT ON COLUMN "public"."sys_feedback"."user_name" IS '用户名 - 提交反馈的用户显示名';
COMMENT ON COLUMN "public"."sys_feedback"."type" IS '反馈类型: suggestion-建议, bug-缺陷, complaint-投诉, other-其他';
COMMENT ON COLUMN "public"."sys_feedback"."title" IS '反馈标题 - 反馈的主题';
COMMENT ON COLUMN "public"."sys_feedback"."content" IS '反馈内容 - 反馈的详细描述';
COMMENT ON COLUMN "public"."sys_feedback"."attachments" IS '附件 - JSON格式的用户上传附件列表';
COMMENT ON COLUMN "public"."sys_feedback"."contact" IS '联系方式 - 用户提供的联系方式';
COMMENT ON COLUMN "public"."sys_feedback"."status" IS '处理状态: pending-待处理, processing-处理中, resolved-已解决, rejected-已拒绝, closed-已关闭';
COMMENT ON COLUMN "public"."sys_feedback"."handler_id" IS '处理者ID - 处理反馈的管理员ID';
COMMENT ON COLUMN "public"."sys_feedback"."handler_name" IS '处理者名称 - 处理反馈的管理员名称';
COMMENT ON COLUMN "public"."sys_feedback"."handler_reply" IS '处理回复 - 管理员的回复内容';
COMMENT ON COLUMN "public"."sys_feedback"."handler_time" IS '处理时间 - 管理员回复的时间';
COMMENT ON COLUMN "public"."sys_feedback"."rating" IS '满意度评分: 1-非常不满意, 2-不满意, 3-一般, 4-满意, 5-非常满意';
COMMENT ON COLUMN "public"."sys_feedback"."handler_attachments" IS '处理附件 - JSON格式的处理相关附件';
COMMENT ON COLUMN "public"."sys_feedback"."client_info" IS '客户端信息 - JSON格式的浏览器和设备信息';
COMMENT ON COLUMN "public"."sys_feedback"."page_url" IS '反馈页面URL - 用户提交反馈时的页面URL';

-- =============================================================================
-- Table structures: workflow-service (工作流服务)
-- =============================================================================

-- 工作流定义表
DROP TABLE IF EXISTS "public"."workflows";
CREATE TABLE "public"."workflows" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "definition" jsonb NOT NULL DEFAULT '{}'::jsonb,
  "status" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'draft'::character varying,
  "version" int4 NOT NULL DEFAULT 1,
  "created_by" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."workflows" IS '工作流定义表 - 工作流的流程定义与配置，支持版本管理';
COMMENT ON COLUMN "public"."workflows"."name" IS '工作流名称 - 工作流的显示名称';
COMMENT ON COLUMN "public"."workflows"."description" IS '工作流描述 - 工作流的详细说明';
COMMENT ON COLUMN "public"."workflows"."definition" IS '流程定义 - JSON格式的流程图定义';
COMMENT ON COLUMN "public"."workflows"."status" IS '状态: draft-草稿, active-启用, inactive-停用';
COMMENT ON COLUMN "public"."workflows"."version" IS '版本号 - 工作流版本，用于版本控制';
COMMENT ON COLUMN "public"."workflows"."created_by" IS '创建者ID - 工作流创建者的用户ID';

-- 工作流节点表
DROP TABLE IF EXISTS "public"."workflow_nodes";
CREATE TABLE "public"."workflow_nodes" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "workflow_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "node_type" varchar(32) COLLATE "pg_catalog"."default" NOT NULL,
  "position_x" int4 NOT NULL DEFAULT 0,
  "position_y" int4 NOT NULL DEFAULT 0,
  "config" jsonb DEFAULT '{}'::jsonb,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "timeout" int4,
  "auto_complete" bool DEFAULT false
);
COMMENT ON TABLE "public"."workflow_nodes" IS '工作流节点表 - 工作流中的节点定义，包括任务节点和网关';
COMMENT ON COLUMN "public"."workflow_nodes"."workflow_id" IS '工作流ID - 所属的工作流ID';
COMMENT ON COLUMN "public"."workflow_nodes"."name" IS '节点名称 - 节点的显示名称';
COMMENT ON COLUMN "public"."workflow_nodes"."node_type" IS '节点类型: start-开始节点, end-结束节点, task-任务节点, gateway-网关节点, approve-审批节点';
COMMENT ON COLUMN "public"."workflow_nodes"."position_x" IS 'X坐标 - 流程图中节点的X坐标';
COMMENT ON COLUMN "public"."workflow_nodes"."position_y" IS 'Y坐标 - 流程图中节点的Y坐标';
COMMENT ON COLUMN "public"."workflow_nodes"."config" IS '节点配置 - JSON格式的节点详细配置';
COMMENT ON COLUMN "public"."workflow_nodes"."timeout" IS '超时时间 - 节点的超时时间（分钟），0表示不超时';
COMMENT ON COLUMN "public"."workflow_nodes"."auto_complete" IS '是否自动完成 - true-满足条件自动完成';

-- 工作流连线表
DROP TABLE IF EXISTS "public"."workflow_edges";
CREATE TABLE "public"."workflow_edges" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "workflow_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "source_node_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "target_node_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "edge_type" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'default'::character varying,
  "condition_expression" text COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "label" varchar(255) COLLATE "pg_catalog"."default",
  "priority" int4 DEFAULT 0,
  "condition" varchar(500) COLLATE "pg_catalog"."default"
);
COMMENT ON TABLE "public"."workflow_edges" IS '工作流连线表 - 工作流中节点之间的连线与条件，控制流程走向';
COMMENT ON COLUMN "public"."workflow_edges"."workflow_id" IS '工作流ID - 所属的工作流ID';
COMMENT ON COLUMN "public"."workflow_edges"."source_node_id" IS '源节点ID - 连线的起始节点ID';
COMMENT ON COLUMN "public"."workflow_edges"."target_node_id" IS '目标节点ID - 连线的结束节点ID';
COMMENT ON COLUMN "public"."workflow_edges"."edge_type" IS '连线类型: default-默认连线, conditional-条件连线, fallback-备选连线';
COMMENT ON COLUMN "public"."workflow_edges"."condition_expression" IS '条件表达式 - DSL格式的流转条件';
COMMENT ON COLUMN "public"."workflow_edges"."label" IS '连线标签 - 显示在流程图上的标签';
COMMENT ON COLUMN "public"."workflow_edges"."priority" IS '优先级 - 条件判断优先级，数字越大越优先';

-- 工作流实例表
DROP TABLE IF EXISTS "public"."workflow_instances";
CREATE TABLE "public"."workflow_instances" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "workflow_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "workflow_version" int4 NOT NULL DEFAULT 1,
  "status" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'pending'::character varying,
  "current_node_id" varchar(64) COLLATE "pg_catalog"."default",
  "variables" jsonb DEFAULT '{}'::jsonb,
  "started_by" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "started_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "completed_at" timestamptz(6)
);
COMMENT ON TABLE "public"."workflow_instances" IS '工作流实例表 - 工作流运行实例，记录每个工作流的执行状态';
COMMENT ON COLUMN "public"."workflow_instances"."workflow_id" IS '工作流定义ID - 关联的工作流定义ID';
COMMENT ON COLUMN "public"."workflow_instances"."workflow_version" IS '工作流版本 - 实例创建时的工作流版本号';
COMMENT ON COLUMN "public"."workflow_instances"."status" IS '状态: pending-待执行, running-执行中, suspended-暂停, completed-已完成, cancelled-已取消';
COMMENT ON COLUMN "public"."workflow_instances"."current_node_id" IS '当前节点ID - 流程当前停留的节点';
COMMENT ON COLUMN "public"."workflow_instances"."variables" IS '流程变量 - JSON格式的运行时数据变量';
COMMENT ON COLUMN "public"."workflow_instances"."started_by" IS '启动者ID - 启动工作流实例的用户ID';

-- 任务记录表
DROP TABLE IF EXISTS "public"."task_records";
CREATE TABLE "public"."task_records" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "instance_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "node_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "node_name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "assignee" varchar(64) COLLATE "pg_catalog"."default",
  "status" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'pending'::character varying,
  "comment" text COLLATE "pg_catalog"."default",
  "started_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "completed_at" timestamptz(6),
  "result" jsonb,
  "form_data" jsonb,
  "timeout_at" timestamptz(6)
);
COMMENT ON TABLE "public"."task_records" IS '任务记录表 - 工作流中任务的执行记录，跟踪每个任务的处理状态';
COMMENT ON COLUMN "public"."task_records"."instance_id" IS '实例ID - 所属的工作流实例ID';
COMMENT ON COLUMN "public"."task_records"."node_id" IS '节点ID - 对应的节点定义ID';
COMMENT ON COLUMN "public"."task_records"."node_name" IS '节点名称 - 任务的显示名称';
COMMENT ON COLUMN "public"."task_records"."assignee" IS '办理人ID - 任务分配的用户ID';
COMMENT ON COLUMN "public"."task_records"."status" IS '状态: pending-待领取, active-执行中, completed-已完成, cancelled-已取消';
COMMENT ON COLUMN "public"."task_records"."comment" IS '处理意见 - 办理人对任务的处理意见';
COMMENT ON COLUMN "public"."task_records"."result" IS '处理结果 - JSON格式的处理结果数据';
COMMENT ON COLUMN "public"."task_records"."form_data" IS '表单数据 - JSON格式的用户提交表单数据';
COMMENT ON COLUMN "public"."task_records"."timeout_at" IS '超时时间 - 任务需要完成的时间点';

-- =============================================================================
-- Table structures: audit-service (审计服务)
-- =============================================================================

-- 系统操作日志表
DROP TABLE IF EXISTS "public"."sys_operation_logs";
CREATE TABLE "public"."sys_operation_logs" (
  "id" int8 NOT NULL DEFAULT nextval('sys_operation_logs_id_seq'::regclass),
  "user_id" int8,
  "username" varchar(100) COLLATE "pg_catalog"."default",
  "module" varchar(50) COLLATE "pg_catalog"."default",
  "business_type" varchar(30) COLLATE "pg_catalog"."default",
  "method" varchar(100) COLLATE "pg_catalog"."default",
  "request_method" varchar(10) COLLATE "pg_catalog"."default",
  "request_url" varchar(500) COLLATE "pg_catalog"."default",
  "request_params" text COLLATE "pg_catalog"."default",
  "request_body" text COLLATE "pg_catalog"."default",
  "response_data" text COLLATE "pg_catalog"."default",
  "status" int2,
  "error_msg" text COLLATE "pg_catalog"."default",
  "execution_time" int4,
  "ip_address" varchar(50) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_operation_logs" IS '系统操作日志表 - 用户操作审计，记录所有业务操作便于追溯和审计';
COMMENT ON COLUMN "public"."sys_operation_logs"."user_id" IS '用户ID - 执行操作的用户ID';
COMMENT ON COLUMN "public"."sys_operation_logs"."username" IS '用户名 - 执行操作的用户名';
COMMENT ON COLUMN "public"."sys_operation_logs"."module" IS '模块名称 - 操作所属的模块';
COMMENT ON COLUMN "public"."sys_operation_logs"."business_type" IS '业务类型: create-新增, update-修改, delete-删除, query-查询';
COMMENT ON COLUMN "public"."sys_operation_logs"."method" IS '操作方法 - 具体操作的函数或方法名';
COMMENT ON COLUMN "public"."sys_operation_logs"."request_method" IS 'HTTP方法 - GET/POST/PUT/DELETE';
COMMENT ON COLUMN "public"."sys_operation_logs"."request_url" IS '请求URL - 操作的请求路径';
COMMENT ON COLUMN "public"."sys_operation_logs"."request_params" IS '请求参数 - URL查询参数';
COMMENT ON COLUMN "public"."sys_operation_logs"."request_body" IS '请求体 - POST/PUT请求的body内容';
COMMENT ON COLUMN "public"."sys_operation_logs"."response_data" IS '响应数据 - 操作返回的数据';
COMMENT ON COLUMN "public"."sys_operation_logs"."status" IS '状态: 1-成功, 0-失败';
COMMENT ON COLUMN "public"."sys_operation_logs"."error_msg" IS '错误信息 - 操作失败时的错误详情';
COMMENT ON COLUMN "public"."sys_operation_logs"."execution_time" IS '执行时长 - 操作执行的时间（毫秒）';
COMMENT ON COLUMN "public"."sys_operation_logs"."ip_address" IS 'IP地址 - 操作者的IP地址';

-- 敏感操作审计表
DROP TABLE IF EXISTS "public"."sys_sensitive_audits";
CREATE TABLE "public"."sys_sensitive_audits" (
  "id" int8 NOT NULL DEFAULT nextval('sys_sensitive_audits_id_seq'::regclass),
  "user_id" int8,
  "username" varchar(100) COLLATE "pg_catalog"."default",
  "operation_type" varchar(50) COLLATE "pg_catalog"."default",
  "operation_desc" varchar(255) COLLATE "pg_catalog"."default",
  "request_data" text COLLATE "pg_catalog"."default",
  "ip_address" varchar(50) COLLATE "pg_catalog"."default",
  "confirm_status" int2 DEFAULT 1,
  "confirm_time" timestamptz(6),
  "confirmed_by" int8,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_sensitive_audits" IS '敏感操作审计表 - 敏感操作的二次确认记录，确保安全操作';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."user_id" IS '用户ID - 执行敏感操作用户ID';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."username" IS '用户名 - 执行敏感操作用户名';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."operation_type" IS '操作类型 - 敏感操作的类型';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."operation_desc" IS '操作描述 - 敏感操作的描述';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."request_data" IS '请求数据 - 操作的请求详情';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."ip_address" IS 'IP地址 - 操作者的IP地址';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."confirm_status" IS '确认状态: 1-待确认, 2-已确认, 3-已取消';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."confirm_time" IS '确认时间 - 操作被确认的时间';
COMMENT ON COLUMN "public"."sys_sensitive_audits"."confirmed_by" IS '确认人ID - 确认操作的管理员ID';

-- IP白名单表
DROP TABLE IF EXISTS "public"."sys_ip_whitelist";
CREATE TABLE "public"."sys_ip_whitelist" (
  "id" int8 NOT NULL DEFAULT nextval('sys_ip_whitelist_id_seq'::regclass),
  "ip_address" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "ip_type" varchar(20) COLLATE "pg_catalog"."default",
  "start_ip" varchar(50) COLLATE "pg_catalog"."default",
  "end_ip" varchar(50) COLLATE "pg_catalog"."default",
  "description" varchar(255) COLLATE "pg_catalog"."default",
  "is_enabled" int2 DEFAULT 1,
  "created_by" int8,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_ip_whitelist" IS 'IP白名单表 - IP访问控制，允许特定IP绕过限制访问系统';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."ip_address" IS 'IP地址 - 白名单中的单个IP地址';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."ip_type" IS 'IP类型: single-单个IP, range-IP段';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."start_ip" IS '起始IP - IP段的起始地址';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."end_ip" IS '结束IP - IP段的结束地址';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."description" IS '描述说明 - 白名单规则的说明';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."is_enabled" IS '是否启用: 1-启用, 0-禁用';
COMMENT ON COLUMN "public"."sys_ip_whitelist"."created_by" IS '创建者ID - 规则创建者的用户ID';

-- =============================================================================
-- Table structures: clean-service (清理/报表服务)
-- =============================================================================

-- 报表定义表
DROP TABLE IF EXISTS "public"."reports";
CREATE TABLE "public"."reports" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "report_type" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'table'::character varying,
  "config" jsonb NOT NULL DEFAULT '{}'::jsonb,
  "schedule" jsonb,
  "status" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'draft'::character varying,
  "created_by" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."reports" IS '报表定义表 - 数据报表配置，定义报表的结构和数据源';
COMMENT ON COLUMN "public"."reports"."name" IS '报表名称 - 报表的显示名称';
COMMENT ON COLUMN "public"."reports"."description" IS '报表描述 - 报表的详细说明';
COMMENT ON COLUMN "public"."reports"."report_type" IS '报表类型: table-表格, chart-图表, dashboard-仪表盘';
COMMENT ON COLUMN "public"."reports"."config" IS '报表配置 - JSON格式的报表结构配置';
COMMENT ON COLUMN "public"."reports"."schedule" IS '调度配置 - JSON格式的定时调度配置';
COMMENT ON COLUMN "public"."reports"."status" IS '状态: draft-草稿, active-启用, inactive-停用';
COMMENT ON COLUMN "public"."reports"."created_by" IS '创建者ID - 报表创建者的用户ID';

-- 报表任务表
DROP TABLE IF EXISTS "public"."report_tasks";
CREATE TABLE "public"."report_tasks" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "report_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "status" varchar(32) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'pending'::character varying,
  "result" jsonb,
  "error_message" text COLLATE "pg_catalog"."default",
  "started_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "completed_at" timestamptz(6)
);
COMMENT ON TABLE "public"."report_tasks" IS '报表任务表 - 报表生成的执行记录，跟踪报表生成任务的状态';
COMMENT ON COLUMN "public"."report_tasks"."report_id" IS '报表ID - 关联的报表定义ID';
COMMENT ON COLUMN "public"."report_tasks"."status" IS '状态: pending-待执行, running-执行中, completed-已完成, failed-失败';
COMMENT ON COLUMN "public"."report_tasks"."result" IS '执行结果 - JSON格式的报表生成结果';
COMMENT ON COLUMN "public"."report_tasks"."error_message" IS '错误信息 - 任务失败时的错误详情';

-- 调度任务表
DROP TABLE IF EXISTS "public"."schedule_tasks";
CREATE TABLE "public"."schedule_tasks" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "task_type" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'once'::character varying,
  "cron_expression" varchar(100) COLLATE "pg_catalog"."default",
  "interval_seconds" int8,
  "start_time" timestamptz(6),
  "end_time" timestamptz(6),
  "task_handler" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "task_params" jsonb,
  "status" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'pending'::character varying,
  "execute_strategy" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'immediate'::character varying,
  "max_retries" int4 NOT NULL DEFAULT 3,
  "retry_count" int4 NOT NULL DEFAULT 0,
  "timeout_seconds" int8 DEFAULT 3600,
  "last_run_time" timestamptz(6),
  "next_run_time" timestamptz(6),
  "created_by" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "tenant_id" varchar(36) COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."schedule_tasks" IS '调度任务表 - 定时任务配置与调度，支持Cron表达式和间隔执行';
COMMENT ON COLUMN "public"."schedule_tasks"."name" IS '任务名称 - 定时任务的显示名称';
COMMENT ON COLUMN "public"."schedule_tasks"."description" IS '任务描述 - 定时任务的详细说明';
COMMENT ON COLUMN "public"."schedule_tasks"."task_type" IS '任务类型: once-一次性任务, cron-Cron表达式调度, interval-间隔执行';
COMMENT ON COLUMN "public"."schedule_tasks"."cron_expression" IS 'Cron表达式 - 6位或7位Cron表达式';
COMMENT ON COLUMN "public"."schedule_tasks"."interval_seconds" IS '间隔秒数 - 间隔执行时的间隔时间（秒）';
COMMENT ON COLUMN "public"."schedule_tasks"."start_time" IS '开始时间 - 调度开始的时间';
COMMENT ON COLUMN "public"."schedule_tasks"."end_time" IS '结束时间 - 调度结束的时间，NULL表示永不过期';
COMMENT ON COLUMN "public"."schedule_tasks"."task_handler" IS '任务处理器 - 任务执行的处理函数或类';
COMMENT ON COLUMN "public"."schedule_tasks"."task_params" IS '任务参数 - JSON格式的任务执行参数';
COMMENT ON COLUMN "public"."schedule_tasks"."status" IS '状态: pending-待执行, running-执行中, completed-已完成, failed-失败, cancelled-已取消';
COMMENT ON COLUMN "public"."schedule_tasks"."execute_strategy" IS '执行策略: immediate-立即执行, queued-队列执行';
COMMENT ON COLUMN "public"."schedule_tasks"."max_retries" IS '最大重试次数 - 任务失败后的最大重试次数';
COMMENT ON COLUMN "public"."schedule_tasks"."retry_count" IS '已重试次数 - 当前已重试的次数';
COMMENT ON COLUMN "public"."schedule_tasks"."timeout_seconds" IS '超时时间 - 任务执行超时时间（秒）';
COMMENT ON COLUMN "public"."schedule_tasks"."last_run_time" IS '上次运行时间 - 最后一次执行的时间';
COMMENT ON COLUMN "public"."schedule_tasks"."next_run_time" IS '下次运行时间 - 下一次计划执行的时间';
COMMENT ON COLUMN "public"."schedule_tasks"."created_by" IS '创建者ID - 任务创建者的用户ID';
COMMENT ON COLUMN "public"."schedule_tasks"."tenant_id" IS '租户ID - 任务所属的租户';

-- 任务执行记录表
DROP TABLE IF EXISTS "public"."task_executions";
CREATE TABLE "public"."task_executions" (
  "id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "task_id" varchar(36) COLLATE "pg_catalog"."default" NOT NULL,
  "status" varchar(50) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'running'::character varying,
  "start_time" timestamptz(6) NOT NULL,
  "end_time" timestamptz(6),
  "result" text COLLATE "pg_catalog"."default",
  "error_message" text COLLATE "pg_catalog"."default",
  "retry_count" int4 NOT NULL DEFAULT 0,
  "created_at" timestamptz(6) NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."task_executions" IS '任务执行记录表 - 调度任务的执行历史，记录每次执行的结果';
COMMENT ON COLUMN "public"."task_executions"."task_id" IS '任务ID - 关联的调度任务ID';
COMMENT ON COLUMN "public"."task_executions"."status" IS '执行状态: running-执行中, completed-已完成, failed-失败';
COMMENT ON COLUMN "public"."task_executions"."start_time" IS '开始时间 - 执行开始的时间点';
COMMENT ON COLUMN "public"."task_executions"."end_time" IS '结束时间 - 执行结束的时间点';
COMMENT ON COLUMN "public"."task_executions"."result" IS '执行结果 - 任务执行的成功结果';
COMMENT ON COLUMN "public"."task_executions"."error_message" IS '错误信息 - 任务失败的错误详情';
COMMENT ON COLUMN "public"."task_executions"."retry_count" IS '重试次数 - 本次执行的已重试次数';

-- 数据源配置表
DROP TABLE IF EXISTS "public"."data_sources";
CREATE TABLE "public"."data_sources" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "ds_type" varchar(32) COLLATE "pg_catalog"."default" NOT NULL,
  "config" jsonb NOT NULL DEFAULT '{}'::jsonb,
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."data_sources" IS '数据源配置表 - 报表和工作流的数据源配置，支持多种数据库类型';
COMMENT ON COLUMN "public"."data_sources"."name" IS '数据源名称 - 数据源的显示名称';
COMMENT ON COLUMN "public"."data_sources"."ds_type" IS '数据源类型: postgresql-PostgreSQL, mysql-MySQL, mongodb-MongoDB, rest_api-REST API, file-文件数据源';
COMMENT ON COLUMN "public"."data_sources"."config" IS '连接配置 - JSON格式的数据库连接配置（含加密凭证）';

-- =============================================================================
-- Table structures: api-key-service (API密钥服务)
-- =============================================================================

-- API密钥表
DROP TABLE IF EXISTS "public"."api_keys";
CREATE TABLE "public"."api_keys" (
  "id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "name" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default",
  "key_id" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "secret_key_hash" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "key_hint" varchar(20) COLLATE "pg_catalog"."default" NOT NULL,
  "permission_level" int4 DEFAULT 1,
  "allowed_ips" jsonb DEFAULT '[]'::jsonb,
  "rate_limit" int8 DEFAULT 1000,
  "tenant_id" int8,
  "user_id" int8,
  "status" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'active'::character varying,
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "expires_at" timestamptz(6),
  "last_used_at" timestamptz(6),
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."api_keys" IS 'API密钥表 - API访问密钥管理，支持密钥的创建、续期和禁用';
COMMENT ON COLUMN "public"."api_keys"."name" IS '密钥名称 - API密钥的显示名称';
COMMENT ON COLUMN "public"."api_keys"."description" IS '密钥描述 - API密钥的使用说明';
COMMENT ON COLUMN "public"."api_keys"."key_id" IS '密钥ID - API密钥的唯一标识（公钥）';
COMMENT ON COLUMN "public"."api_keys"."secret_key_hash" IS '密钥哈希 - Secret Key的哈希值，用于验证';
COMMENT ON COLUMN "public"."api_keys"."key_hint" IS '密钥提示 - Secret Key的最后4位，用于识别密钥';
COMMENT ON COLUMN "public"."api_keys"."permission_level" IS '权限级别: 1-只读, 2-读写, 3-管理员';
COMMENT ON COLUMN "public"."api_keys"."allowed_ips" IS '允许的IP - JSON格式的IP白名单';
COMMENT ON COLUMN "public"."api_keys"."rate_limit" IS '速率限制 - 每分钟允许的最大请求次数';
COMMENT ON COLUMN "public"."api_keys"."tenant_id" IS '租户ID - 密钥所属的租户';
COMMENT ON COLUMN "public"."api_keys"."user_id" IS '用户ID - 密钥所属的用户';
COMMENT ON COLUMN "public"."api_keys"."status" IS '状态: active-激活, inactive-停用, expired-已过期';
COMMENT ON COLUMN "public"."api_keys"."expires_at" IS '过期时间 - 密钥的失效时间';
COMMENT ON COLUMN "public"."api_keys"."last_used_at" IS '最后使用时间 - 密钥最近一次使用的时间';

-- API密钥使用记录表
DROP TABLE IF EXISTS "public"."api_key_usage_logs";
CREATE TABLE "public"."api_key_usage_logs" (
  "id" int8 NOT NULL DEFAULT nextval('api_key_usage_logs_id_seq'::regclass),
  "key_id" varchar(64) COLLATE "pg_catalog"."default" NOT NULL,
  "endpoint" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "method" varchar(10) COLLATE "pg_catalog"."default" NOT NULL,
  "status_code" int4,
  "latency_ms" int4,
  "ip_address" varchar(50) COLLATE "pg_catalog"."default",
  "user_agent" varchar(500) COLLATE "pg_catalog"."default",
  "request_size" int8,
  "response_size" int8,
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."api_key_usage_logs" IS 'API密钥使用记录表 - API调用审计与监控，记录每次API调用';
COMMENT ON COLUMN "public"."api_key_usage_logs"."key_id" IS '密钥ID - 使用的API密钥ID';
COMMENT ON COLUMN "public"."api_key_usage_logs"."endpoint" IS '端点 - 调用的API端点路径';
COMMENT ON COLUMN "public"."api_key_usage_logs"."method" IS 'HTTP方法 - GET/POST/PUT/DELETE';
COMMENT ON COLUMN "public"."api_key_usage_logs"."status_code" IS '状态码 - HTTP响应状态码';
COMMENT ON COLUMN "public"."api_key_usage_logs"."latency_ms" IS '延迟 - 请求处理时间（毫秒）';
COMMENT ON COLUMN "public"."api_key_usage_logs"."ip_address" IS 'IP地址 - 调用者的IP地址';
COMMENT ON COLUMN "public"."api_key_usage_logs"."user_agent" IS 'User-Agent - 调用者的客户端标识';
COMMENT ON COLUMN "public"."api_key_usage_logs"."request_size" IS '请求大小 - 请求体的字节数';
COMMENT ON COLUMN "public"."api_key_usage_logs"."response_size" IS '响应大小 - 响应体的字节数';

-- =============================================================================
-- Table structures: tow-service (拖车服务)
-- =============================================================================

-- 拖车车辆表
DROP TABLE IF EXISTS "public"."tow_car";
CREATE TABLE "public"."tow_car" (
  "id" int8 NOT NULL DEFAULT nextval('tow_car_id_seq'::regclass),
  "license" varchar(20) COLLATE "pg_catalog"."default",
  "vehicle" jsonb,
  "engine" varchar(50) COLLATE "pg_catalog"."default",
  "car_type" varchar(50) COLLATE "pg_catalog"."default",
  "dc_type" varchar(50) COLLATE "pg_catalog"."default",
  "dc_causes" varchar(200) COLLATE "pg_catalog"."default",
  "car_color" varchar(20) COLLATE "pg_catalog"."default",
  "dc_date" varchar(50) COLLATE "pg_catalog"."default",
  "dc_address" varchar(200) COLLATE "pg_catalog"."default",
  "dc_key" varchar(50) COLLATE "pg_catalog"."default",
  "dc_party_name" varchar(100) COLLATE "pg_catalog"."default",
  "dc_party_cardid" varchar(20) COLLATE "pg_catalog"."default",
  "dc_party_tel" varchar(20) COLLATE "pg_catalog"."default",
  "p_name" varchar(100) COLLATE "pg_catalog"."default",
  "p_id" varchar(20) COLLATE "pg_catalog"."default",
  "dc_acc" varchar(100) COLLATE "pg_catalog"."default",
  "dc_name" varchar(50) COLLATE "pg_catalog"."default",
  "dc_into_date" varchar(50) COLLATE "pg_catalog"."default",
  "car_remark" varchar(500) COLLATE "pg_catalog"."default",
  "driver" varchar(50) COLLATE "pg_catalog"."default",
  "operator" varchar(50) COLLATE "pg_catalog"."default",
  "drag_km" varchar(20) COLLATE "pg_catalog"."default",
  "drag_unit" varchar(50) COLLATE "pg_catalog"."default",
  "drag_money" varchar(20) COLLATE "pg_catalog"."default",
  "cmd_unit" varchar(100) COLLATE "pg_catalog"."default",
  "cmd_user" varchar(50) COLLATE "pg_catalog"."default",
  "cv" varchar(50) COLLATE "pg_catalog"."default",
  "cv_acc" varchar(100) COLLATE "pg_catalog"."default",
  "cv_name" varchar(50) COLLATE "pg_catalog"."default",
  "cv_date" varchar(50) COLLATE "pg_catalog"."default",
  "cv_opinion" varchar(200) COLLATE "pg_catalog"."default",
  "tv" varchar(50) COLLATE "pg_catalog"."default",
  "tv_acc" varchar(100) COLLATE "pg_catalog"."default",
  "tv_name" varchar(50) COLLATE "pg_catalog"."default",
  "tv_date" varchar(50) COLLATE "pg_catalog"."default",
  "tv_opinion" varchar(200) COLLATE "pg_catalog"."default",
  "rc_name" varchar(50) COLLATE "pg_catalog"."default",
  "rc_idcard" varchar(20) COLLATE "pg_catalog"."default",
  "rc_tel" varchar(20) COLLATE "pg_catalog"."default",
  "parking_date" varchar(50) COLLATE "pg_catalog"."default",
  "parking_unit" varchar(100) COLLATE "pg_catalog"."default",
  "parking_money" varchar(20) COLLATE "pg_catalog"."default",
  "parking_payable" varchar(20) COLLATE "pg_catalog"."default",
  "parking_paidin" varchar(20) COLLATE "pg_catalog"."default",
  "remark" varchar(500) COLLATE "pg_catalog"."default",
  "rs_name" varchar(50) COLLATE "pg_catalog"."default",
  "rs_acc" varchar(100) COLLATE "pg_catalog"."default",
  "rs_date" varchar(50) COLLATE "pg_catalog"."default",
  "attachment" jsonb,
  "create_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "create_user" varchar(50) COLLATE "pg_catalog"."default",
  "update_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "update_user" varchar(50) COLLATE "pg_catalog"."default",
  "delete" bool DEFAULT false
);
COMMENT ON TABLE "public"."tow_car" IS '拖车车辆表 - 违停车辆管理，记录违章车辆和拖车处理全流程';
COMMENT ON COLUMN "public"."tow_car"."license" IS '车牌号 - 车辆牌照号码';
COMMENT ON COLUMN "public"."tow_car"."vehicle" IS '车辆信息 - JSON格式的车辆详细信息（品牌、型号等）';
COMMENT ON COLUMN "public"."tow_car"."engine" IS '发动机号 - 车辆发动机编号';
COMMENT ON COLUMN "public"."tow_car"."car_type" IS '车辆类型 - 对应的车辆类型表ID';
COMMENT ON COLUMN "public"."tow_car"."dc_type" IS '扣押原因类型 - 对应的扣押原因类型';
COMMENT ON COLUMN "public"."tow_car"."dc_causes" IS '扣押原因明细 - 具体违章原因描述';
COMMENT ON COLUMN "public"."tow_car"."car_color" IS '车辆颜色 - 车辆车身颜色';
COMMENT ON COLUMN "public"."tow_car"."dc_date" IS '扣押日期 - 违章拖车日期';
COMMENT ON COLUMN "public"."tow_car"."dc_address" IS '扣押地址 - 违章发生地点';
COMMENT ON COLUMN "public"."tow_car"."dc_key" IS '放行条编号 - 取车时使用的放行条编号';
COMMENT ON COLUMN "public"."tow_car"."dc_party_name" IS '当事人姓名 - 违章当事人姓名';
COMMENT ON COLUMN "public"."tow_car"."dc_party_cardid" IS '当事人身份证 - 当事人身份证号码';
COMMENT ON COLUMN "public"."tow_car"."dc_party_tel" IS '当事人电话 - 当事人联系电话';
COMMENT ON COLUMN "public"."tow_car"."p_name" IS '车主姓名 - 车辆登记的车主姓名';
COMMENT ON COLUMN "public"."tow_car"."p_id" IS '车主身份证 - 车主身份证号码';
COMMENT ON COLUMN "public"."tow_car"."dc_acc" IS '扣押账户 - 执行扣押操作的管理员账户';
COMMENT ON COLUMN "public"."tow_car"."dc_name" IS '扣押操作人 - 执行扣押操作的管理员姓名';
COMMENT ON COLUMN "public"."tow_car"."dc_into_date" IS '入场日期 - 车辆入场停放日期';
COMMENT ON COLUMN "public"."tow_car"."car_remark" IS '车辆备注 - 车辆相关补充信息';
COMMENT ON COLUMN "public"."tow_car"."driver" IS '司机 - 拖车司机姓名';
COMMENT ON COLUMN "public"."tow_car"."operator" IS '操作员 - 系统操作员';
COMMENT ON COLUMN "public"."tow_car"."drag_km" IS '拖车里程 - 拖车行驶里程';
COMMENT ON COLUMN "public"."tow_car"."drag_unit" IS '拖车单位 - 拖车服务提供单位';
COMMENT ON COLUMN "public"."tow_car"."drag_money" IS '拖车费用 - 拖车服务费用';
COMMENT ON COLUMN "public"."tow_car"."cmd_unit" IS '执法单位 - 开具处罚决定的单位';
COMMENT ON COLUMN "public"."tow_car"."cmd_user" IS '执法员 - 执法人员的用户名';
COMMENT ON COLUMN "public"."tow_car"."cv" IS '交警签字 - 交警签字状态';
COMMENT ON COLUMN "public"."tow_car"."cv_acc" IS '交警账户 - 签字交警的账户';
COMMENT ON COLUMN "public"."tow_car"."cv_name" IS '交警姓名 - 签字交警的姓名';
COMMENT ON COLUMN "public"."tow_car"."cv_date" IS '交警签字日期 - 交警签字时间';
COMMENT ON COLUMN "public"."tow_car"."cv_opinion" IS '交警意见 - 交警的审核意见';
COMMENT ON COLUMN "public"."tow_car"."tv" IS '城管签字 - 城管签字状态';
COMMENT ON COLUMN "public"."tow_car"."tv_acc" IS '城管账户 - 签字城管的账户';
COMMENT ON COLUMN "public"."tow_car"."tv_name" IS '城管姓名 - 签字城管的姓名';
COMMENT ON COLUMN "public"."tow_car"."tv_date" IS '城管签字日期 - 城管签字时间';
COMMENT ON COLUMN "public"."tow_car"."tv_opinion" IS '城管意见 - 城管的审核意见';
COMMENT ON COLUMN "public"."tow_car"."rc_name" IS '取车人姓名 - 取车人的姓名';
COMMENT ON COLUMN "public"."tow_car"."rc_idcard" IS '取车人身份证 - 取车人身份证号';
COMMENT ON COLUMN "public"."tow_car"."rc_tel" IS '取车人电话 - 取车人联系电话';
COMMENT ON COLUMN "public"."tow_car"."parking_date" IS '停车开始日期 - 开始计费的日期';
COMMENT ON COLUMN "public"."tow_car"."parking_unit" IS '停车场 - 车辆停放的停车场';
COMMENT ON COLUMN "public"."tow_car"."parking_money" IS '停车费 - 产生的停车费用';
COMMENT ON COLUMN "public"."tow_car"."parking_payable" IS '应付停车费 - 应付但未付的停车费';
COMMENT ON COLUMN "public"."tow_car"."parking_paidin" IS '实付停车费 - 已支付的停车费';
COMMENT ON COLUMN "public"."tow_car"."remark" IS '备注 - 其他补充说明';
COMMENT ON COLUMN "public"."tow_car"."rs_name" IS '放行签字人 - 签字放行的工作人员姓名';
COMMENT ON COLUMN "public"."tow_car"."rs_acc" IS '放行账户 - 放行操作员的账户';
COMMENT ON COLUMN "public"."tow_car"."rs_date" IS '放行日期 - 车辆放行的时间';
COMMENT ON COLUMN "public"."tow_car"."attachment" IS '附件 - JSON格式的图片或文档附件';
COMMENT ON COLUMN "public"."tow_car"."create_user" IS '创建人 - 记录创建者';
COMMENT ON COLUMN "public"."tow_car"."update_user" IS '更新人 - 记录最后更新者';
COMMENT ON COLUMN "public"."tow_car"."delete" IS '软删除标记: true-已删除, false-正常';

-- 车辆分类表
DROP TABLE IF EXISTS "public"."tow_car_class";
CREATE TABLE "public"."tow_car_class" (
  "id" int8 NOT NULL DEFAULT nextval('tow_car_class_id_seq'::regclass),
  "cpt" varchar(50) COLLATE "pg_catalog"."default",
  "remark" varchar(200) COLLATE "pg_catalog"."default",
  "free_time" int8 DEFAULT 0,
  "hm10" varchar(20) COLLATE "pg_catalog"."default",
  "hm24" varchar(20) COLLATE "pg_catalog"."default",
  "gratis_day" int8 DEFAULT 0,
  "cost_day" varchar(20) COLLATE "pg_catalog"."default",
  "create_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "update_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."tow_car_class" IS '车辆分类表 - 收费标准配置，定义不同车辆类型的停车收费规则';
COMMENT ON COLUMN "public"."tow_car_class"."cpt" IS '车辆类型代码 - 对应的车辆类型编码';
COMMENT ON COLUMN "public"."tow_car_class"."remark" IS '分类说明 - 分类的补充描述';
COMMENT ON COLUMN "public"."tow_car_class"."free_time" IS '免费时长 - 车辆入场后的免费停车时长（分钟）';
COMMENT ON COLUMN "public"."tow_car_class"."hm10" IS '10小时收费 - 停车10小时内的收费标准';
COMMENT ON COLUMN "public"."tow_car_class"."hm24" IS '24小时收费 - 停车24小时的收费标准';
COMMENT ON COLUMN "public"."tow_car_class"."gratis_day" IS '免费天数 - 允许的免费停车天数';
COMMENT ON COLUMN "public"."tow_car_class"."cost_day" IS '每日收费 - 超过免费天数后的每日收费标准';

-- 车辆类型表
DROP TABLE IF EXISTS "public"."tow_car_type";
CREATE TABLE "public"."tow_car_type" (
  "id" int8 NOT NULL DEFAULT nextval('tow_car_type_id_seq'::regclass),
  "name" varchar(50) COLLATE "pg_catalog"."default",
  "remark" varchar(200) COLLATE "pg_catalog"."default",
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "create_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "update_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."tow_car_type" IS '车辆类型表 - 车辆分类管理，定义系统支持的车辆类型';
COMMENT ON COLUMN "public"."tow_car_type"."name" IS '类型名称 - 车辆类型的显示名称';
COMMENT ON COLUMN "public"."tow_car_type"."remark" IS '类型说明 - 类型的补充描述';
COMMENT ON COLUMN "public"."tow_car_type"."sort_order" IS '排序号 - 类型列表的排序';
COMMENT ON COLUMN "public"."tow_car_type"."status" IS '状态: 1-启用, 0-禁用';

-- 车辆颜色表
DROP TABLE IF EXISTS "public"."tow_car_color";
CREATE TABLE "public"."tow_car_color" (
  "id" int8 NOT NULL DEFAULT nextval('tow_car_color_id_seq'::regclass),
  "name" varchar(50) COLLATE "pg_catalog"."default",
  "remark" varchar(200) COLLATE "pg_catalog"."default",
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "create_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "update_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."tow_car_color" IS '车辆颜色表 - 车辆颜色管理，记录系统支持的车辆颜色';
COMMENT ON COLUMN "public"."tow_car_color"."name" IS '颜色名称 - 颜色的显示名称';
COMMENT ON COLUMN "public"."tow_car_color"."remark" IS '颜色说明 - 颜色的补充描述';
COMMENT ON COLUMN "public"."tow_car_color"."sort_order" IS '排序号 - 颜色列表的排序';
COMMENT ON COLUMN "public"."tow_car_color"."status" IS '状态: 1-启用, 0-禁用';

-- 扣押原因类型表
DROP TABLE IF EXISTS "public"."tow_dc_type";
CREATE TABLE "public"."tow_dc_type" (
  "id" int8 NOT NULL DEFAULT nextval('tow_dc_type_id_seq'::regclass),
  "name" varchar(50) COLLATE "pg_catalog"."default",
  "code" varchar(20) COLLATE "pg_catalog"."default",
  "remark" varchar(200) COLLATE "pg_catalog"."default",
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "create_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "update_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."tow_dc_type" IS '扣押原因类型表 - 违章原因分类，定义违章行为的分类';
COMMENT ON COLUMN "public"."tow_dc_type"."name" IS '类型名称 - 扣押原因类型的显示名称';
COMMENT ON COLUMN "public"."tow_dc_type"."code" IS '类型编码 - 扣押原因类型的唯一代码';
COMMENT ON COLUMN "public"."tow_dc_type"."remark" IS '类型说明 - 类型的补充描述';
COMMENT ON COLUMN "public"."tow_dc_type"."sort_order" IS '排序号 - 类型列表的排序';
COMMENT ON COLUMN "public"."tow_dc_type"."status" IS '状态: 1-启用, 0-禁用';

-- 扣押原因明细表
DROP TABLE IF EXISTS "public"."tow_dc_causes";
CREATE TABLE "public"."tow_dc_causes" (
  "id" int8 NOT NULL DEFAULT nextval('tow_dc_causes_id_seq'::regclass),
  "name" varchar(100) COLLATE "pg_catalog"."default",
  "type_id" int8,
  "type_name" varchar(50) COLLATE "pg_catalog"."default",
  "remark" varchar(200) COLLATE "pg_catalog"."default",
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "create_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "update_date" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."tow_dc_causes" IS '扣押原因明细表 - 具体违章原因，定义每种违章行为';
COMMENT ON COLUMN "public"."tow_dc_causes"."name" IS '原因名称 - 具体违章原因的名称';
COMMENT ON COLUMN "public"."tow_dc_causes"."type_id" IS '所属类型ID - 关联的扣押原因类型ID';
COMMENT ON COLUMN "public"."tow_dc_causes"."type_name" IS '所属类型名称 - 关联的扣押原因类型名称';
COMMENT ON COLUMN "public"."tow_dc_causes"."remark" IS '原因说明 - 原因的详细说明';
COMMENT ON COLUMN "public"."tow_dc_causes"."sort_order" IS '排序号 - 原因列表的排序';
COMMENT ON COLUMN "public"."tow_dc_causes"."status" IS '状态: 1-启用, 0-禁用';

-- =============================================================================
-- Table structures: ebike-service (电动自行车服务)
-- =============================================================================

-- 电动自行车用户表 (旧表保留用于数据迁移，新代码使用 public.users)
DROP TABLE IF EXISTS "public"."ebike_user";
CREATE TABLE "public"."ebike_user" (
  "username" varchar(255) COLLATE "pg_catalog"."default",
  "password" varchar(255) COLLATE "pg_catalog"."default",
  "create_date" timestamp(6),
  "update_date" timestamp(6),
  "delete" bool,
  "status" int8,
  "last_login" timestamp(6),
  "level" int8
);
COMMENT ON TABLE "public"."ebike_user" IS '电动自行车用户旧表 - 已迁移到 public.users，保留用于数据兼容';

-- 电动自行车用户兼容视图已废弃 (2026-08-04 审计修复):
-- 原视图暴露 password_hash AS password, 且全仓库无代码消费该视图, 故不再重建。
-- 需要兼容查询时请直接使用 public.users (含 ebike_user 迁移字段)。
DROP VIEW IF EXISTS "public"."user";

-- 电动自行车车辆表
DROP TABLE IF EXISTS "public"."ebike_car";
CREATE TABLE "public"."car" (
  "code" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "status" int8,
  "provide" varchar(255) COLLATE "pg_catalog"."default",
  "speed" float8,
  "gps" json,
  "create_date" timestamp(0),
  "update_date" timestamp(0),
  "delete" bool,
  "alert" varchar(255) COLLATE "pg_catalog"."default",
  "remark" varchar(255) COLLATE "pg_catalog"."default",
  "type" int8,
  "time" json
);
COMMENT ON TABLE "public"."car" IS '电动自行车车辆表';
COMMENT ON COLUMN "public"."car"."code" IS '车辆编码';
COMMENT ON COLUMN "public"."car"."status" IS '状态';
COMMENT ON COLUMN "public"."car"."provide" IS '供应商';
COMMENT ON COLUMN "public"."car"."speed" IS '速度';
COMMENT ON COLUMN "public"."car"."gps" IS 'GPS信息';
COMMENT ON COLUMN "public"."car"."create_date" IS '创建时间';
COMMENT ON COLUMN "public"."car"."update_date" IS '更新时间';
COMMENT ON COLUMN "public"."car"."delete" IS '软删除';
COMMENT ON COLUMN "public"."car"."alert" IS '告警';
COMMENT ON COLUMN "public"."car"."remark" IS '备注';
COMMENT ON COLUMN "public"."car"."type" IS '类型';
COMMENT ON COLUMN "public"."car"."time" IS '时间信息';

ALTER TABLE "public"."car" ADD CONSTRAINT "ebike_car_pkey" PRIMARY KEY ("code");

-- 电动自行车车辆历史表（2021年6月快照）
DROP TABLE IF EXISTS "public"."ebike_car_history_2021_6";
CREATE TABLE "public"."car_history_2021_6" (
  "hash" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "code" varchar(255) COLLATE "pg_catalog"."default",
  "status" int8,
  "provide" varchar(255) COLLATE "pg_catalog"."default",
  "speed" float8,
  "gps" json,
  "time" json,
  "create_date" timestamp(6),
  "update_date" timestamp(6),
  "delete" bool,
  "alert" varchar(255) COLLATE "pg_catalog"."default",
  "remark" varchar(255) COLLATE "pg_catalog"."default",
  "type" int8
);
COMMENT ON TABLE "public"."car_history_2021_6" IS '电动自行车车辆历史表（2021年6月快照）';

ALTER TABLE "public"."car_history_2021_6" ADD CONSTRAINT "history_2021_6_pkey" PRIMARY KEY ("hash");

-- 电动自行车存储表
DROP TABLE IF EXISTS "public"."ebike_storage";
CREATE TABLE "public"."storage" (
  "code" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "provide" varchar(255) COLLATE "pg_catalog"."default",
  "gps" json,
  "create_date" timestamp(0),
  "update_date" timestamp(0),
  "delete" bool,
  "alert" varchar(255) COLLATE "pg_catalog"."default",
  "remark" varchar(255) COLLATE "pg_catalog"."default",
  "sum" int8,
  "cur" int8,
  "status" int8,
  "points" varchar(5000) COLLATE "pg_catalog"."default",
  "type" int8
);
COMMENT ON TABLE "public"."storage" IS '电动自行车存储表';
COMMENT ON COLUMN "public"."storage"."code" IS '存储编码';
COMMENT ON COLUMN "public"."storage"."provide" IS '供应商';
COMMENT ON COLUMN "public"."storage"."gps" IS 'GPS信息';
COMMENT ON COLUMN "public"."storage"."create_date" IS '创建时间';
COMMENT ON COLUMN "public"."storage"."update_date" IS '更新时间';
COMMENT ON COLUMN "public"."storage"."delete" IS '软删除';
COMMENT ON COLUMN "public"."storage"."alert" IS '告警';
COMMENT ON COLUMN "public"."storage"."remark" IS '备注';
COMMENT ON COLUMN "public"."storage"."sum" IS '总容量';
COMMENT ON COLUMN "public"."storage"."cur" IS '当前容量';
COMMENT ON COLUMN "public"."storage"."status" IS '状态';
COMMENT ON COLUMN "public"."storage"."points" IS '积分';
COMMENT ON COLUMN "public"."storage"."type" IS '类型';

ALTER TABLE "public"."storage" ADD CONSTRAINT "ebike_storage_pkey" PRIMARY KEY ("code");

-- 电动自行车存储历史表（2021年6月快照）
DROP TABLE IF EXISTS "public"."ebike_storage_history_2021_6";
CREATE TABLE "public"."storage_history_2021_6" (
  "hash" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "code" varchar(255) COLLATE "pg_catalog"."default",
  "status" int8,
  "provide" varchar(255) COLLATE "pg_catalog"."default",
  "gps" json,
  "create_date" timestamp(6),
  "update_date" timestamp(6),
  "delete" bool,
  "alert" varchar(255) COLLATE "pg_catalog"."default",
  "remark" varchar(255) COLLATE "pg_catalog"."default",
  "sum" int8,
  "cur" int8,
  "points" varchar(255) COLLATE "pg_catalog"."default",
  "type" int8
);
COMMENT ON TABLE "public"."storage_history_2021_6" IS '电动自行车存储历史表（2021年6月快照）';

ALTER TABLE "public"."storage_history_2021_6" ADD CONSTRAINT "history_2021_6_copy1_pkey" PRIMARY KEY ("hash");

-- 电动自行车订单表（2021年6月快照）
DROP TABLE IF EXISTS "public"."ebike_order_2021_6";
CREATE TABLE "public"."order_2021_6" (
  "code" varchar(255) COLLATE "pg_catalog"."default",
  "status" int8,
  "provide" varchar(255) COLLATE "pg_catalog"."default",
  "speed" float8,
  "gps" json,
  "time" json,
  "create_date" timestamp(0),
  "update_date" timestamp(0),
  "delete" bool,
  "alert" varchar(255) COLLATE "pg_catalog"."default",
  "remark" varchar(255) COLLATE "pg_catalog"."default",
  "type" int8,
  "hash" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "payable" float8,
  "pay" float8,
  "refund" float8,
  "coupon" float8,
  "order" varchar(255) COLLATE "pg_catalog"."default",
  "paytype" int8,
  "paytime" timestamp(0),
  "paystatus" int8
);
COMMENT ON TABLE "public"."order_2021_6" IS '电动自行车订单表（2021年6月快照）';
COMMENT ON COLUMN "public"."order_2021_6"."code" IS '订单编码';
COMMENT ON COLUMN "public"."order_2021_6"."status" IS '状态';
COMMENT ON COLUMN "public"."order_2021_6"."provide" IS '供应商';
COMMENT ON COLUMN "public"."order_2021_6"."speed" IS '速度';
COMMENT ON COLUMN "public"."order_2021_6"."gps" IS 'GPS信息';
COMMENT ON COLUMN "public"."order_2021_6"."time" IS '时间信息';
COMMENT ON COLUMN "public"."order_2021_6"."create_date" IS '创建时间';
COMMENT ON COLUMN "public"."order_2021_6"."update_date" IS '更新时间';
COMMENT ON COLUMN "public"."order_2021_6"."delete" IS '软删除';
COMMENT ON COLUMN "public"."order_2021_6"."alert" IS '告警';
COMMENT ON COLUMN "public"."order_2021_6"."remark" IS '备注';
COMMENT ON COLUMN "public"."order_2021_6"."type" IS '类型';
COMMENT ON COLUMN "public"."order_2021_6"."hash" IS '哈希值';
COMMENT ON COLUMN "public"."order_2021_6"."payable" IS '应付金额';
COMMENT ON COLUMN "public"."order_2021_6"."pay" IS '实付金额';
COMMENT ON COLUMN "public"."order_2021_6"."refund" IS '退款金额';
COMMENT ON COLUMN "public"."order_2021_6"."coupon" IS '优惠券金额';
COMMENT ON COLUMN "public"."order_2021_6"."order" IS '订单号';
COMMENT ON COLUMN "public"."order_2021_6"."paytype" IS '支付类型';
COMMENT ON COLUMN "public"."order_2021_6"."paytime" IS '支付时间';
COMMENT ON COLUMN "public"."order_2021_6"."paystatus" IS '支付状态';

ALTER TABLE "public"."order_2021_6" ADD CONSTRAINT "car_copy1_pkey" PRIMARY KEY ("hash");

-- 电动自行车配置选项表
DROP TABLE IF EXISTS "public"."ebike_options";
CREATE TABLE "public"."options" (
  "name" varchar(255) COLLATE "pg_catalog"."default",
  "options" json,
  "create_date" timestamp(6),
  "update_date" timestamp(6),
  "delete" bool,
  "level" int8
);
COMMENT ON TABLE "public"."options" IS '电动自行车配置选项表';
COMMENT ON COLUMN "public"."options"."name" IS '选项名称';
COMMENT ON COLUMN "public"."options"."options" IS '选项值（JSON）';
COMMENT ON COLUMN "public"."options"."create_date" IS '创建时间';
COMMENT ON COLUMN "public"."options"."update_date" IS '更新时间';
COMMENT ON COLUMN "public"."options"."delete" IS '软删除';
COMMENT ON COLUMN "public"."options"."level" IS '层级';

-- 插入默认 ebike 配置选项
INSERT INTO "public"."options" VALUES ('默认', '{"system":120,"alert":300}', '2021-06-16 22:15:00', '2021-06-16 22:15:02', 'f', 0);

-- =============================================================================
-- Table structures: ebike-service (新增 2026-07-22)
-- =============================================================================

-- 运营商配额表
DROP TABLE IF EXISTS "public"."operators";
CREATE TABLE "public"."operators" (
    provide VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    contact VARCHAR(50),
    phone VARCHAR(20),
    vehicle_quota INT NOT NULL DEFAULT 0,
    status INT DEFAULT 1,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
COMMENT ON TABLE "public"."operators" IS '共享单车运营商表 - 记录各运营企业信息及车辆配额';

-- 违停记录表
DROP TABLE IF EXISTS "public"."violations";
CREATE TABLE "public"."violations" (
    id SERIAL PRIMARY KEY,
    car_code VARCHAR(100) NOT NULL,
    provide VARCHAR(50) NOT NULL REFERENCES operators(provide),
    lng VARCHAR(30),
    lat VARCHAR(30),
    violation_type VARCHAR(100) DEFAULT '乱停乱放',
    status INT DEFAULT 0,
    alert_level INT DEFAULT 1,
    created_at TIMESTAMP DEFAULT NOW(),
    resolved_at TIMESTAMP,
    remark TEXT
);
COMMENT ON TABLE "public"."violations" IS '违停记录表 - 记录共享单车违规停放事件及处理状态';
COMMENT ON COLUMN "public"."violations"."status" IS '状态: 0-待处理, 1-整改中, 2-已整改, 3-超时未处理';

CREATE INDEX IF NOT EXISTS idx_violations_provide ON violations(provide);
CREATE INDEX IF NOT EXISTS idx_violations_status ON violations(status);
CREATE INDEX IF NOT EXISTS idx_violations_car ON violations(car_code);

-- 默认运营商数据
INSERT INTO operators (provide, name, vehicle_quota) VALUES
    ('hello', '哈啰出行', 5000),
    ('meituan', '美团单车', 5000),
    ('qingju', '青桔单车', 5000)
ON CONFLICT (provide) DO NOTHING;

-- =============================================================================
-- Index Optimization: ebike-service
-- =============================================================================

-- Enable pg_trgm extension for efficient LIKE/ILIKE queries
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- GIN indexes for JSON fields (used in JSON containment and key-existence queries)
CREATE INDEX IF NOT EXISTS idx_car_gps ON "public"."car" USING gin (gps);
CREATE INDEX IF NOT EXISTS idx_car_time ON "public"."car" USING gin (time);
CREATE INDEX IF NOT EXISTS idx_storage_gps ON "public"."storage" USING gin (gps);
CREATE INDEX IF NOT EXISTS idx_order_gps ON "public"."order_2021_6" USING gin (gps);
CREATE INDEX IF NOT EXISTS idx_order_time ON "public"."order_2021_6" USING gin (time);
CREATE INDEX IF NOT EXISTS idx_options_options ON "public"."options" USING gin (options);

-- Indexes for LIKE queries (trigram for leading-wildcard LIKE '%...%')
CREATE INDEX IF NOT EXISTS idx_car_code_trgm ON "public"."car" USING gin (code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_car_provide_trgm ON "public"."car" USING gin (provide gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_car_alert_trgm ON "public"."car" USING gin (alert gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_car_remark_trgm ON "public"."car" USING gin (remark gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_car_create_date ON "public"."car" (create_date);
CREATE INDEX IF NOT EXISTS idx_car_update_date ON "public"."car" (update_date);

CREATE INDEX IF NOT EXISTS idx_storage_code_trgm ON "public"."storage" USING gin (code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_storage_provide_trgm ON "public"."storage" USING gin (provide gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_storage_alert_trgm ON "public"."storage" USING gin (alert gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_storage_remark_trgm ON "public"."storage" USING gin (remark gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_order_code_trgm ON "public"."order_2021_6" USING gin (code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_order_provide_trgm ON "public"."order_2021_6" USING gin (provide gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_order_alert_trgm ON "public"."order_2021_6" USING gin (alert gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_order_remark_trgm ON "public"."order_2021_6" USING gin (remark gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_order_order_trgm ON "public"."order_2021_6" USING gin ("order" gin_trgm_ops);

-- =============================================================================
-- Table structures: shared (共享表)
-- =============================================================================

-- 系统配置表
DROP TABLE IF EXISTS "public"."system_configs";
CREATE TABLE "public"."system_configs" (
  "id" int8 NOT NULL DEFAULT nextval('system_configs_id_seq'::regclass),
  "category" varchar(50) COLLATE "pg_catalog"."default" NOT NULL,
  "config_key" varchar(100) COLLATE "pg_catalog"."default" NOT NULL,
  "config_value" text COLLATE "pg_catalog"."default",
  "value_type" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'string'::character varying,
  "label" varchar(100) COLLATE "pg_catalog"."default",
  "description" text COLLATE "pg_catalog"."default",
  "sort_order" int4 DEFAULT 0,
  "status" int4 DEFAULT 1,
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."system_configs" IS '系统配置表 - 全局配置管理，存储系统运行所需的各种配置项';
COMMENT ON COLUMN "public"."system_configs"."category" IS '配置类别: basic-基础配置, security-安全配置, email-邮件配置, upload-上传配置, api-API配置';
COMMENT ON COLUMN "public"."system_configs"."config_key" IS '配置键 - 配置项的唯一标识键';
COMMENT ON COLUMN "public"."system_configs"."config_value" IS '配置值 - 配置项的值';
COMMENT ON COLUMN "public"."system_configs"."value_type" IS '值类型: string-字符串, number-数字, boolean-布尔, json-JSON对象';
COMMENT ON COLUMN "public"."system_configs"."label" IS '配置标签 - 配置项的显示名称';
COMMENT ON COLUMN "public"."system_configs"."description" IS '配置描述 - 配置项的详细说明';
COMMENT ON COLUMN "public"."system_configs"."sort_order" IS '排序号 - 配置列表的排序';
COMMENT ON COLUMN "public"."system_configs"."status" IS '状态: 1-启用, 0-禁用';

-- API 调用日志表
DROP SEQUENCE IF EXISTS "public"."sys_api_call_logs_id_seq";
CREATE SEQUENCE "public"."sys_api_call_logs_id_seq" INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 START 1 CACHE 1;

DROP TABLE IF EXISTS "public"."sys_api_call_logs";
CREATE TABLE "public"."sys_api_call_logs" (
  "id" int8 NOT NULL DEFAULT nextval('sys_api_call_logs_id_seq'::regclass),
  "request_id" varchar(100) COLLATE "pg_catalog"."default",
  "method" varchar(10) COLLATE "pg_catalog"."default",
  "path" varchar(500) COLLATE "pg_catalog"."default",
  "query_params" text COLLATE "pg_catalog"."default",
  "headers" text COLLATE "pg_catalog"."default",
  "request_size" int8 DEFAULT 0,
  "status_code" int4,
  "response_time" int4,
  "response_size" int8 DEFAULT 0,
  "client_ip" varchar(50) COLLATE "pg_catalog"."default",
  "user_id" int8,
  "username" varchar(100) COLLATE "pg_catalog"."default",
  "error" text COLLATE "pg_catalog"."default",
  "created_at" timestamptz(6) DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON TABLE "public"."sys_api_call_logs" IS 'API调用日志表 - 记录所有API请求的详细信息，用于API治理和监控';
COMMENT ON COLUMN "public"."sys_api_call_logs"."request_id" IS '请求ID - 用于链路追踪的唯一标识';
COMMENT ON COLUMN "public"."sys_api_call_logs"."method" IS 'HTTP方法 - GET/POST/PUT/DELETE/PATCH等';
COMMENT ON COLUMN "public"."sys_api_call_logs"."path" IS '请求路径 - API端点路径';
COMMENT ON COLUMN "public"."sys_api_call_logs"."query_params" IS '查询参数 - URL查询参数字符串';
COMMENT ON COLUMN "public"."sys_api_call_logs"."headers" IS '请求头 - HTTP请求头JSON';
COMMENT ON COLUMN "public"."sys_api_call_logs"."request_size" IS '请求体大小 - 请求体字节数';
COMMENT ON COLUMN "public"."sys_api_call_logs"."status_code" IS '状态码 - HTTP响应状态码';
COMMENT ON COLUMN "public"."sys_api_call_logs"."response_time" IS '响应时间 - 请求处理耗时（毫秒）';
COMMENT ON COLUMN "public"."sys_api_call_logs"."response_size" IS '响应大小 - 响应体字节数';
COMMENT ON COLUMN "public"."sys_api_call_logs"."client_ip" IS '客户端IP - 请求来源IP地址';
COMMENT ON COLUMN "public"."sys_api_call_logs"."user_id" IS '用户ID - 发起请求的用户ID';
COMMENT ON COLUMN "public"."sys_api_call_logs"."username" IS '用户名 - 发起请求的用户名';
COMMENT ON COLUMN "public"."sys_api_call_logs"."error" IS '错误信息 - 请求失败时的错误详情';
COMMENT ON COLUMN "public"."sys_api_call_logs"."created_at" IS '创建时间 - 请求发生时间';

CREATE INDEX "idx_api_call_logs_created_at" ON "public"."sys_api_call_logs" ("created_at");
CREATE INDEX "idx_api_call_logs_method" ON "public"."sys_api_call_logs" ("method");
CREATE INDEX "idx_api_call_logs_path" ON "public"."sys_api_call_logs" ("path");
CREATE INDEX "idx_api_call_logs_status_code" ON "public"."sys_api_call_logs" ("status_code");
CREATE INDEX "idx_api_call_logs_user_id" ON "public"."sys_api_call_logs" ("user_id");
CREATE INDEX "idx_api_call_logs_response_time" ON "public"."sys_api_call_logs" ("response_time");

-- =============================================================================
-- Initial data (初始数据)
-- =============================================================================

-- 插入默认系统配置
INSERT INTO "public"."system_configs" ("category", "config_key", "config_value", "value_type", "label", "description", "sort_order") VALUES
('basic', 'site_name', 'MyAI 管理后台', 'string', '站点名称', '网站显示名称', 0),
('basic', 'site_logo', '/logo.png', 'string', '站点 Logo', '网站 Logo 路径', 0),
('basic', 'site_description', 'MyAI 智能管理后台', 'string', '站点描述', '网站描述信息', 0),
('security', 'password_min_length', '6', 'number', '密码最小长度', '用户密码最小长度', 0),
('security', 'password_require_special', 'false', 'boolean', '密码特殊字符', '是否要求密码包含特殊字符', 0),
('security', 'login_max_attempts', '5', 'number', '登录最大尝试次数', '密码错误最大尝试次数', 0),
('security', 'login_lock_duration', '30', 'number', '锁定时长(分钟)', '账户锁定时长', 0),
('security', 'session_timeout', '3600', 'number', '会话超时(秒)', '用户会话超时时间', 0),
('upload', 'max_file_size', '10485760', 'number', '最大文件大小', '上传文件最大大小(字节)', 0),
('upload', 'allowed_extensions', 'jpg,png,pdf,doc,docx,xls,xlsx', 'string', '允许的扩展名', '允许上传的文件扩展名', 0),
('upload', 'save_path', '/uploads', 'string', '保存路径', '文件保存路径', 0);

-- 插入默认角色
INSERT INTO "public"."roles" ("id", "name", "code", "description", "role_type", "parent_id", "level", "sort_order", "status", "is_default") VALUES
(1, '超级管理员', 'super_admin', '拥有系统所有权限', 'system', NULL, 0, 0, 1, 'f'),
(2, '系统管理员', 'admin', '系统管理权限', 'system', NULL, 1, 0, 1, 'f'),
(3, '普通用户', 'user', '普通用户权限', 'system', NULL, 1, 0, 1, 't'),
(4, '访客', 'guest', '只读权限', 'system', NULL, 1, 0, 1, 'f');

-- 插入默认权限
INSERT INTO "public"."permissions" ("id", "name", "code", "permission_type", "parent_id", "path", "method", "icon", "sort_order", "status") VALUES
(1, '用户列表', 'user:list', 'button', NULL, '/api/admin/users', 'GET', NULL, 0, 1),
(2, '创建用户', 'user:create', 'button', NULL, '/api/admin/users', 'POST', NULL, 0, 1),
(3, '更新用户', 'user:update', 'button', NULL, '/api/admin/users', 'PUT', NULL, 0, 1),
(4, '删除用户', 'user:delete', 'button', NULL, '/api/admin/users', 'DELETE', NULL, 0, 1),
(5, '重置密码', 'user:reset_password', 'button', NULL, '/api/admin/users/*/reset-password', 'POST', NULL, 0, 1),
(6, '批量操作', 'user:batch', 'button', NULL, '/api/admin/users/batch-*', 'POST', NULL, 0, 1),
(7, '角色列表', 'role:list', 'button', NULL, '/api/admin/roles', 'GET', NULL, 0, 1),
(8, '创建角色', 'role:create', 'button', NULL, '/api/admin/roles', 'POST', NULL, 0, 1),
(9, '更新角色', 'role:update', 'button', NULL, '/api/admin/roles', 'PUT', NULL, 0, 1),
(10, '删除角色', 'role:delete', 'button', NULL, '/api/admin/roles', 'DELETE', NULL, 0, 1),
(11, '角色权限', 'role:permissions', 'button', NULL, '/api/admin/roles/*/permissions', 'GET,PUT', NULL, 0, 1),
(12, '部门列表', 'department:list', 'button', NULL, '/api/admin/departments', 'GET', NULL, 0, 1),
(13, '创建部门', 'department:create', 'button', NULL, '/api/admin/departments', 'POST', NULL, 0, 1),
(14, '更新部门', 'department:update', 'button', NULL, '/api/admin/departments', 'PUT', NULL, 0, 1),
(15, '删除部门', 'department:delete', 'button', NULL, '/api/admin/departments', 'DELETE', NULL, 0, 1),
(16, '配置查看', 'config:view', 'button', NULL, '/api/admin/system/configs', 'GET', NULL, 0, 1),
(17, '配置修改', 'config:update', 'button', NULL, '/api/admin/system/configs', 'PUT', NULL, 0, 1),
(18, '公告列表', 'announcement:list', 'button', NULL, '/api/admin/announcements', 'GET', NULL, 0, 1),
(19, '创建公告', 'announcement:create', 'button', NULL, '/api/admin/announcements', 'POST', NULL, 0, 1),
(20, '更新公告', 'announcement:update', 'button', NULL, '/api/admin/announcements', 'PUT', NULL, 0, 1),
(21, '删除公告', 'announcement:delete', 'button', NULL, '/api/admin/announcements', 'DELETE', NULL, 0, 1);

-- 插入超级管理员角色权限
INSERT INTO "public"."role_permissions" ("id", "role_id", "permission_id") VALUES
(1, 1, 1), (2, 1, 2), (3, 1, 3), (4, 1, 4), (5, 1, 5), (6, 1, 6),
(7, 1, 7), (8, 1, 8), (9, 1, 9), (10, 1, 10), (11, 1, 11),
(12, 1, 12), (13, 1, 13), (14, 1, 14), (15, 1, 15),
(16, 1, 16), (17, 1, 17), (18, 1, 18), (19, 1, 19), (20, 1, 20), (21, 1, 21);

-- 插入管理员角色权限
INSERT INTO "public"."role_permissions" ("id", "role_id", "permission_id") VALUES
(22, 2, 1), (23, 2, 2), (24, 2, 3), (25, 2, 5), (26, 2, 6),
(27, 2, 7), (28, 2, 8), (29, 2, 9), (30, 2, 11),
(31, 2, 12), (32, 2, 13), (33, 2, 14),
(34, 2, 16), (35, 2, 17),
(36, 2, 18), (37, 2, 19), (38, 2, 20);

-- 插入普通用户角色权限
INSERT INTO "public"."role_permissions" ("id", "role_id", "permission_id") VALUES
(39, 3, 1), (40, 3, 12);

-- 插入默认用户 (密码: admin123)
INSERT INTO "public"."users" ("id", "username", "nickname", "password_hash", "avatar", "phone", "email", "gender", "address", "role", "status") VALUES
(1, 'admin', 'Administrator', '$2b$12$CZm6fwvZeC6N6V0rxs9a/O9zADNIWs9GpniO2TnuMlN9CHw5fXuri', NULL, NULL, 'admin@example.com', 0, NULL, 'admin', 1);

-- 插入用户角色
INSERT INTO "public"."user_roles" ("id", "user_id", "role_id") VALUES
(1, 1, 1);

-- 插入默认部门
INSERT INTO "public"."departments" ("id", "name", "code", "parent_id", "level", "sort_order", "leader_id", "description", "status") VALUES
(1, '总公司', 'hq', NULL, 0, 0, NULL, NULL, 1),
(2, '技术部', 'tech', 1, 1, 1, NULL, NULL, 1),
(3, '运营部', 'ops', 1, 1, 2, NULL, NULL, 1),
(4, '财务部', 'finance', 1, 1, 3, NULL, NULL, 1),
(5, '前端组', 'frontend', 2, 2, 1, NULL, NULL, 1),
(6, '后端组', 'backend', 2, 2, 2, NULL, NULL, 1);

-- 插入默认CMS分类
INSERT INTO "public"."cms_category" ("id", "parent_id", "name", "slug", "description", "icon", "sort_order", "seo_title", "seo_keywords", "seo_description", "status", "allow_attachment") VALUES
(1, 0, '新闻资讯', 'news', NULL, NULL, 1, NULL, NULL, NULL, 1, 1),
(2, 0, '产品动态', 'product', NULL, NULL, 2, NULL, NULL, NULL, 1, 1),
(3, 0, '技术文档', 'docs', NULL, NULL, 3, NULL, NULL, NULL, 1, 1),
(4, 0, '帮助中心', 'help', NULL, NULL, 4, NULL, NULL, NULL, 1, 1);

-- 插入默认公告
INSERT INTO "public"."announcements" ("id", "title", "content", "announcement_type", "priority", "is_pinned", "is_active", "start_time", "end_time", "created_by", "created_at", "updated_at") VALUES
(1, '欢迎使用 MyAI 管理后台', '欢迎使用 MyAI 智能管理后台系统，祝您使用愉快！', 'info', 1, 't', 't', '2026-05-13 13:53:21.49594+00', NULL, NULL, '2026-05-13 13:53:21.49594+00', '2026-05-13 13:53:21.49594+00'),
(2, '系统维护通知', '系统将于本周日凌晨 2:00-6:00 进行例行维护，请提前做好准备。', 'warning', 0, 'f', 't', '2026-05-13 13:53:21.49594+00', NULL, NULL, '2026-05-13 13:53:21.49594+00', '2026-05-13 13:53:21.49594+00');

-- 插入车辆类型
INSERT INTO "public"."tow_car_type" ("name", "remark", "sort_order") VALUES
('小型车', '蓝色车牌', 1),
('大型车', '黄色车牌', 2),
('新能源车', '绿色车牌', 3),
('摩托车', '两轮车', 4);

-- 插入车辆颜色
INSERT INTO "public"."tow_car_color" ("name", "remark", "sort_order") VALUES
('白色', '白色车身', 1),
('黑色', '黑色车身', 2),
('银色', '银色车身', 3),
('灰色', '灰色车身', 4),
('红色', '红色车身', 5),
('蓝色', '蓝色车身', 6),
('黄色', '黄色车身', 7),
('绿色', '绿色车身', 8);

-- 插入扣押原因类型
INSERT INTO "public"."tow_dc_type" ("name", "code", "remark", "sort_order") VALUES
('违章停车', 'illegal_parking', '违反交通法规停放', 1),
('占道经营', 'street_vending', '占用道路经营', 2),
('乱停乱放', 'random_parking', '随意停放影响交通', 3),
('其他', 'other', '其他违规行为', 99);

-- 插入扣押原因明细
INSERT INTO "public"."tow_dc_causes" ("name", "type_id", "type_name", "remark", "sort_order") VALUES
('在人行道停放', 1, '违章停车', '在人行道上违章停放', 1),
('在机动车道停放', 1, '违章停车', '占用机动车道', 2),
('在消防通道停放', 1, '违章停车', '占用消防通道', 3),
('在黄色网格线内停放', 1, '违章停车', '停在黄色网格线内', 4),
('在盲道上停放', 1, '违章停车', '占用盲道', 5),
('超时停放', 2, '占道经营', '超出允许停放时间', 1),
('超出经营区域', 2, '占道经营', '超出规定的经营区域', 2);

-- 插入消息模板
INSERT INTO "public"."message_templates" ("id", "name", "template_type", "title_template", "content_template", "variables", "is_active") VALUES
(1, '欢迎通知', 'notification', '欢迎 ${username} 加入系统', '欢迎 ${username}！您的账户已成功创建，可以开始使用系统了。', '["username"]', true),
(2, '密码重置', 'email', '密码重置验证码', '您的验证码是 ${code}，有效期 ${expire_minutes} 分钟，请勿泄露给他人。', '["code", "expire_minutes"]', true),
(3, '订单通知', 'push', '订单 ${order_id} 状态更新', '您的订单 ${order_id} 已 ${status}，${message}', '["order_id", "status", "message"]', true);

-- =============================================================================
-- 演示数据 (2026-08-12) - 第一批: 核心业务表
-- 全部使用 ON CONFLICT DO NOTHING / NOT EXISTS 守卫, 幂等可重入。
-- 密码均为 bcrypt 演示哈希 (明文仅供演示环境, 生产由部署注入)。
-- 字段严格对齐线上 \d 实采结构 (2026-08-12 审计)。
-- =============================================================================

-- 拖车业务: 扣押车辆 (tow-service)
INSERT INTO "public"."tow_car" (
    "id", "license", "vehicle", "engine", "car_type", "dc_type", "dc_causes",
    "car_color", "dc_date", "dc_address", "dc_key", "dc_party_name", "dc_party_cardid",
    "dc_party_tel", "p_name", "p_id", "dc_acc", "dc_name", "dc_into_date", "car_remark",
    "driver", "operator", "drag_km", "drag_unit", "drag_money", "cmd_unit", "cmd_user",
    "cv", "cv_acc", "cv_name", "cv_date", "cv_opinion", "tv", "tv_acc", "tv_name",
    "tv_date", "tv_opinion", "rc_name", "rc_idcard", "rc_tel", "parking_date",
    "parking_unit", "parking_money", "parking_payable", "parking_paidin", "remark",
    "rs_name", "rs_acc", "rs_date", "attachment", "create_date", "create_user",
    "update_date", "update_user", "delete"
) VALUES
(1, '粤B12345', '{"brand":"丰田","model":"卡罗拉","color":"白色"}', '4A1B2C3D4E5F', '小型车', '违章停车', '在人行道停放',
 '白色', '2026-08-10 09:30', '深圳市南山区科技园路', 'KEY-001', '张三', '440300199001011234',
 '13800000001', '李四', '440300199202023456', 'ACC-001', '王五', '2026-08-10 10:00', '演示车辆-违停拖移',
 '张师傅', '赵六', '3.5', '公里', '200.00', '第一中队', '孙七',
 '审批中', 'APPROVE-001', '周八', '2026-08-10 11:00', '同意拖移', '放行中', 'TRUCK-001', '吴九',
 '2026-08-10 12:00', '车辆已入停车场', '郑十', '440300198803037890', '13700000002', '2026-08-10 13:00',
 '南山区第一停车场', '50.00', '200.00', '50.00', '演示数据-已拖移',
 '钱十一', 'RS-001', '2026-08-10 14:00', '[]'::jsonb, '2026-08-10 09:30:00+08', 'admin',
 '2026-08-10 14:00:00+08', 'admin', false)
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."tow_car" (
    "id", "license", "vehicle", "engine", "car_type", "dc_type", "dc_causes",
    "car_color", "dc_date", "dc_address", "dc_key", "dc_party_name", "dc_party_cardid",
    "dc_party_tel", "p_name", "p_id", "dc_acc", "dc_name", "dc_into_date", "car_remark",
    "driver", "operator", "drag_km", "drag_unit", "drag_money", "cmd_unit", "cmd_user",
    "cv", "cv_acc", "cv_name", "cv_date", "cv_opinion", "tv", "tv_acc", "tv_name",
    "tv_date", "tv_opinion", "rc_name", "rc_idcard", "rc_tel", "parking_date",
    "parking_unit", "parking_money", "parking_payable", "parking_paidin", "remark",
    "rs_name", "rs_acc", "rs_date", "attachment", "create_date", "create_user",
    "update_date", "update_user", "delete"
) VALUES
(2, '粤B67890', '{"brand":"比亚迪","model":"秦PLUS","color":"灰色"}', '5B2C3D4E5F6A', '新能源车', '占道经营', '超时停放',
 '灰色', '2026-08-11 15:20', '深圳市福田区会展中心路', 'KEY-002', '王芳', '440300198512126789',
 '13900000003', '刘强', '440300199303038901', 'ACC-002', '陈晨', '2026-08-11 16:00', '演示车辆-占道经营',
 '李师傅', '赵六', '2.0', '公里', '150.00', '第二中队', '孙七',
 '已审批', 'APPROVE-002', '周八', '2026-08-11 17:00', '同意拖移', '已放行', 'TRUCK-002', '吴九',
 '2026-08-11 18:00', '车主已领取车辆', '钱十一', '440300197903045678', '13600000004', '2026-08-11 19:00',
 '福田区第二停车场', '50.00', '150.00', '150.00', '演示数据-已处理完成',
 '郑十', 'RS-002', '2026-08-11 20:00', '[]'::jsonb, '2026-08-11 15:20:00+08', 'admin',
 '2026-08-11 20:00:00+08', 'admin', false)
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."tow_car" (
    "id", "license", "vehicle", "engine", "car_type", "dc_type", "dc_causes",
    "car_color", "dc_date", "dc_address", "dc_key", "dc_party_name", "dc_party_cardid",
    "dc_party_tel", "p_name", "p_id", "dc_acc", "dc_name", "dc_into_date", "car_remark",
    "driver", "operator", "drag_km", "drag_unit", "drag_money", "cmd_unit", "cmd_user",
    "cv", "cv_acc", "cv_name", "cv_date", "cv_opinion", "tv", "tv_acc", "tv_name",
    "tv_date", "tv_opinion", "rc_name", "rc_idcard", "rc_tel", "parking_date",
    "parking_unit", "parking_money", "parking_payable", "parking_paidin", "remark",
    "rs_name", "rs_acc", "rs_date", "attachment", "create_date", "create_user",
    "update_date", "update_user", "delete"
) VALUES
(3, '粤C24680', '{"brand":"本田","model":"雅阁","color":"黑色"}', '6C3D4E5F6A7B', '大型车', '乱停乱放', '在盲道上停放',
 '黑色', '2026-08-12 08:10', '深圳市宝安区创业一路', 'KEY-003', '陈志明', '440300197205067890',
 '13500000005', '黄丽', '440300199404046789', 'ACC-003', '林涛', '2026-08-12 08:40', '演示车辆-乱停放',
 '王师傅', '赵六', '1.2', '公里', '100.00', '第三中队', '孙七',
 '待审批', 'APPROVE-003', '周八', NULL, NULL, NULL, NULL,
 NULL, NULL, NULL, NULL, NULL, NULL, '宝安区第三停车场', '50.00', '100.00', '0.00', '演示数据-拖移中',
 NULL, NULL, NULL, NULL, '[]'::jsonb, '2026-08-12 08:10:00+08', 'admin',
 '2026-08-12 08:40:00+08', 'admin', false)
ON CONFLICT (id) DO NOTHING;

-- 电动自行车车辆 (ebike-service / car 表, provide 引用 operators)
INSERT INTO "public"."car" ("code", "status", "provide", "speed", "gps", "create_date", "update_date", "delete", "alert", "remark", "type", "time") VALUES
('EB-HL-0001', 1, 'hellobike', 0.0, '{"lng":104.255441,"lat":23.382518}', '2026-08-01 08:00:00', '2026-08-12 09:00:00', false, NULL, '演示车辆-哈啰', 1, '{"last_ride":"2026-08-11 20:30"}')
ON CONFLICT (code) DO NOTHING;

INSERT INTO "public"."car" ("code", "status", "provide", "speed", "gps", "create_date", "update_date", "delete", "alert", "remark", "type", "time") VALUES
('EB-MT-0002', 1, 'meituan', 0.0, '{"lng":104.245749,"lat":23.386418}', '2026-08-02 09:00:00', '2026-08-12 09:05:00', false, '电量低', '演示车辆-美团', 1, '{"last_ride":"2026-08-12 07:45"}')
ON CONFLICT (code) DO NOTHING;

INSERT INTO "public"."car" ("code", "status", "provide", "speed", "gps", "create_date", "update_date", "delete", "alert", "remark", "type", "time") VALUES
('EB-QJ-0003', 1, 'qingju', 0.0, '{"lng":104.249100,"lat":23.375766}', '2026-08-03 10:00:00', '2026-08-12 09:10:00', false, NULL, '演示车辆-青桔', 1, '{"last_ride":"2026-08-12 08:15"}')
ON CONFLICT (code) DO NOTHING;

INSERT INTO "public"."car" ("code", "status", "provide", "speed", "gps", "create_date", "update_date", "delete", "alert", "remark", "type", "time") VALUES
('EB-HL-0004', 2, 'hellobike', 0.0, '{"lng":104.251440,"lat":23.393186}', '2026-08-04 11:00:00', '2026-08-12 09:15:00', false, '违停告警', '演示车辆-违停处理中', 2, '{"last_ride":"2026-08-10 18:20"}')
ON CONFLICT (code) DO NOTHING;

INSERT INTO "public"."car" ("code", "status", "provide", "speed", "gps", "create_date", "update_date", "delete", "alert", "remark", "type", "time") VALUES
('EB-MT-0005', 3, 'meituan', 0.0, '{"lng":104.257949,"lat":23.389174}', '2026-08-05 12:00:00', '2026-08-12 09:20:00', true, NULL, '演示车辆-已注销', 1, '{"last_ride":"2026-08-01 10:00"}')
ON CONFLICT (code) DO NOTHING;

-- 违停记录 (ebike-service / violations, provide 引用 operators)
INSERT INTO "public"."violations" ("id", "car_code", "provide", "lng", "lat", "violation_type", "status", "alert_level", "created_at", "resolved_at", "remark") VALUES
(1, 'EB-HL-0004', 'hellobike', '104.258374', '23.384138', '乱停乱放', 0, 2, '2026-08-12 08:30:00', NULL, '停在人行道禁停区')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."violations" ("id", "car_code", "provide", "lng", "lat", "violation_type", "status", "alert_level", "created_at", "resolved_at", "remark") VALUES
(2, 'EB-MT-0002', 'meituan', '104.245871', '23.381181', '乱停乱放', 1, 1, '2026-08-11 14:20:00', NULL, '停在消防通道附近')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."violations" ("id", "car_code", "provide", "lng", "lat", "violation_type", "status", "alert_level", "created_at", "resolved_at", "remark") VALUES
(3, 'EB-QJ-0003', 'qingju', '104.249594', '23.382302', '乱停乱放', 2, 1, '2026-08-10 16:00:00', '2026-08-11 10:00:00', '已整改完成')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."violations" ("id", "car_code", "provide", "lng", "lat", "violation_type", "status", "alert_level", "created_at", "resolved_at", "remark") VALUES
(4, 'EB-HL-0001', 'hellobike', '104.241684', '23.383651', '乱停乱放', 3, 3, '2026-08-09 09:00:00', NULL, '超时未处理-重点关注')
ON CONFLICT (id) DO NOTHING;

-- CMS 文章 (cms-service, category_id 引用 cms_category seed id 1-4)
-- content_type: 0=纯文本 1=富文本 2=markdown; status: 1=已发布 0=草稿
INSERT INTO "public"."cms_article" ("id", "category_id", "title", "slug", "summary", "content", "content_type", "cover_image", "author_id", "author_name", "tags", "view_count", "like_count", "comment_count", "share_count", "is_featured", "is_top", "is_draft", "status", "reject_reason", "published_at", "source", "source_url", "attachments", "seo_title", "seo_keywords", "seo_description", "created_at", "updated_at", "deleted_at") VALUES
(1, 1, '平台新功能上线公告', 'platform-update-202608', '本月平台新增多项功能', '<h1>新功能上线</h1><p>平台本月上线了报表导出、工作流审批等功能。</p>', 1, NULL, 1, 'admin', '公告,新功能', 120, 15, 3, 8, true, true, false, 1, NULL, '2026-08-01 09:00:00+08', 'internal', NULL, NULL, '平台新功能上线公告', '新功能,上线', '平台本月新功能', '2026-08-01 08:30:00+08', '2026-08-01 09:00:00+08', NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."cms_article" ("id", "category_id", "title", "slug", "summary", "content", "content_type", "cover_image", "author_id", "author_name", "tags", "view_count", "like_count", "comment_count", "share_count", "is_featured", "is_top", "is_draft", "status", "reject_reason", "published_at", "source", "source_url", "attachments", "seo_title", "seo_keywords", "seo_description", "created_at", "updated_at", "deleted_at") VALUES
(2, 3, '拖车服务流程说明', 'tow-process-guide', '拖车服务的完整流程介绍', '<p>拖车服务流程: 现场扣押 → 停车场入库 → 通知车主 → 处理完成。</p>', 2, NULL, 1, 'admin', '拖车,流程', 86, 9, 2, 5, false, false, false, 1, NULL, '2026-08-05 10:00:00+08', 'internal', NULL, NULL, '拖车服务流程说明', '拖车,流程', '拖车流程介绍', '2026-08-05 09:30:00+08', '2026-08-05 10:00:00+08', NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."cms_article" ("id", "category_id", "title", "slug", "summary", "content", "content_type", "cover_image", "author_id", "author_name", "tags", "view_count", "like_count", "comment_count", "share_count", "is_featured", "is_top", "is_draft", "status", "reject_reason", "published_at", "source", "source_url", "attachments", "seo_title", "seo_keywords", "seo_description", "created_at", "updated_at", "deleted_at") VALUES
(3, 2, 'Q3季度产品规划草案', 'q3-plan-draft', '第三季度产品规划（草稿）', '<p>Q3 规划: 完善多租户、报表中心、消息推送。</p>', 1, NULL, 1, 'admin', '规划,草稿', 0, 0, 0, 0, false, false, true, 0, NULL, NULL, 'internal', NULL, NULL, 'Q3季度产品规划草案', '规划,季度', 'Q3规划草稿', '2026-08-08 14:00:00+08', '2026-08-08 14:00:00+08', NULL)
ON CONFLICT (id) DO NOTHING;

-- 工作流定义 (workflow-service)
INSERT INTO "public"."workflows" ("id", "name", "description", "definition", "status", "version", "created_by", "created_at", "updated_at") VALUES
('wf-leave-approval', '请假审批流程', '员工请假审批: 提交 → 主管审批 → 人事归档', '{"nodes":[{"id":"start","type":"start","name":"开始"},{"id":"submit","type":"task","name":"提交申请"},{"id":"manager","type":"approve","name":"主管审批"},{"id":"hr","type":"approve","name":"人事归档"},{"id":"end","type":"end","name":"结束"}],"edges":[{"from":"start","to":"submit"},{"from":"submit","to":"manager"},{"from":"manager","to":"hr"},{"from":"hr","to":"end"}]}'::jsonb, 'active', 1, 'admin', '2026-08-01 10:00:00+08', '2026-08-01 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."workflows" ("id", "name", "description", "definition", "status", "version", "created_by", "created_at", "updated_at") VALUES
('wf-reimburse', '报销审批流程', '费用报销: 提交 → 部门审批 → 财务打款', '{"nodes":[{"id":"start","type":"start","name":"开始"},{"id":"submit","type":"task","name":"提交报销"},{"id":"dept","type":"approve","name":"部门审批"},{"id":"finance","type":"approve","name":"财务打款"},{"id":"end","type":"end","name":"结束"}],"edges":[{"from":"start","to":"submit"},{"from":"submit","to":"dept"},{"from":"dept","to":"finance"},{"from":"finance","to":"end"}]}'::jsonb, 'draft', 2, 'admin', '2026-08-06 15:00:00+08', '2026-08-06 15:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 报表定义 (report-service)
INSERT INTO "public"."reports" ("id", "name", "description", "report_type", "config", "schedule", "status", "created_by", "created_at", "updated_at") VALUES
('rp-tow-stats', '拖车业务统计表', '按月统计拖车数量与费用', 'table', '{"columns":["月份","拖车数量","总费用"],"data_source":"tow_car","group_by":"month"}'::jsonb, '{"cron":"0 8 1 * *","enabled":true}'::jsonb, 'active', 'admin', '2026-08-02 09:00:00+08', '2026-08-02 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."reports" ("id", "name", "description", "report_type", "config", "schedule", "status", "created_by", "created_at", "updated_at") VALUES
('rp-car-overview', '车辆实时分布图', '单车实时分布热力图', 'chart', '{"type":"heatmap","data_source":"car","field":"gps"}'::jsonb, NULL, 'active', 'admin', '2026-08-03 11:00:00+08', '2026-08-03 11:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- API 密钥 (api-key-service, id 为 varchar, secret_key_hash 为演示占位哈希)
INSERT INTO "public"."api_keys" ("id", "name", "description", "key_id", "secret_key_hash", "key_hint", "permission_level", "allowed_ips", "rate_limit", "tenant_id", "user_id", "status", "created_at", "expires_at", "last_used_at", "updated_at") VALUES
('ak-0001', '第三方对接密钥', '演示用第三方系统对接', 'ak-demo-001', '$2b$12$DemoPlaceholderHashDoNotUseInProd0', 'ak-demo-001...abc', 2, '["192.0.2.0/24"]', 1000, NULL, 1, 'active', '2026-08-01 09:00:00+08', '2027-08-01 09:00:00+08', '2026-08-12 08:00:00+08', '2026-08-01 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."api_keys" ("id", "name", "description", "key_id", "secret_key_hash", "key_hint", "permission_level", "allowed_ips", "rate_limit", "tenant_id", "user_id", "status", "created_at", "expires_at", "last_used_at", "updated_at") VALUES
('ak-0002', '数据同步密钥', '演示用数据同步', 'ak-demo-002', '$2b$12$DemoPlaceholderHashDoNotUseInProd0', 'ak-demo-002...xyz', 1, '[]', 500, NULL, 1, 'active', '2026-08-05 10:00:00+08', '2026-09-01 10:00:00+08', NULL, '2026-08-05 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 数据字典 (dictionary_types / dictionary_items, 字段为 sort 非 sort_order)
INSERT INTO "public"."dictionary_types" ("id", "code", "name", "description", "sort", "status", "created_at", "updated_at") VALUES
(1, 'vehicle_type', '车辆类型', '拖车业务车辆类型字典', 1, 1, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08'),
(2, 'message_priority', '消息优先级', '系统消息优先级', 2, 1, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."dictionary_items" ("id", "type_id", "label", "value", "sort", "status", "is_default", "remark", "created_at", "updated_at") VALUES
(1, 1, '小型车', 'small', 1, 1, true, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08'),
(2, 1, '大型车', 'large', 2, 1, false, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08'),
(3, 1, '新能源车', 'new_energy', 3, 1, false, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08'),
(4, 2, '高', 'high', 1, 1, false, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08'),
(5, 2, '中', 'medium', 2, 1, true, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08'),
(6, 2, '低', 'low', 3, 1, false, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 系统消息 (messaging-service)
INSERT INTO "public"."sys_message" ("id", "type", "title", "content", "priority", "target_type", "target_ids", "sender_id", "sender_name", "expire_time", "created_at", "updated_at") VALUES
(1, 'system', '系统维护通知', '本周六凌晨 2:00-4:00 系统维护，期间服务不可用。', 1, 'all', NULL, 1, 'admin', NULL, '2026-08-10 09:00:00+08', '2026-08-10 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."sys_message" ("id", "type", "title", "content", "priority", "target_type", "target_ids", "sender_id", "sender_name", "expire_time", "created_at", "updated_at") VALUES
(2, 'notice', '拖车费用标准调整', '自 9 月起拖车费用标准调整，详见公告。', 0, 'user', '[1]'::jsonb, 1, 'admin', NULL, '2026-08-11 10:00:00+08', '2026-08-11 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."sys_message_user" ("id", "message_id", "user_id", "is_read", "is_starred", "is_deleted", "is_archived", "read_time", "created_at") VALUES
(1, 1, 1, 1, 0, 0, 0, '2026-08-10 10:00:00+08', '2026-08-10 09:00:00+08'),
(2, 2, 1, 0, 0, 0, 0, NULL, '2026-08-11 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 电动自行车用户 (ebike_user, 密码为 bcrypt 演示哈希)
-- ebike_user 无唯一约束, 用 NOT EXISTS 保证幂等
INSERT INTO "public"."ebike_user" ("username", "password", "create_date", "update_date", "delete", "status", "last_login", "level")
SELECT 'ebike_demo1', '$2b$12$DemoPlaceholderHashDoNotUseInProd0', '2026-08-01 09:00:00', '2026-08-12 09:00:00', false, 1, '2026-08-12 08:00:00', 1
WHERE NOT EXISTS (SELECT 1 FROM "public"."ebike_user" WHERE username = 'ebike_demo1');

INSERT INTO "public"."ebike_user" ("username", "password", "create_date", "update_date", "delete", "status", "last_login", "level")
SELECT 'ebike_demo2', '$2b$12$DemoPlaceholderHashDoNotUseInProd0', '2026-08-02 10:00:00', '2026-08-11 10:00:00', false, 1, '2026-08-11 09:00:00', 2
WHERE NOT EXISTS (SELECT 1 FROM "public"."ebike_user" WHERE username = 'ebike_demo2');

-- LPR 车牌识别记录 (lpr-service)
INSERT INTO "public"."lpr_pass_records" ("id", "plate_no", "plate_color", "plate_type", "vehicle_type", "device_id", "device_name", "park_code", "lane_code", "direction", "pass_time", "image_url", "confidence", "status", "related_order_id", "remark", "created_at", "updated_at") VALUES
(1, '粤B12345', 'blue', 'small', 'car', 'DEV-001', '东门入口摄像机', 'PK-001', 'L1', 'entry', '2026-08-12 08:05:00', '/images/lpr/demo1.jpg', 0.97, 'processed', 'ORDER-001', '演示数据-入场', '2026-08-12 08:05:00', '2026-08-12 08:06:00')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."lpr_pass_records" ("id", "plate_no", "plate_color", "plate_type", "vehicle_type", "device_id", "device_name", "park_code", "lane_code", "direction", "pass_time", "image_url", "confidence", "status", "related_order_id", "remark", "created_at", "updated_at") VALUES
(2, '粤B67890', 'green', 'new_energy', 'car', 'DEV-002', '西门出口摄像机', 'PK-001', 'L2', 'exit', '2026-08-12 09:10:00', '/images/lpr/demo2.jpg', 0.95, 'processed', 'ORDER-002', '演示数据-出场', '2026-08-12 09:10:00', '2026-08-12 09:11:00')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."lpr_pass_records" ("id", "plate_no", "plate_color", "plate_type", "vehicle_type", "device_id", "device_name", "park_code", "lane_code", "direction", "pass_time", "image_url", "confidence", "status", "related_order_id", "remark", "created_at", "updated_at") VALUES
(3, '粤C24680', 'blue', 'large', 'truck', 'DEV-003', '北门入口摄像机', 'PK-002', 'L1', 'entry', '2026-08-12 07:55:00', '/images/lpr/demo3.jpg', 0.88, 'pending', '', '演示数据-待处理', '2026-08-12 07:55:00', '2026-08-12 07:55:00')
ON CONFLICT (id) DO NOTHING;

-- SocialOps 社交账号 (social-ops-service, socialops schema)
INSERT INTO socialops.social_accounts ("id", "user_id", "platform", "account_name", "account_id", "avatar_url", "is_active", "config_json", "created_at", "updated_at") VALUES
('a1b2c3d4-0000-0000-0000-000000000001', NULL, 'wechat', '深圳交警发布', 'wx-official-001', NULL, true, '{"category":"官方账号","description":"演示用官方公众号"}'::jsonb, '2026-08-01 09:00:00+08', '2026-08-12 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO socialops.social_accounts ("id", "user_id", "platform", "account_name", "account_id", "avatar_url", "is_active", "config_json", "created_at", "updated_at") VALUES
('a1b2c3d4-0000-0000-0000-000000000002', NULL, 'weibo', '深圳拖车服务', 'wb-001', NULL, true, '{"category":"业务账号","description":"演示用微博业务号"}'::jsonb, '2026-08-02 10:00:00+08', '2026-08-12 09:05:00+08')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- 演示数据 (2026-08-12) - 第二批: 租户/审计/调度模块
-- =============================================================================

-- 租户 (tenant-service, 第二批新增表)
INSERT INTO "public"."tenants" ("id", "name", "code", "domain", "description", "max_users", "max_storage", "status", "expires_at", "created_at", "updated_at") VALUES
(1, '深圳总公司', 'sz-hq', 'sz-hq.example.com', '演示用深圳总部租户', 500, 107374182400, 1, NULL, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."tenants" ("id", "name", "code", "domain", "description", "max_users", "max_storage", "status", "expires_at", "created_at", "updated_at") VALUES
(2, '广州分公司', 'gz-branch', 'gz-branch.example.com', '演示用广州分公司租户', 200, 53687091200, 1, '2027-12-31 23:59:59+08', '2026-08-02 10:00:00+08', '2026-08-02 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 租户用户关联 (tenant-service)
INSERT INTO "public"."tenant_users" ("id", "tenant_id", "user_id", "role", "department", "position", "status", "joined_at", "created_at") VALUES
(1, 1, 1, 'admin', '技术部', '系统管理员', 1, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 报表任务 (report-service, report_id 引用 reports seed)
INSERT INTO "public"."report_tasks" ("id", "report_id", "status", "result", "error_message", "started_at", "completed_at") VALUES
('rt-0001', 'rp-tow-stats', 'completed', '{"rows": 12, "generated_at": "2026-08-12T08:00:00+08:00"}'::jsonb, NULL, '2026-08-12 08:00:00+08', '2026-08-12 08:01:00+08')
ON CONFLICT (id) DO NOTHING;

-- 调度任务 (schedule-task)
INSERT INTO "public"."schedule_tasks" ("id", "name", "description", "task_type", "cron_expression", "interval_seconds", "start_time", "end_time", "task_handler", "task_params", "status", "execute_strategy", "max_retries", "retry_count", "timeout_seconds", "last_run_time", "next_run_time", "created_by", "tenant_id", "created_at", "updated_at") VALUES
('st-0001', '每日报表生成', '每天凌晨生成拖车统计报表', 'cron', '0 1 * * *', NULL, '2026-08-01 00:00:00+08', NULL, 'report_generator', '{"report_id": "rp-tow-stats"}'::jsonb, 'running', 'immediate', 3, 0, 3600, '2026-08-12 01:00:00+08', '2026-08-13 01:00:00+08', 'admin', '1', '2026-08-01 00:00:00+08', '2026-08-12 01:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."schedule_tasks" ("id", "name", "description", "task_type", "cron_expression", "interval_seconds", "start_time", "end_time", "task_handler", "task_params", "status", "execute_strategy", "max_retries", "retry_count", "timeout_seconds", "last_run_time", "next_run_time", "created_by", "tenant_id", "created_at", "updated_at") VALUES
('st-0002', 'LPR 数据清理', '每小时清理过期 LPR 记录', 'interval', NULL, 3600, '2026-08-01 00:00:00+08', NULL, 'lpr_cleaner', '{"retention_days": 90}'::jsonb, 'paused', 'immediate', 2, 0, 1800, '2026-08-11 12:00:00+08', NULL, 'admin', '1', '2026-08-01 00:00:00+08', '2026-08-11 12:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 任务执行记录 (schedule-task)
INSERT INTO "public"."task_executions" ("id", "task_id", "status", "start_time", "end_time", "result", "error_message", "retry_count", "created_at") VALUES
('te-0001', 'st-0001', 'completed', '2026-08-12 01:00:00+08', '2026-08-12 01:01:30+08', '报表生成成功, 共 12 行', NULL, 0, '2026-08-12 01:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."task_executions" ("id", "task_id", "status", "start_time", "end_time", "result", "error_message", "retry_count", "created_at") VALUES
('te-0002', 'st-0002', 'failed', '2026-08-11 12:00:00+08', '2026-08-11 12:00:30+08', NULL, '数据库连接超时', 1, '2026-08-11 12:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 数据源 (report-service)
INSERT INTO "public"."data_sources" ("id", "name", "ds_type", "config", "created_at", "updated_at") VALUES
('ds-0001', '生产 PostgreSQL', 'postgresql', '{"host": "pg-cluster-postgresql-0", "port": 5432, "database": "myai"}'::jsonb, '2026-08-01 09:00:00+08', '2026-08-01 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."data_sources" ("id", "name", "ds_type", "config", "created_at", "updated_at") VALUES
('ds-0002', '报表缓存 Redis', 'redis', '{"host": "redis-replication-redis-0", "port": 6379, "db": 0}'::jsonb, '2026-08-02 10:00:00+08', '2026-08-02 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 操作日志 (audit-service)
INSERT INTO "public"."sys_operation_logs" ("id", "user_id", "username", "module", "business_type", "method", "request_method", "request_url", "request_params", "request_body", "response_data", "status", "error_msg", "execution_time", "ip_address", "created_at") VALUES
(1, 1, 'admin', '角色管理', 'update', 'update_role', 'PUT', '/api/admin/roles/2', '{"name": "系统管理员"}', '{"description": "系统管理权限"}', '{"code": 200}', 1, NULL, 45, '192.0.2.8', '2026-08-12 09:30:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."sys_operation_logs" ("id", "user_id", "username", "module", "business_type", "request_method", "request_url", "request_params", "request_body", "response_data", "status", "error_msg", "execution_time", "ip_address", "created_at") VALUES
(2, 1, 'admin', '用户管理', 'delete', 'DELETE', '/api/admin/users/99', NULL, NULL, '{"code": 404, "msg": "用户不存在"}', 0, '用户不存在', 12, '192.0.2.8', '2026-08-12 09:45:00+08')
ON CONFLICT (id) DO NOTHING;

-- 登录日志 (auth-service)
INSERT INTO "public"."sys_login_logs" ("id", "user_id", "username", "ip_address", "user_agent", "login_location", "login_status", "fail_reason", "login_type", "created_at") VALUES
(1, 1, 'admin', '192.0.2.8', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36', '深圳市', 1, NULL, 'password', '2026-08-12 09:00:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."sys_login_logs" ("id", "user_id", "username", "ip_address", "user_agent", "login_location", "login_status", "fail_reason", "login_type", "created_at") VALUES
(2, NULL, 'unknown', '203.0.113.7', 'curl/8.0', NULL, 0, '密码错误', 'password', '2026-08-12 08:55:00+08')
ON CONFLICT (id) DO NOTHING;

-- 敏感操作审计 (audit-service)
INSERT INTO "public"."sys_sensitive_audits" ("id", "user_id", "username", "operation_type", "operation_desc", "request_data", "ip_address", "confirm_status", "confirm_time", "confirmed_by", "created_at") VALUES
(1, 1, 'admin', 'delete_role', '删除角色: 测试角色', '{"role_id": 99}', '192.0.2.8', 1, '2026-08-12 09:50:00+08', 1, '2026-08-12 09:49:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."sys_sensitive_audits" ("id", "user_id", "username", "operation_type", "operation_desc", "request_data", "ip_address", "confirm_status", "confirm_time", "confirmed_by", "created_at") VALUES
(2, 1, 'admin', 'reset_password', '重置用户密码: user99', '{"user_id": 99}', '192.0.2.8', 0, NULL, NULL, '2026-08-12 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- 审计日志 (audit-service, 第二批新增表)
-- audit_logs.id 为 GENERATED ALWAYS AS IDENTITY, 需 OVERRIDING SYSTEM VALUE 显式插入
INSERT INTO "public"."audit_logs" ("id", "tenant_id", "user_id", "username", "action", "resource_type", "resource_id", "details", "ip_address", "created_at") OVERRIDING SYSTEM VALUE VALUES
(1, 1, 1, 'admin', 'ROLE_UPDATE', 'role', 2, '更新角色: 系统管理员', '192.0.2.8', '2026-08-12 09:30:00+08')
ON CONFLICT (id) DO NOTHING;

INSERT INTO "public"."audit_logs" ("id", "tenant_id", "user_id", "username", "action", "resource_type", "resource_id", "details", "ip_address", "created_at") OVERRIDING SYSTEM VALUE VALUES
(2, 1, NULL, 'system', 'API_KEY_CREATE', 'api_key', NULL, '创建 API 密钥', '192.0.2.9', '2026-08-12 10:30:00+08')
ON CONFLICT (id) DO NOTHING;

-- API 用量日志 (api-gateway, 第二批新增表)
-- api_usage_logs.id 为 GENERATED ALWAYS AS IDENTITY, 需 OVERRIDING SYSTEM VALUE 显式插入
INSERT INTO "public"."api_usage_logs" ("id", "tenant_id", "created_at") OVERRIDING SYSTEM VALUE VALUES
(1, 1, '2026-08-12 08:00:00+08'),
(2, 1, '2026-08-12 09:00:00+08'),
(3, 1, '2026-08-12 10:00:00+08')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Sequence setval
-- =============================================================================
SELECT setval('"public"."announcements_id_seq"', 2, true);
SELECT setval('"public"."api_key_usage_logs_id_seq"', 1, true);
SELECT setval('"public"."cms_article_id_seq"', 1, false);
SELECT setval('"public"."cms_category_id_seq"', 4, true);
SELECT setval('"public"."cms_comment_id_seq"', 1, false);
SELECT setval('"public"."departments_id_seq"', 6, true);
SELECT setval('"public"."permissions_id_seq"', 21, true);
SELECT setval('"public"."role_permissions_id_seq"', 40, true);
SELECT setval('"public"."role_templates_id_seq"', 1, false);
SELECT setval('"public"."roles_id_seq"', 4, true);
SELECT setval('"public"."sessions_id_seq"', 1, false);
SELECT setval('"public"."sys_feedback_id_seq"', 1, false);
SELECT setval('"public"."sys_files_id_seq"', 1, false);
SELECT setval('"public"."sys_ip_whitelist_id_seq"', 1, false);
SELECT setval('"public"."sys_login_device_id_seq"', 1, false);
SELECT setval('"public"."sys_login_logs_id_seq"', 1, false);
SELECT setval('"public"."sys_message_id_seq"', 1, false);
SELECT setval('"public"."sys_message_user_id_seq"', 1, false);
SELECT setval('"public"."sys_operation_logs_id_seq"', 1, false);
SELECT setval('"public"."sys_sensitive_audits_id_seq"', 1, false);
SELECT setval('"public"."system_configs_id_seq"', 11, true);
SELECT setval('"public"."user_departments_id_seq"', 1, false);
SELECT setval('"public"."user_roles_id_seq"', 2, true);
SELECT setval('"public"."users_id_seq"', 2, true);
SELECT setval('"public"."tow_car_id_seq"', 1, false);
SELECT setval('"public"."tow_car_class_id_seq"', 1, false);
SELECT setval('"public"."tow_car_type_id_seq"', 4, true);
SELECT setval('"public"."tow_car_color_id_seq"', 8, true);
SELECT setval('"public"."tow_dc_type_id_seq"', 4, true);
SELECT setval('"public"."tow_dc_causes_id_seq"', 7, true);
SELECT setval('"public"."message_templates_id_seq"', 3, true);

-- =============================================================================
-- Indexes (索引)
-- =============================================================================

-- users indexes
CREATE INDEX "idx_users_username" ON "public"."users" USING btree ("username" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_users_email" ON "public"."users" USING btree ("email" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_users_status" ON "public"."users" USING btree ("status" "pg_catalog"."int4_ops" ASC NULLS LAST);
CREATE INDEX "idx_users_role_status" ON "public"."users" USING btree ("role" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST, "status" "pg_catalog"."int4_ops" ASC NULLS LAST);
CREATE INDEX "idx_users_created_at" ON "public"."users" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- departments indexes
CREATE INDEX "idx_departments_code" ON "public"."departments" USING btree ("code" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_departments_parent_id" ON "public"."departments" USING btree ("parent_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_departments_status" ON "public"."departments" USING btree ("status" "pg_catalog"."int4_ops" ASC NULLS LAST);

-- sessions indexes
CREATE INDEX "idx_sessions_user_id" ON "public"."sessions" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sessions_expires_at" ON "public"."sessions" USING btree ("expires_at" "pg_catalog"."timestamp_ops" ASC NULLS LAST);

-- sys_login_logs indexes
CREATE INDEX "idx_sys_login_logs_user_id" ON "public"."sys_login_logs" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_logs_username" ON "public"."sys_login_logs" USING btree ("username" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_logs_ip_address" ON "public"."sys_login_logs" USING btree ("ip_address" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_logs_login_status" ON "public"."sys_login_logs" USING btree ("login_status" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_logs_created_at" ON "public"."sys_login_logs" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- sys_login_device indexes
CREATE INDEX "idx_sys_login_device_user_id" ON "public"."sys_login_device" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_device_device_id" ON "public"."sys_login_device" USING btree ("device_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_device_login_time" ON "public"."sys_login_device" USING btree ("login_time" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_login_device_is_active" ON "public"."sys_login_device" USING btree ("is_active" "pg_catalog"."int2_ops" ASC NULLS LAST);

-- user_roles indexes
CREATE INDEX "idx_user_roles_user_id" ON "public"."user_roles" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_user_roles_role_id" ON "public"."user_roles" USING btree ("role_id" "pg_catalog"."int8_ops" ASC NULLS LAST);

-- user_departments indexes
CREATE INDEX "idx_user_departments_user_id" ON "public"."user_departments" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_user_departments_department_id" ON "public"."user_departments" USING btree ("department_id" "pg_catalog"."int8_ops" ASC NULLS LAST);

-- roles indexes
CREATE INDEX "idx_roles_code" ON "public"."roles" USING btree ("code" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_roles_parent_id" ON "public"."roles" USING btree ("parent_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_roles_status" ON "public"."roles" USING btree ("status" "pg_catalog"."int4_ops" ASC NULLS LAST);

-- permissions indexes
CREATE INDEX "idx_permissions_code" ON "public"."permissions" USING btree ("code" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_permissions_parent_id" ON "public"."permissions" USING btree ("parent_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_permissions_type" ON "public"."permissions" USING btree ("permission_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- role_permissions indexes
CREATE INDEX "idx_role_permissions_role_id" ON "public"."role_permissions" USING btree ("role_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_role_permissions_permission_id" ON "public"."role_permissions" USING btree ("permission_id" "pg_catalog"."int8_ops" ASC NULLS LAST);

-- api_keys indexes
CREATE INDEX "idx_api_keys_key_id" ON "public"."api_keys" USING btree ("key_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_api_keys_user_id" ON "public"."api_keys" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_api_keys_tenant_id" ON "public"."api_keys" USING btree ("tenant_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_api_keys_status" ON "public"."api_keys" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_api_keys_created_at" ON "public"."api_keys" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- api_key_usage_logs indexes
CREATE INDEX "idx_api_key_usage_key_id" ON "public"."api_key_usage_logs" USING btree ("key_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_api_key_usage_created_at" ON "public"."api_key_usage_logs" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- cms_category indexes
CREATE INDEX "idx_cms_category_parent_id" ON "public"."cms_category" USING btree ("parent_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_category_status" ON "public"."cms_category" USING btree ("status" "pg_catalog"."int2_ops" ASC NULLS LAST);

-- cms_article indexes
CREATE INDEX "idx_cms_article_category_id" ON "public"."cms_article" USING btree ("category_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_article_author_id" ON "public"."cms_article" USING btree ("author_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_article_status" ON "public"."cms_article" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_article_is_top" ON "public"."cms_article" USING btree ("is_top" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_article_is_featured" ON "public"."cms_article" USING btree ("is_featured" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_article_published_at" ON "public"."cms_article" USING btree ("published_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_article_created_at" ON "public"."cms_article" USING btree ("created_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- cms_comment indexes
CREATE INDEX "idx_cms_comment_article_id" ON "public"."cms_comment" USING btree ("article_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_comment_user_id" ON "public"."cms_comment" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_comment_parent_id" ON "public"."cms_comment" USING btree ("parent_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_comment_status" ON "public"."cms_comment" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_cms_comment_created_at" ON "public"."cms_comment" USING btree ("created_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- sys_message indexes
CREATE INDEX "idx_sys_message_type" ON "public"."sys_message" USING btree ("type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_message_sender_id" ON "public"."sys_message" USING btree ("sender_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_message_priority" ON "public"."sys_message" USING btree ("priority" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_message_created_at" ON "public"."sys_message" USING btree ("created_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- sys_message_user indexes
CREATE INDEX "idx_sys_message_user_user_id" ON "public"."sys_message_user" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_message_user_is_read" ON "public"."sys_message_user" USING btree ("is_read" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_message_user_is_deleted" ON "public"."sys_message_user" USING btree ("is_deleted" "pg_catalog"."int2_ops" ASC NULLS LAST);

-- announcements indexes
CREATE INDEX "idx_announcements_active" ON "public"."announcements" USING btree ("is_active" "pg_catalog"."bool_ops" ASC NULLS LAST, "is_pinned" "pg_catalog"."bool_ops" ASC NULLS LAST, "start_time" "pg_catalog"."timestamptz_ops" ASC NULLS LAST, "end_time" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);
CREATE INDEX "idx_announcements_created_by" ON "public"."announcements" USING btree ("created_by" "pg_catalog"."int8_ops" ASC NULLS LAST);

-- sys_files indexes
CREATE INDEX "idx_sys_files_category" ON "public"."sys_files" USING btree ("category" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_files_created_by" ON "public"."sys_files" USING btree ("created_by" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_files_tenant_id" ON "public"."sys_files" USING btree ("tenant_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_files_mime_type" ON "public"."sys_files" USING btree ("mime_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_files_created_at" ON "public"."sys_files" USING btree ("created_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_files_deleted_at" ON "public"."sys_files" USING btree ("deleted_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- sys_feedback indexes
CREATE INDEX "idx_sys_feedback_user_id" ON "public"."sys_feedback" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_feedback_type" ON "public"."sys_feedback" USING btree ("type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_feedback_status" ON "public"."sys_feedback" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_feedback_created_at" ON "public"."sys_feedback" USING btree ("created_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- sys_operation_logs indexes
CREATE INDEX "idx_sys_operation_logs_user_id" ON "public"."sys_operation_logs" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_operation_logs_username" ON "public"."sys_operation_logs" USING btree ("username" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_operation_logs_module" ON "public"."sys_operation_logs" USING btree ("module" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_operation_logs_business_type" ON "public"."sys_operation_logs" USING btree ("business_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_operation_logs_status" ON "public"."sys_operation_logs" USING btree ("status" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_operation_logs_ip_address" ON "public"."sys_operation_logs" USING btree ("ip_address" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_operation_logs_created_at" ON "public"."sys_operation_logs" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);
CREATE INDEX "idx_sys_operation_logs_user_time" ON "public"."sys_operation_logs" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST, "created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- sys_sensitive_audits indexes
CREATE INDEX "idx_sys_sensitive_audits_user_id" ON "public"."sys_sensitive_audits" USING btree ("user_id" "pg_catalog"."int8_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_sensitive_audits_username" ON "public"."sys_sensitive_audits" USING btree ("username" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_sensitive_audits_type" ON "public"."sys_sensitive_audits" USING btree ("operation_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_sensitive_audits_status" ON "public"."sys_sensitive_audits" USING btree ("confirm_status" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_sensitive_audits_created_at" ON "public"."sys_sensitive_audits" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- sys_ip_whitelist indexes
CREATE INDEX "idx_sys_ip_whitelist_ip" ON "public"."sys_ip_whitelist" USING btree ("ip_address" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_ip_whitelist_enabled" ON "public"."sys_ip_whitelist" USING btree ("is_enabled" "pg_catalog"."int2_ops" ASC NULLS LAST);
CREATE INDEX "idx_sys_ip_whitelist_created_at" ON "public"."sys_ip_whitelist" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- workflows indexes
CREATE INDEX "idx_workflows_status" ON "public"."workflows" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflows_created_by" ON "public"."workflows" USING btree ("created_by" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflows_created_at" ON "public"."workflows" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- workflow_nodes indexes
CREATE INDEX "idx_workflow_nodes_workflow_id" ON "public"."workflow_nodes" USING btree ("workflow_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflow_nodes_type" ON "public"."workflow_nodes" USING btree ("node_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- workflow_edges indexes
CREATE INDEX "idx_workflow_edges_workflow_id" ON "public"."workflow_edges" USING btree ("workflow_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflow_edges_source" ON "public"."workflow_edges" USING btree ("source_node_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflow_edges_target" ON "public"."workflow_edges" USING btree ("target_node_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- workflow_instances indexes
CREATE INDEX "idx_workflow_instances_workflow_id" ON "public"."workflow_instances" USING btree ("workflow_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflow_instances_status" ON "public"."workflow_instances" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflow_instances_started_by" ON "public"."workflow_instances" USING btree ("started_by" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_workflow_instances_started_at" ON "public"."workflow_instances" USING btree ("started_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- task_records indexes
CREATE INDEX "idx_task_records_instance_id" ON "public"."task_records" USING btree ("instance_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_task_records_assignee" ON "public"."task_records" USING btree ("assignee" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_task_records_status" ON "public"."task_records" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_task_records_started_at" ON "public"."task_records" USING btree ("started_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- reports indexes
CREATE INDEX "idx_reports_status" ON "public"."reports" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_reports_report_type" ON "public"."reports" USING btree ("report_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_reports_created_by" ON "public"."reports" USING btree ("created_by" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_reports_created_at" ON "public"."reports" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- report_tasks indexes
CREATE INDEX "idx_report_tasks_report_id" ON "public"."report_tasks" USING btree ("report_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_report_tasks_status" ON "public"."report_tasks" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- schedule_tasks indexes
CREATE INDEX "idx_schedule_tasks_status" ON "public"."schedule_tasks" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_schedule_tasks_handler" ON "public"."schedule_tasks" USING btree ("task_handler" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_schedule_tasks_next_run" ON "public"."schedule_tasks" USING btree ("next_run_time" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);
CREATE INDEX "idx_schedule_tasks_tenant" ON "public"."schedule_tasks" USING btree ("tenant_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_schedule_tasks_created" ON "public"."schedule_tasks" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- task_executions indexes
CREATE INDEX "idx_task_executions_task" ON "public"."task_executions" USING btree ("task_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_task_executions_status" ON "public"."task_executions" USING btree ("status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_task_executions_created" ON "public"."task_executions" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- data_sources indexes
CREATE INDEX "idx_data_sources_name" ON "public"."data_sources" USING btree ("name" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_data_sources_ds_type" ON "public"."data_sources" USING btree ("ds_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- system_configs indexes
CREATE INDEX "idx_system_configs_category" ON "public"."system_configs" USING btree ("category" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_system_configs_key" ON "public"."system_configs" USING btree ("config_key" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- api_permissions indexes
CREATE INDEX "idx_ap_code" ON "public"."api_permissions" USING btree ("code" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_ap_module" ON "public"."api_permissions" USING btree ("module" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- role_function_permissions indexes
CREATE INDEX "idx_rfp_role_id" ON "public"."role_function_permissions" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- role_data_permissions indexes
CREATE INDEX "idx_rdp_role_id" ON "public"."role_data_permissions" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_rdp_resource_type" ON "public"."role_data_permissions" USING btree ("resource_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- role_field_permissions indexes
CREATE INDEX "idx_rfpp_role_id" ON "public"."role_field_permissions" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_rfpp_resource" ON "public"."role_field_permissions" USING btree ("resource_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- permission_inheritances indexes
CREATE INDEX "idx_inherit_parent" ON "public"."permission_inheritances" USING btree ("parent_role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_inherit_child" ON "public"."permission_inheritances" USING btree ("child_role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- permission_expiry indexes
CREATE INDEX "idx_permission_expiry_user_id" ON "public"."permission_expiry" USING btree ("user_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_permission_expiry_role_id" ON "public"."permission_expiry" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_permission_expiry_expires_at" ON "public"."permission_expiry" USING btree ("expires_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- permission_expiries indexes
CREATE INDEX "idx_expiry_user" ON "public"."permission_expiries" USING btree ("user_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_expiry_role_id" ON "public"."permission_expiries" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_expiry_expires" ON "public"."permission_expiries" USING btree ("expires_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST);

-- permission_change_logs indexes
CREATE INDEX "idx_pcl_role_name" ON "public"."permission_change_logs" USING btree ("role_name" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_pcl_operator" ON "public"."permission_change_logs" USING btree ("operator" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_pcl_change_type" ON "public"."permission_change_logs" USING btree ("change_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_pcl_created_at" ON "public"."permission_change_logs" USING btree ("created_at" "pg_catalog"."timestamptz_ops" DESC NULLS FIRST);

-- data_permission_rules indexes
CREATE INDEX "idx_data_perm_role" ON "public"."data_permission_rules" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_data_perm_resource" ON "public"."data_permission_rules" USING btree ("resource_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_data_perm_enabled" ON "public"."data_permission_rules" USING btree ("enabled" "pg_catalog"."bool_ops" ASC NULLS LAST);

-- field_permission_configs indexes
CREATE INDEX "idx_field_perm_role" ON "public"."field_permission_configs" USING btree ("role_id" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_field_perm_resource" ON "public"."field_permission_configs" USING btree ("resource_type" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_field_perm_field" ON "public"."field_permission_configs" USING btree ("field_name" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- tow_car indexes
CREATE INDEX "idx_tow_car_license" ON "public"."tow_car" USING btree ("license" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_tow_car_dc_date" ON "public"."tow_car" USING btree ("dc_date" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_tow_car_rs_date" ON "public"."tow_car" USING btree ("rs_date" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_tow_car_cmd_unit" ON "public"."tow_car" USING btree ("cmd_unit" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);
CREATE INDEX "idx_tow_car_delete" ON "public"."tow_car" USING btree ("delete" "pg_catalog"."bool_ops" ASC NULLS LAST);
CREATE INDEX "idx_tow_car_dc_key" ON "public"."tow_car" USING btree ("dc_key" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- tow_car_class indexes
CREATE INDEX "idx_tow_car_class_cpt" ON "public"."tow_car_class" USING btree ("cpt" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST);

-- tow_dc_causes indexes
CREATE INDEX "idx_tow_dc_causes_type_id" ON "public"."tow_dc_causes" USING btree ("type_id" "pg_catalog"."int8_ops" ASC NULLS LAST);

-- =============================================================================
-- Primary Keys (主键)
-- =============================================================================

ALTER TABLE "public"."users" ADD CONSTRAINT "users_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."departments" ADD CONSTRAINT "departments_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sessions" ADD CONSTRAINT "sessions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_login_logs" ADD CONSTRAINT "sys_login_logs_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_login_device" ADD CONSTRAINT "sys_login_device_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."user_roles" ADD CONSTRAINT "user_roles_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."user_departments" ADD CONSTRAINT "user_departments_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."roles" ADD CONSTRAINT "roles_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."permissions" ADD CONSTRAINT "permissions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."role_permissions" ADD CONSTRAINT "role_permissions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."role_templates" ADD CONSTRAINT "role_templates_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."api_permissions" ADD CONSTRAINT "api_permissions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."cms_category" ADD CONSTRAINT "cms_category_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."cms_article" ADD CONSTRAINT "cms_article_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."cms_comment" ADD CONSTRAINT "cms_comment_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_message" ADD CONSTRAINT "sys_message_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_message_user" ADD CONSTRAINT "sys_message_user_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."announcements" ADD CONSTRAINT "announcements_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_files" ADD CONSTRAINT "sys_files_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_feedback" ADD CONSTRAINT "sys_feedback_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_operation_logs" ADD CONSTRAINT "sys_operation_logs_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_sensitive_audits" ADD CONSTRAINT "sys_sensitive_audits_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."sys_ip_whitelist" ADD CONSTRAINT "sys_ip_whitelist_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."workflows" ADD CONSTRAINT "workflows_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."workflow_nodes" ADD CONSTRAINT "workflow_nodes_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."workflow_edges" ADD CONSTRAINT "workflow_edges_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."workflow_instances" ADD CONSTRAINT "workflow_instances_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."task_records" ADD CONSTRAINT "task_records_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."reports" ADD CONSTRAINT "reports_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."report_tasks" ADD CONSTRAINT "report_tasks_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."schedule_tasks" ADD CONSTRAINT "schedule_tasks_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."task_executions" ADD CONSTRAINT "task_executions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."data_sources" ADD CONSTRAINT "data_sources_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."api_keys" ADD CONSTRAINT "api_keys_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."api_key_usage_logs" ADD CONSTRAINT "api_key_usage_logs_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."system_configs" ADD CONSTRAINT "system_configs_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."role_function_permissions" ADD CONSTRAINT "role_function_permissions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."role_data_permissions" ADD CONSTRAINT "role_data_permissions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."role_field_permissions" ADD CONSTRAINT "role_field_permissions_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."permission_inheritances" ADD CONSTRAINT "permission_inheritances_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."permission_expiry" ADD CONSTRAINT "permission_expiry_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."permission_expiries" ADD CONSTRAINT "permission_expiries_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."permission_change_logs" ADD CONSTRAINT "permission_change_logs_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."data_permission_rules" ADD CONSTRAINT "data_permission_rules_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."field_permission_configs" ADD CONSTRAINT "field_permission_configs_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."tow_car" ADD CONSTRAINT "tow_car_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."tow_car_class" ADD CONSTRAINT "tow_car_class_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."tow_car_type" ADD CONSTRAINT "tow_car_type_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."tow_car_color" ADD CONSTRAINT "tow_car_color_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."tow_dc_type" ADD CONSTRAINT "tow_dc_type_pkey" PRIMARY KEY ("id");
ALTER TABLE "public"."tow_dc_causes" ADD CONSTRAINT "tow_dc_causes_pkey" PRIMARY KEY ("id");

-- =============================================================================
-- Unique Constraints (唯一约束)
-- =============================================================================

ALTER TABLE "public"."users" ADD CONSTRAINT "users_username_key" UNIQUE ("username");
ALTER TABLE "public"."departments" ADD CONSTRAINT "departments_code_key" UNIQUE ("code");
ALTER TABLE "public"."roles" ADD CONSTRAINT "roles_name_key" UNIQUE ("name");
ALTER TABLE "public"."roles" ADD CONSTRAINT "roles_code_key" UNIQUE ("code");
ALTER TABLE "public"."permissions" ADD CONSTRAINT "permissions_code_key" UNIQUE ("code");
ALTER TABLE "public"."api_keys" ADD CONSTRAINT "api_keys_key_id_key" UNIQUE ("key_id");
ALTER TABLE "public"."cms_category" ADD CONSTRAINT "cms_category_slug_key" UNIQUE ("slug");
ALTER TABLE "public"."cms_article" ADD CONSTRAINT "cms_article_slug_key" UNIQUE ("slug");
ALTER TABLE "public"."sys_message_user" ADD CONSTRAINT "sys_message_user_message_id_user_id_key" UNIQUE ("message_id", "user_id");
ALTER TABLE "public"."user_roles" ADD CONSTRAINT "user_roles_user_id_role_id_key" UNIQUE ("user_id", "role_id");
ALTER TABLE "public"."user_departments" ADD CONSTRAINT "user_departments_user_id_department_id_key" UNIQUE ("user_id", "department_id");
ALTER TABLE "public"."role_permissions" ADD CONSTRAINT "role_permissions_role_id_permission_id_key" UNIQUE ("role_id", "permission_id");
ALTER TABLE "public"."role_function_permissions" ADD CONSTRAINT "role_function_permissions_role_id_permission_key" UNIQUE ("role_id", "permission");
ALTER TABLE "public"."role_field_permissions" ADD CONSTRAINT "role_field_permissions_role_id_resource_type_field_name_key" UNIQUE ("role_id", "resource_type", "field_name");
ALTER TABLE "public"."api_permissions" ADD CONSTRAINT "api_permissions_code_key" UNIQUE ("code");
ALTER TABLE "public"."system_configs" ADD CONSTRAINT "system_configs_category_config_key_key" UNIQUE ("category", "config_key");

-- =============================================================================
-- Check Constraints (检查约束)
-- =============================================================================

ALTER TABLE "public"."departments" ADD CONSTRAINT "chk_dept_level" CHECK (level <= 5);
ALTER TABLE "public"."roles" ADD CONSTRAINT "chk_role_level" CHECK (level <= 3);

-- =============================================================================
-- Foreign Keys (外键)
-- =============================================================================

-- user-service foreign keys
ALTER TABLE "public"."sessions" ADD CONSTRAINT "sessions_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."departments" ADD CONSTRAINT "departments_parent_id_fkey" FOREIGN KEY ("parent_id") REFERENCES "public"."departments" ("id") ON DELETE SET NULL ON UPDATE NO ACTION;
ALTER TABLE "public"."departments" ADD CONSTRAINT "departments_leader_id_fkey" FOREIGN KEY ("leader_id") REFERENCES "public"."users" ("id") ON DELETE SET NULL ON UPDATE NO ACTION;
ALTER TABLE "public"."user_roles" ADD CONSTRAINT "user_roles_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."user_roles" ADD CONSTRAINT "user_roles_role_id_fkey" FOREIGN KEY ("role_id") REFERENCES "public"."roles" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."user_departments" ADD CONSTRAINT "user_departments_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."user_departments" ADD CONSTRAINT "user_departments_department_id_fkey" FOREIGN KEY ("department_id") REFERENCES "public"."departments" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;

-- auth-service foreign keys
ALTER TABLE "public"."permissions" ADD CONSTRAINT "permissions_parent_id_fkey" FOREIGN KEY ("parent_id") REFERENCES "public"."permissions" ("id") ON DELETE SET NULL ON UPDATE NO ACTION;
ALTER TABLE "public"."role_permissions" ADD CONSTRAINT "role_permissions_role_id_fkey" FOREIGN KEY ("role_id") REFERENCES "public"."roles" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."role_permissions" ADD CONSTRAINT "role_permissions_permission_id_fkey" FOREIGN KEY ("permission_id") REFERENCES "public"."permissions" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."roles" ADD CONSTRAINT "roles_parent_id_fkey" FOREIGN KEY ("parent_id") REFERENCES "public"."roles" ("id") ON DELETE SET NULL ON UPDATE NO ACTION;

-- cms-service foreign keys
ALTER TABLE "public"."cms_article" ADD CONSTRAINT "cms_article_category_id_fkey" FOREIGN KEY ("category_id") REFERENCES "public"."cms_category" ("id") ON DELETE RESTRICT ON UPDATE NO ACTION;
ALTER TABLE "public"."cms_article" ADD CONSTRAINT "cms_article_author_id_fkey" FOREIGN KEY ("author_id") REFERENCES "public"."users" ("id") ON DELETE RESTRICT ON UPDATE NO ACTION;
ALTER TABLE "public"."cms_comment" ADD CONSTRAINT "cms_comment_article_id_fkey" FOREIGN KEY ("article_id") REFERENCES "public"."cms_article" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."cms_comment" ADD CONSTRAINT "cms_comment_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON DELETE RESTRICT ON UPDATE NO ACTION;

-- messaging-service foreign keys
ALTER TABLE "public"."sys_message" ADD CONSTRAINT "sys_message_sender_id_fkey" FOREIGN KEY ("sender_id") REFERENCES "public"."users" ("id") ON DELETE SET NULL ON UPDATE NO ACTION;
ALTER TABLE "public"."sys_message_user" ADD CONSTRAINT "sys_message_user_message_id_fkey" FOREIGN KEY ("message_id") REFERENCES "public"."sys_message" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."sys_message_user" ADD CONSTRAINT "sys_message_user_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."announcements" ADD CONSTRAINT "announcements_created_by_fkey" FOREIGN KEY ("created_by") REFERENCES "public"."users" ("id") ON DELETE SET NULL ON UPDATE NO ACTION;

-- workflow-service foreign keys
ALTER TABLE "public"."workflow_nodes" ADD CONSTRAINT "workflow_nodes_workflow_id_fkey" FOREIGN KEY ("workflow_id") REFERENCES "public"."workflows" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."workflow_edges" ADD CONSTRAINT "workflow_edges_workflow_id_fkey" FOREIGN KEY ("workflow_id") REFERENCES "public"."workflows" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."workflow_instances" ADD CONSTRAINT "workflow_instances_workflow_id_fkey" FOREIGN KEY ("workflow_id") REFERENCES "public"."workflows" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "public"."task_records" ADD CONSTRAINT "task_records_instance_id_fkey" FOREIGN KEY ("instance_id") REFERENCES "public"."workflow_instances" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;

-- clean-service foreign keys
ALTER TABLE "public"."report_tasks" ADD CONSTRAINT "report_tasks_report_id_fkey" FOREIGN KEY ("report_id") REFERENCES "public"."reports" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;

-- =============================================================================
-- Multi-Tenant Schema Migration Helpers
-- =============================================================================
-- These helpers move tables from the default 'public' schema into
-- per-service schemas for schema-based isolation mode.
--
-- Usage:
--   SELECT migrate_service_tables('user-service', 'user_service');
--   SELECT migrate_service_tables('auth-service', 'auth_service');
--
-- This moves all tables belonging to a service into its dedicated schema
-- and updates their foreign key references.

CREATE OR REPLACE FUNCTION migrate_service_tables(
  service_name text,
  target_schema text
) RETURNS TABLE(tbl text, status text) AS $$
DECLARE
  tbl_name text;
  rec record;
BEGIN
  FOR rec IN
    -- Move tables owned by this service to the target schema
    SELECT unnest(
      CASE service_name
        WHEN 'user-service' THEN ARRAY['users','departments','sessions','sys_login_logs','sys_login_device','user_roles','user_departments']
        WHEN 'auth-service' THEN ARRAY['roles','permissions','role_permissions','role_function_permissions','role_data_permissions','role_field_permissions','role_templates','permission_inheritances','permission_expiry','permission_expiries','permission_change_logs','data_permission_rules','field_permission_configs','api_permissions']
        WHEN 'cms-service' THEN ARRAY['cms_article','cms_category','cms_comment']
        WHEN 'messaging-service' THEN ARRAY['sys_message','sys_message_user','announcements']
        WHEN 'file-service' THEN ARRAY['sys_files']
        WHEN 'feedback-service' THEN ARRAY['sys_feedback']
        WHEN 'workflow-service' THEN ARRAY['workflows','workflow_nodes','workflow_edges','workflow_instances','task_records']
        WHEN 'audit-service' THEN ARRAY['sys_operation_logs','sys_sensitive_audits','sys_ip_whitelist']
        WHEN 'clean-service' THEN ARRAY['reports','report_tasks','schedule_tasks','task_executions','data_sources']
        WHEN 'hik-service' THEN ARRAY['data_sources']
        WHEN 'api-key-service' THEN ARRAY['api_keys','api_key_usage_logs']
        WHEN 'tow-service' THEN ARRAY['tow_car','tow_car_class','tow_car_type','tow_car_color','tow_dc_type','tow_dc_causes']
        WHEN 'ebike-service' THEN ARRAY['users','car','car_history_2021_6','storage','storage_history_2021_6','order_2021_6','options']
        WHEN 'tenant-service' THEN ARRAY[]::text[]
        WHEN 'pay-service' THEN ARRAY[]::text[]
        ELSE ARRAY[]::text[]
      END
    ) AS tbl
  LOOP
    tbl_name := rec.tbl;

    -- Check if table exists in public schema
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = tbl_name) THEN
      -- Move sequences first
      EXECUTE format(
        'DO $$ BEGIN
          IF EXISTS (SELECT 1 FROM information_schema.sequences WHERE sequence_schema = ''public'' AND sequence_name = ''%s_id_seq'') THEN
            ALTER SEQUENCE public."%s_id_seq" SET SCHEMA %I;
          END IF;
        END $$',
        tbl_name, tbl_name, target_schema
      );

      -- Move the table
      EXECUTE format('ALTER TABLE public."%s" SET SCHEMA %I', tbl_name, target_schema);
      RETURN QUERY SELECT tbl_name, 'migrated'::text;
    ELSE
      RETURN QUERY SELECT tbl_name, 'not_found'::text;
    END IF;
  END LOOP;
END;
$$ LANGUAGE plpgsql;

-- ===========================================================================
-- LPR 车牌识别通行记录表
-- 记录 Vz 等车牌识别相机推送的车辆通行事件
-- ===========================================================================

CREATE TABLE IF NOT EXISTS public.lpr_pass_records (
    id               BIGSERIAL PRIMARY KEY,
    plate_no         VARCHAR(20)  NOT NULL,
    plate_color      VARCHAR(10)  NOT NULL DEFAULT '',
    plate_type       VARCHAR(20)  NOT NULL DEFAULT '',
    vehicle_type     VARCHAR(20)  NOT NULL DEFAULT '',
    device_id        VARCHAR(64)  NOT NULL DEFAULT '',
    device_name      VARCHAR(128) NOT NULL DEFAULT '',
    park_code        VARCHAR(64)  NOT NULL DEFAULT '',
    lane_code        VARCHAR(32)  NOT NULL DEFAULT '',
    direction        VARCHAR(10)  NOT NULL DEFAULT 'entry',
    pass_time        TIMESTAMP    NOT NULL DEFAULT NOW(),
    image_url        TEXT         NOT NULL DEFAULT '',
    confidence       REAL         NOT NULL DEFAULT 0.0,
    status           VARCHAR(20)  NOT NULL DEFAULT 'pending',
    related_order_id VARCHAR(64)  NOT NULL DEFAULT '',
    remark           TEXT         NOT NULL DEFAULT '',
    created_at       TIMESTAMP    NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMP    NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_lpr_pass_records_plate_no   ON public.lpr_pass_records(plate_no);
CREATE INDEX IF NOT EXISTS idx_lpr_pass_records_park_code  ON public.lpr_pass_records(park_code);
CREATE INDEX IF NOT EXISTS idx_lpr_pass_records_pass_time  ON public.lpr_pass_records(pass_time);
CREATE INDEX IF NOT EXISTS idx_lpr_pass_records_device_id  ON public.lpr_pass_records(device_id);
CREATE INDEX IF NOT EXISTS idx_lpr_pass_records_direction  ON public.lpr_pass_records(direction);

CREATE OR REPLACE FUNCTION update_lpr_pass_records_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_lpr_pass_records_updated_at ON public.lpr_pass_records;
CREATE TRIGGER trg_lpr_pass_records_updated_at
    BEFORE UPDATE ON public.lpr_pass_records
    FOR EACH ROW
    EXECUTE FUNCTION update_lpr_pass_records_updated_at();

-- ===========================================================================
-- SocialOps 社交媒体运营模块
-- 社交账号管理、内容管理、抓取/洗文/发布/统计
-- ===========================================================================

CREATE SCHEMA IF NOT EXISTS socialops;

-- 1. 社交账号
CREATE TABLE IF NOT EXISTS socialops.social_accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID,
    platform        VARCHAR(32) NOT NULL,
    account_name    VARCHAR(255) NOT NULL,
    account_id      VARCHAR(255),
    avatar_url      TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    config_json     JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS socialops.platform_credentials (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES socialops.social_accounts(id) ON DELETE CASCADE,
    credential_type VARCHAR(32) NOT NULL,
    encrypted_data  TEXT NOT NULL,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. 内容管理
CREATE TABLE IF NOT EXISTS socialops.content_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type     VARCHAR(16) NOT NULL DEFAULT 'manual',
    content_type    VARCHAR(16) NOT NULL DEFAULT 'article',
    title           VARCHAR(512),
    summary         TEXT,
    body            TEXT,
    source_url      TEXT,
    source_hash     VARCHAR(64),
    author_name     VARCHAR(255),
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
    raw_data        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_socialops_content_source_hash ON socialops.content_items(source_hash);
CREATE INDEX IF NOT EXISTS idx_socialops_content_status ON socialops.content_items(status);

CREATE TABLE IF NOT EXISTS socialops.content_tags (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id) ON DELETE CASCADE,
    tag_name        VARCHAR(128) NOT NULL,
    category        VARCHAR(64)
);

CREATE TABLE IF NOT EXISTS socialops.content_media (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id) ON DELETE CASCADE,
    media_type      VARCHAR(16) NOT NULL,
    original_url    TEXT NOT NULL,
    storage_path    TEXT,
    file_size       BIGINT,
    mime_type       VARCHAR(128),
    width           INT,
    height          INT,
    duration_secs   INT,
    thumbnail_url   TEXT,
    download_status VARCHAR(32) DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. 抓取模块
CREATE TABLE IF NOT EXISTS socialops.crawl_sources (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID,
    platform        VARCHAR(32) NOT NULL,
    source_name     VARCHAR(255),
    source_config   JSONB NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    crawl_interval  INT NOT NULL DEFAULT 3600,
    last_crawled_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS socialops.crawl_tasks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id       UUID NOT NULL REFERENCES socialops.crawl_sources(id),
    status          VARCHAR(32) NOT NULL DEFAULT 'pending',
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    items_found     INT DEFAULT 0,
    items_new       INT DEFAULT 0,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS socialops.crawl_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id         UUID NOT NULL REFERENCES socialops.crawl_tasks(id) ON DELETE CASCADE,
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id) ON DELETE CASCADE,
    is_new          BOOLEAN NOT NULL DEFAULT true,
    platform_id     VARCHAR(255),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 4. 洗文模块
CREATE TABLE IF NOT EXISTS socialops.llm_providers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID,
    provider_name   VARCHAR(128) NOT NULL,
    api_endpoint    TEXT NOT NULL,
    api_key_enc     TEXT NOT NULL,
    model_name      VARCHAR(255),
    default_params  JSONB,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS socialops.rewrite_tasks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID,
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id),
    llm_provider_id UUID NOT NULL REFERENCES socialops.llm_providers(id),
    rewrite_prompt  TEXT,
    status          VARCHAR(32) NOT NULL DEFAULT 'pending',
    target_count    INT NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS socialops.rewrite_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id         UUID NOT NULL REFERENCES socialops.rewrite_tasks(id) ON DELETE CASCADE,
    version_seq     INT NOT NULL,
    rewritten_title VARCHAR(512),
    rewritten_body  TEXT,
    similarity_score FLOAT,
    llm_raw_response JSONB,
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5. 发布模块
CREATE TABLE IF NOT EXISTS socialops.publish_tasks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID,
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id),
    target_account  UUID NOT NULL REFERENCES socialops.social_accounts(id),
    platform_post_id VARCHAR(255),
    status          VARCHAR(32) NOT NULL DEFAULT 'pending',
    publish_mode    VARCHAR(16) NOT NULL DEFAULT 'manual',
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS socialops.publish_schedules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID,
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id),
    target_account  UUID NOT NULL REFERENCES socialops.social_accounts(id),
    rewrite_version_id UUID REFERENCES socialops.rewrite_versions(id),
    cron_expression VARCHAR(128),
    scheduled_at    TIMESTAMPTZ,
    is_recurring    BOOLEAN NOT NULL DEFAULT false,
    status          VARCHAR(32) NOT NULL DEFAULT 'active',
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS socialops.publish_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id         UUID NOT NULL REFERENCES socialops.publish_tasks(id) ON DELETE CASCADE,
    platform        VARCHAR(32) NOT NULL,
    platform_post_id VARCHAR(255),
    post_url        TEXT,
    status_code     INT,
    response_body   JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 6. 统计模块
CREATE TABLE IF NOT EXISTS socialops.platform_stats (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES socialops.social_accounts(id) ON DELETE CASCADE,
    stats_date      DATE NOT NULL,
    followers_count     INT DEFAULT 0,
    following_count     INT DEFAULT 0,
    total_posts         INT DEFAULT 0,
    total_likes         BIGINT DEFAULT 0,
    total_comments      BIGINT DEFAULT 0,
    total_shares        BIGINT DEFAULT 0,
    total_views         BIGINT DEFAULT 0,
    raw_data            JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(account_id, stats_date)
);

CREATE TABLE IF NOT EXISTS socialops.content_stats (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id      UUID NOT NULL REFERENCES socialops.content_items(id) ON DELETE CASCADE,
    account_id      UUID NOT NULL REFERENCES socialops.social_accounts(id),
    platform_post_id VARCHAR(255),
    stats_date      DATE NOT NULL,
    views           INT DEFAULT 0,
    likes           INT DEFAULT 0,
    comments        INT DEFAULT 0,
    shares          INT DEFAULT 0,
    favorites       INT DEFAULT 0,
    avg_read_duration_sec INT,
    raw_data        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(content_id, account_id, stats_date)
);

CREATE TABLE IF NOT EXISTS socialops.stat_insights (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES socialops.social_accounts(id) ON DELETE CASCADE,
    insight_type    VARCHAR(32) NOT NULL,
    period_start    DATE NOT NULL,
    period_end      DATE NOT NULL,
    title           VARCHAR(255),
    summary         TEXT,
    analysis_body   TEXT,
    suggestions     JSONB,
    llm_provider_id UUID REFERENCES socialops.llm_providers(id),
    llm_raw_response JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
-- 漂移补齐表 (2026-08-12 审计对齐)
-- 线上库存在但 schema.sql 缺失的表, 按线上 \d 实采结构补齐。
-- 全部使用 CREATE TABLE IF NOT EXISTS, 幂等可重入。
-- =============================================================================

-- 租户表 (tenant-service)
CREATE TABLE IF NOT EXISTS "public"."tenants" (
    "id" bigserial PRIMARY KEY,
    "name" varchar(100) NOT NULL,
    "code" varchar(50) NOT NULL,
    "domain" varchar(200),
    "description" text,
    "max_users" integer DEFAULT 100,
    "max_storage" bigint DEFAULT 1073741824,
    "status" integer DEFAULT 1,
    "expires_at" timestamptz,
    "created_at" timestamptz DEFAULT now(),
    "updated_at" timestamptz DEFAULT now(),
    CONSTRAINT "tenants_code_key" UNIQUE ("code")
);
COMMENT ON TABLE "public"."tenants" IS '租户表 - 多租户管理';

-- 租户用户关联表 (tenant-service)
CREATE TABLE IF NOT EXISTS "public"."tenant_users" (
    "id" bigserial PRIMARY KEY,
    "tenant_id" bigint NOT NULL,
    "user_id" bigint NOT NULL,
    "role" varchar(50) NOT NULL DEFAULT '',
    "department" varchar(100),
    "position" varchar(100),
    "status" integer NOT NULL DEFAULT 1,
    "joined_at" timestamptz NOT NULL DEFAULT now(),
    "created_at" timestamptz DEFAULT now(),
    CONSTRAINT "tenant_users_tenant_id_user_id_key" UNIQUE ("tenant_id", "user_id")
);
COMMENT ON TABLE "public"."tenant_users" IS '租户用户关联表';

-- 反馈表 (feedback-service)
CREATE TABLE IF NOT EXISTS "public"."feedbacks" (
    "id" bigserial PRIMARY KEY,
    "user_id" bigint NOT NULL,
    "type" varchar(30) NOT NULL,
    "title" varchar(200) NOT NULL,
    "content" text NOT NULL,
    "contact" varchar(100),
    "status" varchar(20) NOT NULL DEFAULT 'pending',
    "handler_id" bigint,
    "handler_name" varchar(100),
    "handler_reply" text,
    "handler_time" timestamptz,
    "rating" smallint,
    "created_at" timestamptz DEFAULT now(),
    "updated_at" timestamptz DEFAULT now(),
    "deleted_at" timestamptz
);
COMMENT ON TABLE "public"."feedbacks" IS '用户反馈表 - 意见反馈与处理';

-- 租户文件统计表 (tenant-service)
CREATE TABLE IF NOT EXISTS "public"."files" (
    "id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    "file_size" bigint NOT NULL DEFAULT 0,
    "tenant_id" bigint NOT NULL DEFAULT 0,
    "created_at" timestamptz NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."files" IS '租户文件统计表';

-- 支付表 (pay-service)
CREATE TABLE IF NOT EXISTS "public"."pay" (
    "id" varchar(64) PRIMARY KEY,
    "order" varchar(64) NOT NULL,
    "status" varchar(20) DEFAULT 'generate',
    "type" varchar(20),
    "order_pay" jsonb,
    "amount" numeric,
    "remark" text,
    "create_service" varchar(50),
    "create_params" jsonb,
    "create_date" timestamptz DEFAULT CURRENT_TIMESTAMP,
    "update_date" timestamptz DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_pay_order ON "public"."pay" ("order");
CREATE INDEX IF NOT EXISTS idx_pay_status ON "public"."pay" (status);
CREATE INDEX IF NOT EXISTS idx_pay_type ON "public"."pay" (type);
COMMENT ON TABLE "public"."pay" IS '支付单表 - 多渠道支付记录';

-- 环卫发票表 (clean-service)
CREATE TABLE IF NOT EXISTS "public"."clean_invoice" (
    "id" varchar(64) PRIMARY KEY,
    "no" varchar(64),
    "imposing_no" bigint,
    "imposing_name" varchar(100),
    "fingerprint" varchar(100),
    "zone" varchar(100),
    "payer" varchar(100),
    "sum" varchar(100),
    "sum_capital" varchar(200),
    "remark" text,
    "review" varchar(100),
    "operator" varchar(100),
    "create_user" varchar(100),
    "create_date" varchar(50),
    "update_user" varchar(100),
    "update_date" varchar(50),
    "collection_name" varchar(100),
    "print" integer,
    "project" jsonb,
    "invalid" boolean,
    "delete" boolean
);
COMMENT ON TABLE "public"."clean_invoice" IS '环卫收费发票表 (clean-service)';

-- 环卫缴费信息表 (clean-service)
CREATE TABLE IF NOT EXISTS "public"."sf_payment_info" (
    "id" varchar(64) PRIMARY KEY,
    "no" varchar(64),
    "imposing_no" varchar(64),
    "imposing_name" varchar(100),
    "fingerprint" varchar(100),
    "zone" varchar(100),
    "payer" varchar(100),
    "project_code" varchar(100),
    "project_name" varchar(200),
    "unit" varchar(100),
    "num" varchar(100),
    "criteria" varchar(100),
    "sum" varchar(100),
    "sum_capital" varchar(200),
    "remark" text,
    "review" varchar(100),
    "operator" varchar(100),
    "collection_name" varchar(100),
    "print" smallint,
    "project" jsonb,
    "invalid" boolean,
    "create_user" varchar(100) NOT NULL DEFAULT '',
    "create_date" varchar(50) NOT NULL DEFAULT '',
    "update_user" varchar(100),
    "update_date" varchar(50),
    "delete" boolean,
    "serial_number" varchar(100)
);
COMMENT ON TABLE "public"."sf_payment_info" IS '环卫缴费信息表 (clean-service)';

-- 环卫正式票据表 (clean-service)
CREATE TABLE IF NOT EXISTS "public"."sf_formal_bill" (
    "id" varchar(64) PRIMARY KEY,
    "create_user" varchar(100),
    "create_date" varchar(50),
    "update_user" varchar(100),
    "update_date" varchar(50),
    "numbering" varchar(100),
    "name" varchar(200),
    "address" varchar(500),
    "principal" varchar(100),
    "telephone" varchar(50),
    "bill_year" varchar(10),
    "bill_month" varchar(10),
    "amount" varchar(100),
    "price" varchar(100),
    "money" bigint,
    "status" varchar(20),
    "fzr_id" varchar(100),
    "sys_org_code" varchar(100),
    "sys_company_code" varchar(100)
);
COMMENT ON TABLE "public"."sf_formal_bill" IS '环卫正式票据表 (clean-service)';

-- 环卫线上缴费表 (clean-service)
CREATE TABLE IF NOT EXISTS "public"."sf_payment_web" (
    "id" varchar(64) PRIMARY KEY,
    "create_user" varchar(100) NOT NULL DEFAULT '',
    "create_date" varchar(50) NOT NULL DEFAULT '',
    "update_user" varchar(100),
    "update_date" varchar(50),
    "orderform_code" varchar(100),
    "payment_time" varchar(50),
    "payment_amount" bigint,
    "payment_type" varchar(50),
    "numbering" varchar(100),
    "name" varchar(200),
    "principal" varchar(100),
    "telephone" varchar(50),
    "address" varchar(500),
    "receipt_status" varchar(50),
    "status" varchar(20) NOT NULL DEFAULT '',
    "openid" varchar(200),
    "type" varchar(50),
    "receipt_number" varchar(100),
    "serial_number" varchar(100)
);
COMMENT ON TABLE "public"."sf_payment_web" IS '环卫线上缴费表 (clean-service)';

-- 审计日志表 (audit-service)
CREATE TABLE IF NOT EXISTS "public"."audit_logs" (
    "id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    "tenant_id" bigint NOT NULL,
    "user_id" bigint,
    "username" varchar(100),
    "action" varchar(100) NOT NULL,
    "resource_type" varchar(50),
    "resource_id" bigint,
    "details" text,
    "ip_address" varchar(50),
    "created_at" timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON "public"."audit_logs" (action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON "public"."audit_logs" (created_at);
COMMENT ON TABLE "public"."audit_logs" IS '审计日志表 (audit-service)';

-- API 用量日志表 (api-gateway)
CREATE TABLE IF NOT EXISTS "public"."api_usage_logs" (
    "id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    "tenant_id" bigint NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now()
);
COMMENT ON TABLE "public"."api_usage_logs" IS 'API 用量日志表 (api-gateway)';

-- 电动自行车车辆历史表（2026年8月快照）
CREATE TABLE IF NOT EXISTS "public"."car_history_2026_8" (
    "id" varchar(255) NOT NULL,
    "code" varchar(255),
    "status" bigint,
    "provide" varchar(255),
    "speed" double precision,
    "gps" json,
    "time" json,
    "create_date" timestamp without time zone,
    "update_date" timestamp without time zone,
    "delete" boolean,
    "alert" varchar(255),
    "remark" varchar(255),
    "gps_type" bigint,
    "type" bigint,
    PRIMARY KEY ("id")
);
CREATE INDEX IF NOT EXISTS idx_car_h_2026_8_code ON "public"."car_history_2026_8" (code);
COMMENT ON TABLE "public"."car_history_2026_8" IS '电动自行车车辆历史表（2026年8月快照）';

-- 电动自行车存储历史表（2026年8月快照）
CREATE TABLE IF NOT EXISTS "public"."storage_history_2026_8" (
    "id" varchar(255) NOT NULL,
    "code" varchar(255),
    "status" bigint,
    "provide" varchar(255),
    "gps" json,
    "create_date" timestamp without time zone,
    "update_date" timestamp without time zone,
    "delete" boolean,
    "alert" varchar(255),
    "remark" varchar(255),
    "sum" bigint,
    "cur" bigint,
    "points" varchar(255),
    "gps_type" bigint,
    "type" bigint,
    PRIMARY KEY ("id")
);
COMMENT ON TABLE "public"."storage_history_2026_8" IS '电动自行车存储历史表（2026年8月快照）';

-- 电动自行车订单表（2026年8月快照）
CREATE TABLE IF NOT EXISTS "public"."order_2026_8" (
    "id" varchar(255) NOT NULL,
    "code" varchar(255),
    "status" bigint,
    "provide" varchar(255),
    "speed" double precision,
    "gps" json,
    "time" json,
    "create_date" timestamp without time zone,
    "update_date" timestamp without time zone,
    "delete" boolean,
    "alert" varchar(255),
    "remark" varchar(255),
    "gps_type" bigint,
    "hash" varchar(255) NOT NULL,
    "payable" double precision,
    "pay" double precision,
    "refund" double precision,
    "coupon" double precision,
    "order" varchar(255),
    "pay_type" bigint,
    "pay_time" timestamp without time zone,
    "pay_status" bigint,
    "paytype" bigint,
    "paytime" timestamp without time zone,
    "type" bigint,
    "paystatus" bigint,
    PRIMARY KEY ("hash")
);
CREATE INDEX IF NOT EXISTS idx_order_h_2026_8_code ON "public"."order_2026_8" (code);
CREATE INDEX IF NOT EXISTS idx_order_h_2026_8_provide ON "public"."order_2026_8" (provide);
COMMENT ON TABLE "public"."order_2026_8" IS '电动自行车订单表（2026年8月快照）';

-- 表级权限授予应用角色
ALTER TABLE "public"."tenants" OWNER TO myai_app;
ALTER TABLE "public"."tenant_users" OWNER TO myai_app;
ALTER TABLE "public"."feedbacks" OWNER TO myai_app;
ALTER TABLE "public"."files" OWNER TO myai_app;
ALTER TABLE "public"."pay" OWNER TO myai_app;
ALTER TABLE "public"."clean_invoice" OWNER TO myai_app;
ALTER TABLE "public"."sf_payment_info" OWNER TO myai_app;
ALTER TABLE "public"."sf_formal_bill" OWNER TO myai_app;
ALTER TABLE "public"."sf_payment_web" OWNER TO myai_app;
ALTER TABLE "public"."audit_logs" OWNER TO myai_app;
ALTER TABLE "public"."api_usage_logs" OWNER TO myai_app;
ALTER TABLE "public"."car_history_2026_8" OWNER TO myai_app;
ALTER TABLE "public"."storage_history_2026_8" OWNER TO myai_app;
ALTER TABLE "public"."order_2026_8" OWNER TO myai_app;

-- =============================================================================
-- End of schema
-- =============================================================================

-- Migration framework: use sqlx::migrate for incremental schema updates
-- Migrations directory: Backend/sql/migrations/

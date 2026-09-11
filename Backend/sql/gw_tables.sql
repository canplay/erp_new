-- =============================================================================
-- Gateway 自有表（api-gateway 管理）
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."gw_login_devices" (
    "id" int8 NOT NULL DEFAULT nextval('gw_login_devices_id_seq'::regclass),
    "user_id" int8,
    "device_id" varchar(255) NOT NULL,
    "device_type" varchar(50),
    "device_name" varchar(255),
    "browser" varchar(100),
    "os" varchar(100),
    "ip_address" varchar(50),
    "is_trusted" bool DEFAULT false,
    "is_active" bool DEFAULT true,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE INDEX IF NOT EXISTS idx_gw_login_devices_user_id ON "public"."gw_login_devices" ("user_id");
CREATE INDEX IF NOT EXISTS idx_gw_login_devices_device_id ON "public"."gw_login_devices" ("device_id");
COMMENT ON TABLE "public"."gw_login_devices" IS '登录设备管理表 - 记录用户登录设备，用于安全审计和异常检测';

CREATE TABLE IF NOT EXISTS "public"."gw_ip_whitelist" (
    "id" int8 NOT NULL DEFAULT nextval('gw_ip_whitelist_id_seq'::regclass),
    "ip" varchar(50) NOT NULL,
    "description" varchar(255),
    "is_active" bool DEFAULT true,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE INDEX IF NOT EXISTS idx_gw_ip_whitelist_ip ON "public"."gw_ip_whitelist" ("ip");
COMMENT ON TABLE "public"."gw_ip_whitelist" IS 'IP 白名单表 - 允许访问的 IP 地址列表';

CREATE TABLE IF NOT EXISTS "public"."gw_sensitive_audit" (
    "id" int8 NOT NULL DEFAULT nextval('gw_sensitive_audit_id_seq'::regclass),
    "user_id" int8,
    "action" varchar(100) NOT NULL,
    "resource" varchar(255),
    "detail" text,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE INDEX IF NOT EXISTS idx_gw_sensitive_audit_user_id ON "public"."gw_sensitive_audit" ("user_id");
COMMENT ON TABLE "public"."gw_sensitive_audit" IS '敏感操作审计表 - 记录敏感操作便于安全追溯';

CREATE TABLE IF NOT EXISTS "public"."gw_scheduled_tasks" (
    "id" int8 NOT NULL DEFAULT nextval('gw_scheduled_tasks_id_seq'::regclass),
    "name" varchar(255) NOT NULL,
    "cron" varchar(100) NOT NULL,
    "handler" varchar(255) NOT NULL,
    "is_active" bool DEFAULT true,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
COMMENT ON TABLE "public"."gw_scheduled_tasks" IS '定时任务表 - 存储定时任务定义';

CREATE TABLE IF NOT EXISTS "public"."gw_reports" (
    "id" int8 NOT NULL DEFAULT nextval('gw_reports_id_seq'::regclass),
    "name" varchar(255) NOT NULL,
    "template_id" int8,
    "params" jsonb,
    "is_active" bool DEFAULT true,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
COMMENT ON TABLE "public"."gw_reports" IS '报表定义表 - 存储报表配置';

CREATE TABLE IF NOT EXISTS "public"."gw_data_sources" (
    "id" int8 NOT NULL DEFAULT nextval('gw_data_sources_id_seq'::regclass),
    "name" varchar(255) NOT NULL,
    "type" varchar(50) NOT NULL,
    "config" jsonb,
    "is_active" bool DEFAULT true,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
COMMENT ON TABLE "public"."gw_data_sources" IS '数据源配置表 - 存储外部数据源连接信息';

CREATE TABLE IF NOT EXISTS "public"."gw_report_templates" (
    "id" int8 NOT NULL DEFAULT nextval('gw_report_templates_id_seq'::regclass),
    "name" varchar(255) NOT NULL,
    "format" varchar(50) NOT NULL,
    "config" jsonb,
    "is_active" bool DEFAULT true,
    "created_at" timestamptz DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
COMMENT ON TABLE "public"."gw_report_templates" IS '报表模板表 - 存储报表模板定义';

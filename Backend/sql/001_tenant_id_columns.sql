-- =============================================================================
-- Multi-tenancy Migration: Add missing tenant_id columns and constraints
-- =============================================================================
-- This migration adds tenant_id columns to tables that should be tenant-scoped
-- but currently lack them. It also adds foreign key constraints for data integrity.
--
-- SAFE TO RE-RUN: Uses IF NOT EXISTS / idempotent patterns.
-- =============================================================================

-- =============================================================================
-- Step 1: Audit tables that should have tenant_id (but don't)
-- =============================================================================

-- Check which tables need tenant_id (informational, no changes)
-- Tables already having tenant_id: sys_files, schedule_tasks, api_keys, files, audit_logs, api_usage_logs
-- Tables needing tenant_id: cms_article, cms_category, cms_comment, sys_message, announcements,
--                           sys_feedback, sys_operation_logs, sys_login_logs, tow_car

-- =============================================================================
-- Step 2: Add tenant_id columns where missing
-- =============================================================================

-- CMS tables (content management)
ALTER TABLE IF EXISTS public.cms_article ADD COLUMN IF NOT EXISTS tenant_id bigint;
ALTER TABLE IF EXISTS public.cms_category ADD COLUMN IF NOT EXISTS tenant_id bigint;
ALTER TABLE IF EXISTS public.cms_comment ADD COLUMN IF NOT EXISTS tenant_id bigint;

-- Messaging tables
ALTER TABLE IF EXISTS public.sys_message ADD COLUMN IF NOT EXISTS tenant_id bigint;
ALTER TABLE IF EXISTS public.announcements ADD COLUMN IF NOT EXISTS tenant_id bigint;
ALTER TABLE IF EXISTS public.message_templates ADD COLUMN IF NOT EXISTS tenant_id bigint;

-- Feedback
ALTER TABLE IF EXISTS public.sys_feedback ADD COLUMN IF NOT EXISTS tenant_id bigint;

-- Audit and logs
ALTER TABLE IF EXISTS public.sys_operation_logs ADD COLUMN IF NOT EXISTS tenant_id bigint;
ALTER TABLE IF EXISTS public.sys_login_logs ADD COLUMN IF NOT EXISTS tenant_id bigint;
ALTER TABLE IF EXISTS public.sys_sensitive_audits ADD COLUMN IF NOT EXISTS tenant_id bigint;

-- Workflow tables
ALTER TABLE IF EXISTS public.workflows ADD COLUMN IF NOT EXISTS tenant_id varchar(64);
ALTER TABLE IF EXISTS public.workflow_instances ADD COLUMN IF NOT EXISTS tenant_id varchar(64);
ALTER TABLE IF EXISTS public.task_records ADD COLUMN IF NOT EXISTS tenant_id varchar(64);

-- Tow service
ALTER TABLE IF EXISTS public.tow_car ADD COLUMN IF NOT EXISTS tenant_id bigint;

-- Clean service
ALTER TABLE IF EXISTS public.reports ADD COLUMN IF NOT EXISTS tenant_id varchar(64);
ALTER TABLE IF EXISTS public.report_tasks ADD COLUMN IF NOT EXISTS tenant_id varchar(64);

-- =============================================================================
-- Step 3: Add foreign key constraints to tenants table
-- =============================================================================

-- CMS tables
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_cms_article_tenant') THEN
        ALTER TABLE public.cms_article ADD CONSTRAINT fk_cms_article_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_cms_category_tenant') THEN
        ALTER TABLE public.cms_category ADD CONSTRAINT fk_cms_category_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_cms_comment_tenant') THEN
        ALTER TABLE public.cms_comment ADD CONSTRAINT fk_cms_comment_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Messaging
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_sys_message_tenant') THEN
        ALTER TABLE public.sys_message ADD CONSTRAINT fk_sys_message_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_announcements_tenant') THEN
        ALTER TABLE public.announcements ADD CONSTRAINT fk_announcements_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Feedback
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_sys_feedback_tenant') THEN
        ALTER TABLE public.sys_feedback ADD CONSTRAINT fk_sys_feedback_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Audit logs
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_sys_operation_logs_tenant') THEN
        ALTER TABLE public.sys_operation_logs ADD CONSTRAINT fk_sys_operation_logs_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE SET NULL;
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_sys_login_logs_tenant') THEN
        ALTER TABLE public.sys_login_logs ADD CONSTRAINT fk_sys_login_logs_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Workflow
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_workflows_tenant') THEN
        ALTER TABLE public.workflows ADD CONSTRAINT fk_workflows_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Tow service
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_tow_car_tenant') THEN
        ALTER TABLE public.tow_car ADD CONSTRAINT fk_tow_car_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Clean service
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_reports_tenant') THEN
        ALTER TABLE public.reports ADD CONSTRAINT fk_reports_tenant FOREIGN KEY (tenant_id) REFERENCES public.tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- =============================================================================
-- Step 4: Add indexes for tenant_id columns
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_cms_article_tenant_id ON public.cms_article(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cms_category_tenant_id ON public.cms_category(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cms_comment_tenant_id ON public.cms_comment(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_message_tenant_id ON public.sys_message(tenant_id);
CREATE INDEX IF NOT EXISTS idx_announcements_tenant_id ON public.announcements(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_feedback_tenant_id ON public.sys_feedback(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_operation_logs_tenant_id ON public.sys_operation_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_login_logs_tenant_id ON public.sys_login_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflows_tenant_id ON public.workflows(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tow_car_tenant_id ON public.tow_car(tenant_id);
CREATE INDEX IF NOT EXISTS idx_reports_tenant_id ON public.reports(tenant_id);

-- =============================================================================
-- DEPRECATED: This file has been split into separate scripts
-- =============================================================================
--
-- This file is kept for backward compatibility but should not be used directly.
--
-- Use instead:
--   1. Backend/sql/000_init.sql - Safe initialization (CREATE TABLE IF NOT EXISTS only)
--   2. Backend/sql/999_reset.sql - Destructive reset (DROP TABLE IF EXISTS only)
--
-- For initial deployment:
#   psql -U postgres -d myai -f Backend/sql/000_init.sql
#
-- For database reset (WARNING: DESTROYS ALL DATA):
#   psql -U postgres -d myai -f Backend/sql/999_reset.sql
#   psql -U postgres -d myai -f Backend/sql/000_init.sql
--
-- =============================================================================

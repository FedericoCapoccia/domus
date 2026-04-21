-- Add down migration script here
DROP INDEX IF EXISTS platform_user_single_owner_idx;

ALTER TABLE platform_user DROP COLUMN IF EXISTS role;

DROP TYPE IF EXISTS platform_user_role;

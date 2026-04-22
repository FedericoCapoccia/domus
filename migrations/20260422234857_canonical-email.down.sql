-- Add down migration script here
ALTER TABLE platform_user
DROP CONSTRAINT IF EXISTS platform_user_email_canonical_chk;

-- Add up migration script here
CREATE TYPE platform_user_role AS ENUM ('owner', 'admin', 'user');

ALTER TABLE platform_user
ADD COLUMN role platform_user_role NOT NULL DEFAULT 'user';

CREATE UNIQUE INDEX platform_user_single_owner_idx
ON platform_user (role)
WHERE role = 'owner';

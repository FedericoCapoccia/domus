-- Add up migration script her
CREATE TYPE platform_user_role AS ENUM ('owner', 'admin', 'user');

CREATE TABLE platform_user (
	id uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	email text NOT NULL,
	password_hash text NOT NULL,
    role platform_user_role NOT NULL DEFAULT 'user',
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT platform_user_email_unique UNIQUE(email),
    CONSTRAINT platform_user_email_canonical_chk CHECK (email = lower(btrim(email)))
);

CREATE UNIQUE INDEX platform_user_single_owner_idx ON platform_user (role) WHERE role = 'owner';

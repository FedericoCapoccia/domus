CREATE TYPE platform_user_role AS ENUM ('owner', 'admin', 'user');

CREATE COLLATION email_ci (
    provider = icu,
    locale = 'und-u-ks-level2',
    deterministic = false
);

CREATE TABLE platform_user (
	id uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	email text COLLATE email_ci NOT NULL,
	password_hash text NOT NULL,
    role platform_user_role NOT NULL DEFAULT 'user',
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT platform_user_email_unique UNIQUE(email),
    CONSTRAINT platform_user_email_trim_chk CHECK (email = btrim(email))
);

CREATE UNIQUE INDEX platform_user_single_owner_idx ON platform_user (role) WHERE role = 'owner';

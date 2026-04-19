-- Add up migration script her
CREATE TABLE platform_user (
	id uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	email text NOT NULL,
	password_hash text NOT NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT platform_user_email_unique UNIQUE(email)
);

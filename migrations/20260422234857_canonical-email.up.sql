-- Add up migration script here
ALTER TABLE platform_user
ADD CONSTRAINT platform_user_email_canonical_chk
CHECK (email = lower(btrim(email)));

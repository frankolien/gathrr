ALTER TABLE users ADD COLUMN is_guest BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN claimed_at TIMESTAMPTZ;

ALTER TABLE users DROP CONSTRAINT users_phone_key;
ALTER TABLE users DROP CONSTRAINT users_email_key;

CREATE UNIQUE INDEX users_phone_claimed_key ON users(phone)
  WHERE is_guest = false AND phone IS NOT NULL;
CREATE UNIQUE INDEX users_email_claimed_key ON users(email)
  WHERE is_guest = false AND email IS NOT NULL;
CREATE INDEX users_guest_phone_idx ON users(phone)
  WHERE is_guest = true AND phone IS NOT NULL;

CREATE TABLE guest_sessions (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL UNIQUE,
  invite_id  UUID REFERENCES invites(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX guest_sessions_user_idx ON guest_sessions(user_id);

CREATE TYPE identity_provider AS ENUM ('apple','google');

CREATE TABLE identities (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider   identity_provider NOT NULL,
  subject    TEXT NOT NULL,
  email      TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (provider, subject)
);

CREATE INDEX identities_user_idx ON identities(user_id);

ALTER TABLE users DROP COLUMN apple_sub;

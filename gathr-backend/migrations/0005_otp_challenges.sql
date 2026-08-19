CREATE TYPE otp_channel AS ENUM ('email','phone');

CREATE TABLE otp_challenges (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  channel     otp_channel NOT NULL,
  destination TEXT NOT NULL,
  code_hash   TEXT NOT NULL,
  attempts    INTEGER NOT NULL DEFAULT 0,
  consumed_at TIMESTAMPTZ,
  expires_at  TIMESTAMPTZ NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX otp_pending_idx
  ON otp_challenges(channel, destination, created_at DESC)
  WHERE consumed_at IS NULL;

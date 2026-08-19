CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  apple_sub       TEXT UNIQUE,
  phone           TEXT UNIQUE,
  email           TEXT UNIQUE,
  display_name    TEXT NOT NULL,
  avatar_media_id UUID,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE event_status AS ENUM ('draft','published','ongoing','ended','cancelled');

CREATE TABLE events (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  host_id        UUID NOT NULL REFERENCES users(id),
  title          TEXT NOT NULL,
  category       TEXT NOT NULL DEFAULT 'other',
  description    TEXT,
  cover_media_id UUID,
  location_name  TEXT,
  location_lat   DOUBLE PRECISION,
  location_lng   DOUBLE PRECISION,
  starts_at      TIMESTAMPTZ NOT NULL,
  ends_at        TIMESTAMPTZ,
  timezone       TEXT NOT NULL DEFAULT 'Africa/Lagos',
  capacity       INTEGER CHECK (capacity IS NULL OR capacity > 0),
  max_plus_ones  INTEGER NOT NULL DEFAULT 2 CHECK (max_plus_ones >= 0),
  status         event_status NOT NULL DEFAULT 'draft',
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (ends_at IS NULL OR ends_at > starts_at)
);
CREATE INDEX idx_events_starts_at ON events(starts_at);
CREATE INDEX idx_events_host ON events(host_id, starts_at);

CREATE TABLE invites (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id   UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  code       TEXT NOT NULL UNIQUE,
  max_uses   INTEGER CHECK (max_uses IS NULL OR max_uses > 0),
  uses       INTEGER NOT NULL DEFAULT 0 CHECK (uses >= 0),
  expires_at TIMESTAMPTZ,
  created_by UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_invites_event ON invites(event_id);

CREATE TYPE rsvp_status AS ENUM ('invited','going','maybe','declined','waitlisted');

CREATE TABLE rsvps (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id   UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  status     rsvp_status NOT NULL DEFAULT 'invited',
  plus_ones  INTEGER NOT NULL DEFAULT 0 CHECK (plus_ones >= 0),
  invite_id  UUID REFERENCES invites(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (event_id, user_id)
);
CREATE INDEX idx_rsvps_event_status ON rsvps(event_id, status);

CREATE TABLE messages (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id   UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  sender_id  UUID NOT NULL REFERENCES users(id),
  seq        BIGINT NOT NULL,
  body       TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (event_id, seq)
);

CREATE TABLE devices (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  apns_token   TEXT NOT NULL UNIQUE,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE media (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id     UUID NOT NULL REFERENCES users(id),
  bucket_key   TEXT NOT NULL,
  content_type TEXT NOT NULL,
  width        INTEGER,
  height       INTEGER,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE reminder_jobs (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id   UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  run_at     TIMESTAMPTZ NOT NULL,
  kind       TEXT NOT NULL,
  status     TEXT NOT NULL DEFAULT 'pending',
  attempts   INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_reminder_due ON reminder_jobs(run_at) WHERE status = 'pending';

CREATE TABLE event_counters (
  event_id UUID PRIMARY KEY REFERENCES events(id) ON DELETE CASCADE,
  last_seq BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE idempotency_keys (
  key           TEXT PRIMARY KEY,
  user_id       UUID NOT NULL,
  request_hash  TEXT NOT NULL,
  response_code INTEGER,
  response_body JSONB,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_idempotency_created ON idempotency_keys(created_at);

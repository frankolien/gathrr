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


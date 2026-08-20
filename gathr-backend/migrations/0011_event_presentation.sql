CREATE TYPE event_visibility AS ENUM ('public', 'private');

ALTER TABLE events
  ADD COLUMN cover_template_id TEXT,
  ADD COLUMN visibility        event_visibility NOT NULL DEFAULT 'public',
  ADD COLUMN requires_approval BOOLEAN NOT NULL DEFAULT FALSE;

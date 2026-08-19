ALTER TABLE devices ADD COLUMN environment TEXT NOT NULL DEFAULT 'sandbox';

CREATE TABLE event_mutes (
  user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  muted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, event_id)
);


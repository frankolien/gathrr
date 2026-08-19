ALTER TABLE devices ADD COLUMN environment TEXT NOT NULL DEFAULT 'sandbox';

CREATE TABLE event_mutes (
  user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  muted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, event_id)
);

CREATE UNIQUE INDEX reminder_jobs_event_kind_key ON reminder_jobs(event_id, kind);

ALTER TABLE reminder_jobs ADD COLUMN locked_at TIMESTAMPTZ;
ALTER TABLE reminder_jobs ADD COLUMN last_error TEXT;

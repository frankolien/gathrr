CREATE TYPE report_subject AS ENUM ('message', 'user');

CREATE TABLE reports (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  subject     report_subject NOT NULL,
  subject_id  UUID NOT NULL,
  event_id    UUID REFERENCES events(id) ON DELETE CASCADE,
  reason      TEXT NOT NULL,
  detail      TEXT,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_reports_subject ON reports(subject, subject_id);
CREATE UNIQUE INDEX reports_one_per_reporter ON reports(reporter_id, subject, subject_id);

CREATE TABLE blocks (
  blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  blocked_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (blocker_id, blocked_id),
  CHECK (blocker_id <> blocked_id)
);
CREATE INDEX idx_blocks_blocked ON blocks(blocked_id);

ALTER TABLE messages ADD COLUMN redacted_at TIMESTAMPTZ;
ALTER TABLE messages ALTER COLUMN sender_id DROP NOT NULL;
ALTER TABLE messages DROP CONSTRAINT messages_sender_id_fkey;
ALTER TABLE messages ADD CONSTRAINT messages_sender_id_fkey
  FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE events DROP CONSTRAINT events_host_id_fkey;
ALTER TABLE events ADD CONSTRAINT events_host_id_fkey
  FOREIGN KEY (host_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE invites DROP CONSTRAINT invites_created_by_fkey;
ALTER TABLE invites ADD CONSTRAINT invites_created_by_fkey
  FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE media DROP CONSTRAINT media_owner_id_fkey;
ALTER TABLE media ADD CONSTRAINT media_owner_id_fkey
  FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE rsvps DROP CONSTRAINT rsvps_invite_id_fkey;
ALTER TABLE rsvps ADD CONSTRAINT rsvps_invite_id_fkey
  FOREIGN KEY (invite_id) REFERENCES invites(id) ON DELETE SET NULL;

ALTER TABLE guest_sessions DROP CONSTRAINT guest_sessions_invite_id_fkey;
ALTER TABLE guest_sessions ADD CONSTRAINT guest_sessions_invite_id_fkey
  FOREIGN KEY (invite_id) REFERENCES invites(id) ON DELETE SET NULL;

CREATE TYPE host_role AS ENUM ('owner', 'co_host');

CREATE TABLE event_hosts (
  event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role     host_role NOT NULL DEFAULT 'co_host',
  added_by UUID REFERENCES users(id) ON DELETE SET NULL,
  added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (event_id, user_id)
);

CREATE INDEX idx_event_hosts_user ON event_hosts(user_id);
CREATE UNIQUE INDEX event_hosts_single_owner ON event_hosts(event_id) WHERE role = 'owner';

INSERT INTO event_hosts (event_id, user_id, role, added_by)
SELECT id, host_id, 'owner', host_id FROM events
ON CONFLICT DO NOTHING;

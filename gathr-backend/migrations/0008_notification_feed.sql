CREATE TYPE notification_kind AS ENUM (
  'rsvp_accepted',
  'rsvp_declined',
  'rsvp_waitlisted',
  'message_posted',
  'event_published',
  'event_cancelled',
  'event_reminder'
);

CREATE TABLE notifications (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  actor_id   UUID REFERENCES users(id) ON DELETE SET NULL,
  event_id   UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  kind       notification_kind NOT NULL,
  read_at    TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


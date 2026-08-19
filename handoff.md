# Specification Pack: Gathr, a Mobile Invite and RSVP App

Backend: Rust + Actix Web. Client: SwiftUI iOS. Context: Lagos, Nigeria (WAT, UTC+1).

## TL;DR

- Build an offline-first iOS app on a Rust/Actix Web + Postgres backend, benchmarked against Partiful (Going/Maybe/Can't Go, text-blast reminders, capacity with waitlists, no app needed to RSVP), Luma, and Apple Invites (launched February 4, 2025).
- Recommended stack: Actix Web 4.x (latest 4.14.1, MSRV Rust 1.88) with a Cargo workspace (hexagonal layout), SQLx 0.8.6 with Postgres, actix-ws for chat, the a2 crate for APNs, Cloudflare R2 for images via presigned URLs, and Postgres FOR UPDATE SKIP LOCKED for reminder jobs.
- iOS: Swift 6 strict concurrency, @Observable, NavigationStack with a router pattern, SPM-modularized packages, and GRDB/SQLiteData for the offline cache. Deployment floor iOS 17.0 (Decision D4).
- Sections 7-21 turn the spec into a buildable plan: design tokens read off the mockups, specs for the nine screens the mockups omit, the web RSVP path that the conversion thesis depends on, the offline sync protocol, Lagos-specific delivery and network constraints, and a migration ledger. Section 17 settles every conflict between the mockups and the written spec; Section 18 is the cut line if there is a demo deadline.

## Key Findings

Benchmark facts drive the scope. Partiful uses Going/Maybe/Can't Go with Maybe toggleable, and its own help center states auto-reminders "are sent 1 week before the event to 'Invited' and 'Maybe' guests, and 2 hours before the event to 'Going' guests," and it requires no app download to RSVP. Apple Invites, per Apple's February 4, 2025 newsroom release, works so that "iCloud+ subscribers can create invitations, and anyone can RSVP, regardless of whether they have an Apple Account or Apple device." Luma targets tech/creator communities with ticketing (5% platform fee plus Stripe processing, waived on Luma Plus). Gathr should copy the frictionless link-based RSVP and lean into a native chat feature that Apple Invites lacks.

Partiful's momentum sets the market bar: TIME's 2025 TIME100 writeup reports it "added over 2 million new users in the first quarter of 2025, and grew globally as well, reaching users in over 100 countries," with "user activity [that] rose by 600% in 2024." It has raised roughly $27.3M total, with a November 8, 2022 $20M Series A led by Andreessen Horowitz, and was founded in 2020 by Palantir alumni Shreya Murthy (CEO) and Joy Tao (CTO).

---

## 1. Product Requirements Document

### 1.1 Vision

Gathr is the simplest way to bring people together: invite friends, plan events, and keep everyone in the loop. The product wins on speed to create an invite, frictionless RSVP (no forced signup to respond), and a live event chat that keeps guests engaged before and during the event. Primary market is Lagos, Nigeria, with events in Victoria Island and Ikeja, so the app must tolerate intermittent connectivity and be offline-first.

### 1.2 Personas

Host (Amara, 29, Lagos): plans birthdays and game nights. Needs fast creation with templates, guest tracking, reminders, and a way to message everyone at once. Success = high RSVP conversion and attendance.

Guest (Tunde, 26, Lagos): receives an invite link or QR code. Wants to see when/where at a glance, RSVP in one tap, bring a plus-one, and coordinate via chat. Success = one-tap RSVP without friction.

### 1.3 User Stories with Acceptance Criteria

Auth/Onboarding
- As a new user I can complete an onboarding carousel and sign in with Apple or phone/email OTP. AC: Sign in with Apple returns a stable user; OTP delivered within a target window; a guest can view an invite and RSVP before creating a full account (deferred auth).

Event Creation with Templates and Cover Images
- As a host I can create an event from a template or from scratch with a cover image, category (e.g. BIRTHDAY), title, date/time, location, and capacity. AC: cover image uploads via presigned URL; event saved as draft then published; timezone stored explicitly.

Invitations (Deep Links, Invite Codes, QR)
- As a host I can share an invite deep link, a short invite code, and a QR code. AC: universal link opens the event detail directly; entering a code resolves the event; QR scan resolves the same; codes can be single-use or multi-use with optional expiry.

RSVP Flow
- As a guest I can RSVP Going, Maybe, or Can't Go and add plus-ones. AC: capacity enforced server-side (a Going RSVP is rejected when at capacity, offered a waitlist); RSVP is idempotent; plus-one count validated against a per-event max.

Guest Management
- As a host I can see and manage the guest list, remove guests, and promote from waitlist. AC: Manage screen lists guests grouped by status with counts ("18 going").

Event Chat
- As a guest I can chat with other attendees in a per-event thread. AC: messages persist, ordered by a monotonic per-event sequence, delivered in near real time over WebSocket, and paginated by cursor.

Countdown/Reminders/Push
- As a guest I see a live countdown and receive reminders. AC: countdown computed from server UTC; reminders scheduled (e.g. 1 week and 2 hours before, matching Partiful's cadence) and delivered by push.

Discovery
- As a user I see "This week" cards, plus Hosting and Attending lists. AC: This week sorts by start time with hosting-priority weighting.

Edit/Cancel Event
- As a host I can edit or cancel; guests are notified. AC: cancel transitions lifecycle to cancelled and fans out a notification.

Offline Behavior
- As a user I can view cached events and my RSVPs offline and queue an RSVP that syncs when back online. AC: reads served from local cache; writes queued and retried with idempotency keys.

### 1.4 Non-Functional Requirements

- Latency: p95 read API under 200 ms server-side; chat message round trip under 300 ms on good networks.
- Offline-first: all primary read screens work from cache; writes queue and reconcile.
- Privacy: guests can RSVP without exposing phone numbers to other guests; PII encrypted in transit and at rest; minimal data collection.
- Availability: target 99.9% for the API.

### 1.5 Scope

MVP: auth (Apple + OTP), event create from template, cover image, invite link + code + QR, RSVP Going/Maybe/Can't Go + plus-ones, guest list, countdown, push reminders, This week/Hosting/Attending, edit/cancel, basic offline reads.
V1: event chat, waitlists, co-hosts, richer templates, shared photo album.
V2: discovery/explore feed, recurring events, ticketing/payments, collaborative playlist.

### 1.6 Success Metrics

- Invite-to-RSVP conversion rate (target above 40%).
- RSVP-to-attendance rate.
- Time-to-create an event (target under 2 minutes).
- D7 host retention and events created per host.

---

## 2. System Design: Rust + Actix Web Backend

### 2.1 Architecture and Workspace Layout

Use Actix Web 4.x (latest 4.14.1, MSRV Rust 1.88) with a Cargo workspace in a hexagonal (ports and adapters) layout. Domain logic is pure and has no framework dependency; adapters implement traits for Postgres, object storage, and APNs.

```
gathr-backend/
  Cargo.toml            # [workspace]
  crates/
    domain/             # entities, value objects, invariants, trait ports (no actix, no sqlx)
    application/        # use cases / services orchestrating ports
    infra_db/           # sqlx adapters implementing domain ports
    infra_storage/      # R2/S3 presigned URL adapter
    infra_push/         # a2 APNs adapter
    api/                # actix-web handlers, extractors, middleware, DTOs
    worker/             # background jobs (reminders) binary
    common/             # error types, tracing setup, config
  migrations/           # sqlx migrations
```

This keeps the domain testable in isolation and lets adapters be swapped. The `api` and `worker` are separate binaries sharing the same crates.

### 2.2 Database Access: SQLx vs Diesel vs SeaORM

Recommendation: SQLx 0.8.6 (the latest 0.8 track; SQLx 0.9.0 shipped around May 2026 with a raised MSRV of Rust 1.94, new smol/async-global-executor runtimes, and an sqlx.toml config file, but pin to 0.8.6 until 0.9 stabilizes in your CI). SQLx gives async-native, compile-time-checked raw SQL against Postgres without an ORM DSL, which suits a schema with explicit transactional invariants (capacity, sequence numbers) where hand-written SQL with FOR UPDATE and ON CONFLICT is clearest. Diesel is compile-time safe but its DSL fights dynamic queries and needs diesel-async; SeaORM is ergonomic for CRUD-heavy Rails-style code but adds abstraction over the same SQLx foundation. For a correctness-critical, query-shaped domain, SQLx is the right call.

Recommended Cargo features: `runtime-tokio`, `tls-rustls-ring-webpki`, `postgres`, `macros`, `migrate`, `uuid`, `time`, `json`. Install `sqlx-cli` for migrations and enable offline mode (a committed `.sqlx/` cache) so CI compiles query macros without a live DB.

### 2.3 Authentication

- Sign in with Apple plus phone/email OTP.
- Passwords (if any email/password path) hashed with argon2 (argon2id), chosen over bcrypt for faster hashing and stronger memory-hardness.
- JWT access tokens (short-lived, ~15 min) + refresh tokens with rotation: every refresh issues a new refresh token, burns the old one, and reuse of a burned token revokes the whole token family. Store refresh token metadata server-side (Postgres, optionally Redis) keyed by a family id and jti.
- Access token carries sub and jti; verify signature, exp, iss, aud.

### 2.4 Realtime Chat

Use actix-ws (0.3) WebSockets, not the older actor-based actix-web-actors. Each connection is handled in a spawned task reading a message stream. Fan-out uses a tokio broadcast channel per event room held in shared state; a subscriber task forwards to each session. Messages are persisted to Postgres with a per-event monotonic sequence number assigned inside a transaction before broadcast. SSE was considered but rejected: chat is bidirectional. Heartbeat ping/pong with a client timeout detects dead connections.

### 2.5 Push Notifications

Use the a2 crate (async APNs over HTTP/2, token-based .p8 auth with signature renewal and caching; battle-tested pushing millions of notifications daily in the WalletConnect Echo Server). Reuse a single Client across requests (opening a new connection per request risks APNs treating it as a DoS). Store device push tokens per user/device; prune tokens on APNs "Unregistered" responses.

### 2.6 Image Upload Pipeline

Cloudflare R2 (S3-compatible, zero egress). The client requests a presigned PUT URL from the API (generated via the AWS SDK for Rust `presigned()` on `put_object`, expiry ~5 minutes), uploads directly to R2, then confirms; the API records a media row. Generate resized variants (thumbnail, card, cover) either via an image-resizing worker or an on-the-fly transform. Never proxy bytes through the API.

### 2.7 Invite Codes and QR

Short codes use Crockford base32 (no I, L, O, U to avoid ambiguity). Codes stored with a unique index; QR encodes the universal link that embeds the code. See Section 5.3 for semantics and Section 5.1 for the generation algorithm.

### 2.8 Rate Limiting and Idempotency

- Rate limiting: token bucket per user/IP on sensitive endpoints (OTP request, RSVP, message send).
- Idempotency: mutating POSTs accept an `Idempotency-Key` header; the server saves the resulting status code and body for a given key and replays the stored response on retry, following the Stripe pattern (which guarantees the same result, including 5xx, for repeat keys). Critical for queued offline writes.

### 2.9 Background Jobs

Reminders use a Postgres-backed queue with `SELECT ... FOR UPDATE SKIP LOCKED` claiming, which is atomic and race-free across concurrent workers (each worker gets a different job, guaranteed). A `worker` binary polls (or uses LISTEN/NOTIFY for low latency) for due reminder jobs, sends push via a2, and marks them done. For heavier needs, graphile_worker_rs (a Rust port of Graphile Worker with SKIP LOCKED job claiming, LISTEN/NOTIFY wakeups, and cron) is a drop-in option.

### 2.10 Database Schema (Postgres DDL)

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  apple_sub     TEXT UNIQUE,
  phone         TEXT UNIQUE,
  email         TEXT UNIQUE,
  display_name  TEXT NOT NULL,
  avatar_media_id UUID,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE event_status AS ENUM ('draft','published','ongoing','ended','cancelled');

CREATE TABLE events (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  host_id       UUID NOT NULL REFERENCES users(id),
  title         TEXT NOT NULL,
  category      TEXT,
  description   TEXT,
  cover_media_id UUID,
  location_name TEXT,
  location_lat  DOUBLE PRECISION,
  location_lng  DOUBLE PRECISION,
  starts_at     TIMESTAMPTZ NOT NULL,
  ends_at       TIMESTAMPTZ,
  timezone      TEXT NOT NULL DEFAULT 'Africa/Lagos',
  capacity      INTEGER,
  status        event_status NOT NULL DEFAULT 'draft',
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_events_starts_at ON events(starts_at);

CREATE TABLE invites (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id      UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  code          TEXT NOT NULL UNIQUE,
  max_uses      INTEGER,            -- NULL = unlimited
  uses          INTEGER NOT NULL DEFAULT 0,
  expires_at    TIMESTAMPTZ,        -- NULL = no expiry
  created_by     UUID NOT NULL REFERENCES users(id),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE rsvp_status AS ENUM ('invited','going','maybe','declined','waitlisted');

CREATE TABLE rsvps (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id      UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  user_id       UUID NOT NULL REFERENCES users(id),
  status        rsvp_status NOT NULL DEFAULT 'invited',
  plus_ones     INTEGER NOT NULL DEFAULT 0 CHECK (plus_ones >= 0),
  invite_id     UUID REFERENCES invites(id),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (event_id, user_id)
);
CREATE INDEX idx_rsvps_event_status ON rsvps(event_id, status);

CREATE TABLE messages (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id      UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  sender_id     UUID NOT NULL REFERENCES users(id),
  seq           BIGINT NOT NULL,
  body          TEXT NOT NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (event_id, seq)
);

CREATE TABLE devices (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  apns_token    TEXT NOT NULL UNIQUE,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE media (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id      UUID NOT NULL REFERENCES users(id),
  bucket_key    TEXT NOT NULL,
  content_type  TEXT NOT NULL,
  width         INTEGER,
  height        INTEGER,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE reminder_jobs (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id      UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  run_at        TIMESTAMPTZ NOT NULL,
  kind          TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'pending',
  attempts      INTEGER NOT NULL DEFAULT 0,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_reminder_due ON reminder_jobs(run_at) WHERE status = 'pending';

CREATE TABLE event_counters (
  event_id      UUID PRIMARY KEY REFERENCES events(id) ON DELETE CASCADE,
  last_seq      BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE idempotency_keys (
  key           TEXT PRIMARY KEY,
  user_id       UUID NOT NULL,
  response_code INTEGER,
  response_body JSONB,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 2.11 API Design (Endpoint Table)

Versioned under `/v1`. JSON everywhere. Error format below.

| Method | Path | Purpose | Body / Notes |
|---|---|---|---|
| POST | /v1/auth/apple | Sign in with Apple | identity token |
| POST | /v1/auth/otp/request | Request OTP | phone/email |
| POST | /v1/auth/otp/verify | Verify OTP, issue tokens | code |
| POST | /v1/auth/refresh | Rotate tokens | refresh token |
| POST | /v1/auth/logout | Revoke family | - |
| GET | /v1/me | Current user | - |
| PATCH | /v1/me | Update profile | display_name, avatar |
| POST | /v1/devices | Register APNs token | apns_token |
| DELETE | /v1/devices/{id} | Remove device | - |
| POST | /v1/events | Create event (Idempotency-Key) | event fields |
| GET | /v1/events/{id} | Event detail | - |
| PATCH | /v1/events/{id} | Edit event | partial |
| POST | /v1/events/{id}/publish | Draft to published | - |
| POST | /v1/events/{id}/cancel | Cancel event | - |
| GET | /v1/events?filter=this_week\|hosting\|attending | Feeds | - |
| POST | /v1/events/{id}/invites | Create invite code | max_uses, expires_at |
| GET | /v1/invites/{code} | Resolve code | - |
| POST | /v1/events/{id}/rsvp | Upsert RSVP (Idempotency-Key) | status, plus_ones |
| GET | /v1/events/{id}/guests | Guest list | - |
| DELETE | /v1/events/{id}/guests/{uid} | Remove guest | host only |
| GET | /v1/events/{id}/messages?cursor= | Chat history (keyset) | - |
| POST | /v1/events/{id}/messages | Send message (Idempotency-Key) | body |
| GET | /v1/events/{id}/chat/ws | WebSocket upgrade | - |
| POST | /v1/media/presign | Presigned upload URL | content_type |

Error format:
```json
{ "error": { "code": "capacity_exceeded", "message": "Event is at capacity", "request_id": "..." } }
```

### 2.12 Observability, Deployment, Testing

- Observability: `tracing` with JSON output from day one; request spans with request_id; metrics (Prometheus) for latency, RSVP counts, push success. Structured JSON logging is far easier to set up from the start than to retrofit.
- Deployment: multi-stage Docker (cargo-chef for cached builds, images often around 80 MB), deploy to Fly.io (Fly Machines, hardware-virtualized containers, TLS terminated by the platform) or a VPS with Caddy for automatic Let's Encrypt TLS. App speaks plain HTTP behind the proxy. Use rustls not OpenSSL to avoid cross-compile pain. Fly's `fly launch` scanner generates a cargo-chef Dockerfile automatically.
- Testing: unit tests on the domain crate; integration tests with testcontainers (testcontainers-modules Postgres) spinning up a real Postgres per suite, fresh DB per test for isolation. Real-database tests catch SQL syntax, constraints, and transaction issues that mocks miss.

---

## 3. SwiftUI iOS Architecture

### 3.1 Pattern and Concurrency

Use a pragmatic MV/MVVM hybrid on the Observation framework (@Observable), which is pull-based and access-tracked for precise invalidation and better performance than the Combine-based ObservableObject. Views that only render (pure views like Text/Image) need no view model; screens with real logic (event creation, RSVP, chat) get an @Observable class isolated to @MainActor. Networking and persistence live in actors. Adopt Swift 6 strict concurrency: models Sendable, services as actors, view models @MainActor, and lean on region-based isolation, which the compiler uses to cut required Sendable annotations by 50 to 70% in a typical migration.

Minimum target: iOS 17 for @Observable. An iOS 18 floor buys the latest concurrency and SwiftUI conveniences but cuts off the iPhone X/8/8 Plus generation, which still carries real share in the Lagos secondhand market. Decision D4 in Section 17 sets the floor at **iOS 17.0** for this reason; revisit after beta telemetry.

### 3.2 Modularization (SPM Packages)

```
GathrApp/                 # app target, DI composition root
Packages/
  DesignSystem/           # colors, typography, components (cards, chips, avatar cluster)
  Models/                 # Sendable domain models, DTOs
  Networking/             # URLSession client, endpoints, WebSocket client
  Persistence/            # GRDB/SQLiteData cache, offline queue
  Features/
    Onboarding/
    Home/
    EventDetail/
    Create/
    Chat/
    Profile/
```

### 3.3 Navigation

NavigationStack driven by a per-tab Router (an @Observable holding a typed path of a Route enum). Each tab owns its own independent stack (the tab-router pattern avoids cross-tab path corruption). Universal links map to Route values so an invite link deep-links straight to EventDetail. A router gives coordinator-level power with less ceremony than a full coordinator hierarchy.

### 3.4 Dependency Injection

Environment-based injection of service protocols (a lightweight container assembled at the composition root), so features depend on protocols (e.g. `EventService`, `ChatService`) and can be previewed/tested with fakes.

### 3.5 Local Cache and Offline

Use GRDB (or SQLiteData, the Point-Free SwiftData alternative built on GRDB with optional CloudKit sync) rather than SwiftData for the offline-first cache: direct SQLite access, better read/write performance (frameworks that talk directly to SQLite outperform Core Data, which outperforms SwiftData), usability outside SwiftUI (Observable models, UIKit), and full control over the sync/outbox table. SwiftData is viable for simpler needs but its higher abstraction costs performance and control. An outbox table holds queued mutations (RSVP, message) with idempotency keys, retried on reconnect. Note also the third-party vendor-risk lesson: Realm/Atlas Device Sync reached end-of-life with cloud sync ending September 30, 2025, so prefer Apple-first or self-controlled local stores.

### 3.6 Networking and Chat Client

- URLSession-based client with async endpoints, typed via an `Endpoint` protocol; async/await throughout; typed errors.
- Chat over URLSessionWebSocketTask wrapped in an actor; auto-reconnect with backoff; messages reconciled by per-event seq.
- Push registration via UNUserNotificationCenter; token posted to /v1/devices.
- Image loading/caching: a small async image cache (or a vetted library) reading resized R2 variants.

### 3.7 Design System

Light, iOS-native aesthetic matching the mockups: system font (SF) with a defined type scale, soft cards with rounded corners and subtle shadows, category chips (BIRTHDAY etc.), countdown chips ("In 9 days"), avatar cluster with "+14 going". Centralize color tokens and spacing in DesignSystem.

### 3.8 Testing

Swift Testing framework (@Test, #expect, parameterized tests, tags), which is macro-powered and runs side by side with XCTest during migration. Unit-test view models and services with injected fakes; snapshot-test DesignSystem components.

### 3.9 Key Protocols

```swift
protocol EventService: Sendable {
    func feed(_ filter: FeedFilter) async throws -> [Event]
    func detail(_ id: Event.ID) async throws -> EventDetail
    func rsvp(_ id: Event.ID, status: RSVPStatus, plusOnes: Int, idempotencyKey: String) async throws -> RSVP
}
protocol ChatService: Sendable {
    func history(_ id: Event.ID, cursor: String?) async throws -> Page<Message>
    func connect(_ id: Event.ID) async throws -> AsyncStream<Message>
    func send(_ id: Event.ID, body: String, idempotencyKey: String) async throws
}
```

---

## 4. Formal Methods: Core Invariants

Notation: predicate/TLA+-style. `count(...)` counts rows.

### 4.1 RSVP Status State Machine

States: invited, going, maybe, declined, waitlisted.

```
guest_selectable = { going, maybe, declined }

  any state -> any guest_selectable    [guard: CAP, and only when target = going]
  going     -> waitlisted              [system only, when CAP fails and the guest opted in]
  *         -> invited                 [forbidden; invited is an initial state only]
  *         -> waitlisted              [forbidden as a guest choice]
```

The original enumeration omitted `declined -> maybe`, which contradicts Section 1.3 ("a guest can RSVP Going, Maybe, or Can't Go") and Section 8.5, where the sheet offers all three unconditionally — a guest who tapped Can't Go and reopened the sheet would have hit an error. The three guest-selectable statuses are freely interchangeable; only `invited` (initial) and `waitlisted` (system-assigned on a failed CAP with opt-in) are not directly choosable. Encoded in `gathr-domain::rsvp::submit`.

Waitlisting is opt-in, never automatic: a failed CAP returns `capacity_exceeded` so the client can offer the waitlist explicitly (8.5, 10.3). Re-confirming an existing waitlist place does not reset `waitlisted_at`, so a guest cannot lose their queue position by tapping twice.

Guard CAP (canonical, single definition — every transition into `going` uses exactly this):

```
seats_held(E, U)  = count(r in rsvps: r.event=E and r.status='going' and r.user != U)
                  + sum(r.plus_ones for those rows)
seats_needed(U)   = 1 + requested_plus_ones
CAP(E, U)         = capacity(E) IS NULL OR seats_held(E, U) + seats_needed(U) <= capacity(E)
```

The actor's own existing row is excluded from `seats_held` so that a guest already `going` who edits their plus-one count is not double-counted. A `going` RSVP consumes `1 + plus_ones` seats, never one seat.

Waitlist fairness orders by `waitlisted_at` (set once on entry to `waitlisted`, never touched again), not `updated_at` — `updated_at` moves on any edit and would silently reshuffle the queue. See migration 0004 in Section 21.

### 4.2 Event Lifecycle State Machine

```
draft -> published        [guard: has title, starts_at, host]
published -> ongoing       [auto: now >= starts_at]
ongoing -> ended           [auto: now >= ends_at]
published -> cancelled
ongoing -> cancelled
draft -> cancelled
(ended and cancelled are terminal)
```

### 4.3 Invite Code Semantics

- Uniqueness: `forall i1,i2 in invites: i1.code = i2.code => i1.id = i2.id` (enforced by UNIQUE index).
- Expiry: an invite is redeemable iff `expires_at IS NULL OR now() < expires_at`.
- Single vs multi use: redeemable iff `max_uses IS NULL OR uses < max_uses`. Single-use sets max_uses=1.

### 4.4 Chat Ordering and Delivery

- Monotonic sequence: `forall m in messages(E): unique(m.seq)` and seq assigned strictly increasing per event.
- Ordering guarantee: messages are totally ordered per event by seq (not by wall-clock created_at).
- Delivery: at-least-once over WebSocket; client dedupes by (event_id, seq). Persist-before-broadcast ensures no acknowledged message is lost.

### 4.5 Safety and Liveness

Safety invariants:
- CAP: `count(going rsvps incl plus_ones) <= capacity` for every event with non-null capacity.
- INV: `forall r in rsvps: exists valid invite OR event is public` (no RSVP without a valid path).
- SEQ: message sequence numbers are unique and monotonic per event.

Liveness properties:
- A waitlisted guest eventually transitions to going if capacity frees and they are next (fairness by updated_at order).
- A due reminder job is eventually delivered (worker claims and completes).

### 4.6 Mapping to Database and Transactional Code

- CAP: enforce inside a transaction using `SELECT ... FOR UPDATE` on the event row (or a counter row) before inserting/updating a going RSVP; reject if guard fails. Prevents the classic capacity race.
- Uniqueness of RSVP: `UNIQUE (event_id, user_id)` with upsert via `ON CONFLICT`.
- SEQ: allocate seq inside the same transaction that inserts the message, via an upsert so the first message of an event cannot silently no-op on a missing counter row:
  ```sql
  INSERT INTO event_counters (event_id, last_seq) VALUES ($1, 1)
  ON CONFLICT (event_id) DO UPDATE SET last_seq = event_counters.last_seq + 1
  RETURNING last_seq;
  ```
  `UNIQUE (event_id, seq)` is the backstop. A plain `UPDATE ... RETURNING` returns zero rows when no counter exists and must not be used.
- Invite codes: `UNIQUE(code)`; redemption increments `uses` under row lock with the expiry/max_uses guard.
- Reminders: `FOR UPDATE SKIP LOCKED` guarantees each job is claimed by exactly one worker.

---

## 5. Algorithms with Complexity

### 5.1 Invite Code Generation (collision-resistant)

Use Crockford base32 codes of length L. With a 32-symbol alphabet, an L-character code has 32^L = 2^(5L) possibilities. An 8-char code gives 2^40 (~1.1e12) space. By the birthday bound, the probability of any collision among n codes is approximately `1 - e^(-n^2 / (2 * space))`; collision risk scales with n^2, not n, so it grows far faster than linear intuition suggests. For 1e6 live codes against 2^40 that risk is roughly 35 to 45%, so 8 chars is too short at scale; use 10 chars (2^50 ~ 1.1e15) which drops it to about 4e-4, or 12 chars for very large scale. Algorithm: generate L random symbols from a CSPRNG, INSERT with UNIQUE(code); on conflict, retry (expected retries near 1 while the load factor is low). Complexity: O(1) expected per code.

### 5.2 Countdown Computation

Store starts_at as TIMESTAMPTZ (UTC) plus an IANA timezone (Africa/Lagos = WAT, UTC+1, no DST). Countdown = starts_at - server_now, computed server-side and sent as an absolute UTC instant so clients render locally without trusting device clocks. Display in the event's timezone for consistency; international guests see their local equivalent. Complexity O(1).

### 5.3 Feed Ranking ("This week")

Sort candidate events (starts_at within [now, now+7d], user is host or attendee) by a key: primary start_time ascending, with a hosting-priority boost so the user's hosted events surface first when times are close. Score = starts_at_epoch - (is_host ? boost : 0). Sort is O(n log n).

### 5.4 Avatar Cluster Selection

For the "+N going" cluster, take the first k (e.g. 5) going guests ordered by RSVP recency or social affinity, render overlapping avatars, and show "+ (going_count - k)". O(k) after an indexed query.

### 5.5 Reminder Scheduling

On publish, insert reminder_jobs at computed run_at (e.g. starts_at - 7d, starts_at - 2h, matching Partiful's cadence). Worker loop: `SELECT ... WHERE status='pending' AND run_at <= now() ORDER BY run_at LIMIT batch FOR UPDATE SKIP LOCKED`, send, mark done. DB polling with SKIP LOCKED (optionally LISTEN/NOTIFY to cut latency). Each poll O(batch).

### 5.6 Chat Pagination

Keyset/cursor pagination on (event_id, seq): `WHERE event_id=E AND seq < cursor ORDER BY seq DESC LIMIT k`. O(k) with the index, stable under inserts, no OFFSET drift.

### 5.7 Rate Limiting (token bucket)

Per key (user or IP): bucket of capacity C refilling at r tokens/sec. On request, refill by elapsed*r capped at C, allow if >=1 token and decrement, else 429. O(1) per request; state in Redis or in-memory.

### 5.8 Idempotent RSVP Upsert

```sql
INSERT INTO rsvps (event_id, user_id, status, plus_ones, invite_id)
VALUES ($1,$2,$3,$4,$5)
ON CONFLICT (event_id, user_id)
DO UPDATE SET status = EXCLUDED.status,
              plus_ones = EXCLUDED.plus_ones,
              updated_at = now()
RETURNING *;
```
Wrapped in a transaction that first locks the event row and checks the capacity guard for transitions into going. O(1).

---

## 6. Claude Code Handoff (CLAUDE.md)

### 6.1 CLAUDE.md (ready to paste)

```markdown
# CLAUDE.md: Gathr Engineering Rules

## Golden rules
- No code comments anywhere. Code must be self-documenting through precise naming.
- Small, focused modules. One responsibility per file. No file over ~300 lines.
- No dead code. No unused deps. No commented-out code. Delete, don't disable.
- Self-review before finishing: would a senior engineer merge this as-is?

## Rust conventions
- Actix Web 4.x (4.14.1+), SQLx 0.8.6, tokio. Edition 2021. MSRV Rust 1.88+.
- Error handling: `thiserror` for typed library/domain errors at crate boundaries; `anyhow` only in binaries (main, worker) at the top level. Handlers map domain errors to the JSON error envelope.
- Domain crate has zero framework/IO deps. IO lives behind traits in infra crates.
- All SQL via sqlx compile-time checked macros (query!, query_as!). No string-built queries except audited dynamic filters. Commit the .sqlx offline cache.
- Every mutating endpoint honors Idempotency-Key. Every capacity-affecting write uses a transaction with row locks.
- `tracing` spans on every handler. No println.

## Swift conventions
- Swift 6, strict concurrency complete, zero concurrency warnings. iOS 17.0 deployment target (D4); gate newer APIs with @available.
- @Observable for models with logic; no ObservableObject. Value types Sendable.
- Typed errors (enums) at service boundaries. No force-unwraps in non-test code.
- Feature = SPM package. Views depend on protocols injected via environment.
- Networking and persistence in actors. Views never touch URLSession/DB directly.
- No literal colors, fonts, radii, or spacing outside DesignSystem. Section 7 tokens only.
- No hardcoded user-facing strings. String catalog from the first commit.
- Dates formatted with Date.FormatStyle in the event's timezone. Never a format string.

## Non-negotiables
- Every capacity-affecting write goes through the single CAP guard (Section 4.1). One implementation, one call site pattern, integration-tested under concurrency.
- Idempotency keys are generated once at enqueue and reused across retries. Never regenerated per attempt.
- Phone numbers never appear in a guest-visible DTO. Enforced by a serialization test, not by review.
- Authorization goes through can_manage(event, user), never a bare host_id comparison (migration 0007).
- Every icon-only control has an accessibility label. Every card is one VoiceOver element.
- New endpoint => row in the Section 12.3 table, error codes in 12.2, integration test, all in the same commit.

## Naming
- Rust: snake_case fns, CamelCase types, verbs for functions (create_event), nouns for types.
- Swift: lowerCamelCase, descriptive names, no abbreviations.

## Commits
- Conventional Commits: feat:, fix:, refactor:, test:, chore:, docs:. Imperative mood. One logical change per commit.

## Folder structure
- Enforce the workspace layout (Section 2.1) and SPM layout (Section 3.2). New code goes in the correct crate/package or it does not merge.

## Testing (required)
- Rust: unit tests on domain; integration tests with testcontainers Postgres. New endpoint => integration test.
- Swift: Swift Testing for view models/services with fakes. New feature => tests.
- CI must be green: fmt, clippy -D warnings, tests, sqlx prepare check.
```

### 6.2 Suggested Claude Code Skills / Slash-Commands

- `/design-system`: generate or update DesignSystem components from the token spec (colors, type scale, card, chip, avatar cluster).
- `/api-contract-sync`: keep the Rust DTOs and the Swift Models/Networking layer in sync from a single source (OpenAPI or a shared schema); regenerate Swift endpoints when the endpoint table changes.
- `/migration`: scaffold a new sqlx migration plus the matching schema doc update and DDL constraints.
- `/new-feature`: scaffold a new SPM feature package (Router, @Observable model, protocol, tests).
- `/endpoint`: scaffold an Actix handler + DTO + integration test + endpoint table row.

### 6.3 Phased Build Plan

Phases end on the metric gates in Section 23, not on features shipped. Feature scope per phase is governed by the doctrine in Section 22.

Phase 0: Foundations
- Backend: workspace scaffold, config, tracing, Docker + cargo-chef, Fly.io/Caddy deploy of a health check to `jnb`, Postgres + first migration, CI (fmt/clippy/test/sqlx-prepare).
- Platform (L0, Section 24): feature flag service with kill switch, server-driven config endpoint, min-supported-client version check with a forced-upgrade screen. These three ship before any feature does.
- iOS: app scaffold, SPM packages, DesignSystem tokens, Router, string catalog, Swift Testing setup.

Phase 1: Auth + Users
- Backend: Sign in with Apple, OTP request/verify, JWT + refresh rotation (argon2 where needed), devices endpoints, users/me.
- iOS: onboarding carousel, sign-in, token storage in Keychain, deferred-auth guest path.

Phase 2: Events + Invitations (MVP core)
- Backend: event CRUD, publish/cancel, media presign (R2), invite codes, feeds (this_week/hosting/attending).
- iOS: home screen (This week cards, greeting, quick actions), create event with templates + cover upload, event detail, invite share (link/code/QR), universal links.

Phase 3: RSVP + Guests + Reminders
- Backend: idempotent RSVP upsert with capacity transaction, waitlist, guest list/manage, reminder_jobs + worker + APNs (a2).
- iOS: RSVP flow (Going/Maybe/Can't Go + plus-ones), guest management, countdown, push registration + handling, offline outbox for RSVP.

Phase 4: Chat
- Backend: messages table + per-event seq, actix-ws endpoint, broadcast fan-out, keyset history.
- iOS: chat UI, URLSessionWebSocketTask actor client, reconnect, offline queue for messages.

Phase 5: Hardening
- Rate limiting, idempotency store, observability dashboards, load tests, integration test coverage, accessibility pass, App Store prep.

---

## 7. Design System Specification

Tokens below are read off the three reference mockups (onboarding, home, event detail). Everything in `DesignSystem` is a token; no literal colors, sizes, or radii anywhere in feature packages. Anything marked *(extension)* is not in the mockups and is designed to match.

### 7.1 Color Tokens

| Token | Light | Dark *(extension)* | Usage |
|---|---|---|---|
| `canvas` | `#F2F3F5` | `#0B0B0C` | Screen background |
| `surface` | `#FFFFFF` | `#1C1C1E` | Cards, tiles, list rows, tab bar |
| `surfaceInset` | `#F0F1F3` | `#2C2C2E` | Countdown segments, About box |
| `surfaceInsetActive` | `#E7E8EB` | `#3A3A3C` | Focused countdown segment ("09 Days") |
| `textPrimary` | `#111214` | `#FFFFFF` | Titles, body |
| `textSecondary` | `#8E8E93` | `#98989F` | Meta rows, subtitles, onboarding subhead |
| `textTertiary` | `#B0B0B5` | `#6C6C70` | Disabled, placeholder |
| `accent` | `#007AFF` | `#0A84FF` | Primary button, "See all", "Manage", "Skip", FAB |
| `accentPressed` | `#0062CC` | `#3395FF` | Pressed state |
| `onAccent` | `#FFFFFF` | `#FFFFFF` | Label on accent |
| `separator` | `#E5E5EA` | `#38383A` | Hairlines, tab bar top edge |
| `onPhoto` | `#FFFFFF` | `#FFFFFF` | Text over cover imagery |
| `glassChip` | `rgba(28,28,30,0.45)` + 20pt blur | same | "Hosting", "BIRTHDAY" chips over photos |
| `pillOnPhoto` | `rgba(255,255,255,0.92)` | same | "In 9 days" countdown pill |

Status colors *(extension — required by the RSVP sheet and guest list, absent from mockups)*:

| Token | Value | Meaning |
|---|---|---|
| `statusGoing` | `#34C759` | Going |
| `statusMaybe` | `#FF9F0A` | Maybe |
| `statusDeclined` | `#FF3B30` | Can't Go |
| `statusWaitlisted` | `#AF52DE` | Waitlisted |
| `statusInvited` | `#8E8E93` | Invited, no response |

Photo scrim: vertical linear gradient `rgba(0,0,0,0)` at 45% height to `rgba(0,0,0,0.60)` at 100%. The scrim is mandatory, not decorative — it is what makes `onPhoto` text meet 4.5:1 contrast over arbitrary user-uploaded imagery (see 15.1).

### 7.2 Type Scale

System font (SF Pro), Dynamic Type enabled on every token via `relativeTo:`.

| Token | Size / Weight | Tracking | SwiftUI base | Usage in mockups |
|---|---|---|---|---|
| `display` | 28 / Bold | -0.2 | `.title` | "Create beautiful invitations" |
| `titleL` | 24 / Bold | -0.3 | `.title2` | Hero card title "Amara's 26th Birthday" |
| `titleM` | 22 / Bold | -0.2 | `.title3` | Event detail title |
| `titleS` | 20 / Bold | 0 | `.title3` | Section headers, greeting name "Dara" |
| `headline` | 17 / Semibold | 0 | `.headline` | List row titles, button labels |
| `body` | 16 / Regular | 0 | `.body` | About copy |
| `subhead` | 15 / Regular | 0 | `.subheadline` | Onboarding subtitle |
| `footnote` | 13 / Regular | 0 | `.footnote` | Date/location meta, tile subtitles |
| `eyebrow` | 12 / Medium, uppercase | +0.6 | `.caption` | "GOOD EVENING", "EVENT STARTS IN" |
| `chip` | 11 / Semibold, uppercase | +0.5 | `.caption2` | "BIRTHDAY", "Hosting" |
| `numeral` | 20 / Semibold, **monospaced digits** | 0 | `.title3.monospacedDigit()` | Countdown "09 / 14 / 32" |

`numeral` must use `.monospacedDigit()`. Proportional digits make the countdown reflow on every tick, which reads as a visual stutter on the highest-value screen in the app.

All type is **SF Pro Rounded at weight `.medium`** — one family, one weight, hierarchy carried by
size alone. `Typography` is the only place a font is constructed; call sites that previously bolted
`.weight(...)` onto a token have been stripped, because a second weight defeats the point.

### 7.3 Spacing, Radii, Elevation

Base unit 4pt. `gutter` 20 · `cardPadding` 16 · `stackGap` 12 · `sectionGap` 28 · `rowHeight` 64.

Radii: `hero` 20 · `card` 20 · `tile` 16 · `image` 16 · `thumb` 10 · `pill` = height/2.

Elevation: `card` = `0 4 16 rgba(0,0,0,0.06)` · `raised` = `0 8 24 rgba(0,0,0,0.10)` · `fab` = `0 6 16 rgba(0,122,255,0.35)`.

Minimum hit target 44×44 for every interactive element, including the "…" overflow on the hero card and the carousel page dots.

### 7.4 Component Inventory

Each is a public type in `DesignSystem` with a snapshot test (3.8) and previews for light/dark, Dynamic Type XL and AX3, and long-content overflow.

| Component | Anatomy | Notes |
|---|---|---|
| `EventPosterCard` | 3:4 portrait, role badge top-left (crown + "Hosting"), centered bottom stack: avatar cluster → title → date → location, all `onPhoto` | Onboarding carousel. Side cards render at 0.88 scale, 60% opacity |
| `EventHeroCard` | 4:3, category chip top-left, overflow top-right, title (2 lines max, truncating), date row, location row, bottom row: avatar cluster + "+N going" on the left, countdown pill on the right | "This week" carousel |
| `EventListRow` | 44×44 thumb (`thumb` radius) · title `headline` · subtitle `footnote` secondary · chevron | Hosting / Attending lists |
| `CategoryChip` | SF Symbol + uppercase label | `glass` variant over photos, `tinted` variant on `surface` |
| `RoleBadge` | crown glyph + "Hosting" | Glass only |
| `CountdownPill` | Single relative string, "In 9 days" | Card-level. Recomputed on the minute |
| `CountdownSegments` | 3 equal segments Days/Hours/Mins, first segment on `surfaceInsetActive` | Detail-level. One `TimelineView(.periodic(by: 60))` per screen, never per segment |
| `AvatarCluster` | Overlapping circles, −8pt overlap, 2pt `surface` ring | `k` configurable: 4 on cards (linear), 6 on detail (staggered 3+3) |
| `QuickActionTile` | Icon · title `headline` · subtitle `footnote` | Two-up grid, equal height |
| `SectionHeader` | `titleS` + trailing `accent` action | "This week / See all", "Guests / Manage" |
| `PrimaryButton` | 56pt height, `onAccent` on `accent` | `.pill` (onboarding) and `.rounded` (detail action bar) variants |
| `SecondaryButton` | 56pt, `textPrimary` on `surfaceInset` | "Your RSVP" |
| `ActionBar` | Bottom-pinned pair, safe-area inset, `surface` with top hairline | Event detail |
| `TabBar` | 4 slots + detached FAB | See D1 in Section 17 |
| `PageDots` | Active = `accent` 8pt capsule, inactive 6pt `textTertiary` | Onboarding + carousel |

### 7.5 Category Taxonomy

`events.category` is currently free `TEXT` (2.10) but the UI needs a stable icon and tint per value. Constrain it to this closed set at the application layer (not a Postgres enum — new categories should not require a migration), and default unknown values to `other`.

| Value | Label | SF Symbol | Tint |
|---|---|---|---|
| `birthday` | BIRTHDAY | `birthday.cake.fill` | `#FF375F` |
| `party` | PARTY | `party.popper.fill` | `#AF52DE` |
| `meetup` | MEETUP | `person.3.fill` | `#0A84FF` |
| `dinner` | DINNER | `fork.knife` | `#FF9500` |
| `game_night` | GAME NIGHT | `gamecontroller.fill` | `#30D158` |
| `wedding` | WEDDING | `heart.fill` | `#FF2D55` |
| `other` | EVENT | `calendar` | `#8E8E93` |

### 7.6 Motion

Carousel paging: `.interactiveSpring(response: 0.35, dampingFraction: 0.86)`. Card press: scale 0.97, 120ms. Sheet presentation: system. Countdown segment value change: no animation (a number that animates on every tick is noise). All motion respects `accessibilityReduceMotion`, which disables the onboarding parallax and card press scaling.

---


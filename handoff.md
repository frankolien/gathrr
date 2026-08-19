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


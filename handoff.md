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


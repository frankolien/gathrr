# Specification Pack: Gathr, a Mobile Invite and RSVP App

Backend: Rust + Actix Web. Client: SwiftUI iOS. Context: Lagos, Nigeria (WAT, UTC+1).

## TL;DR

- Build an offline-first iOS app on a Rust/Actix Web + Postgres backend, benchmarked against Partiful (Going/Maybe/Can't Go, text-blast reminders, capacity with waitlists, no app needed to RSVP), Luma, and Apple Invites (launched February 4, 2025).
- Recommended stack: Actix Web 4.x (latest 4.14.1, MSRV Rust 1.88) with a Cargo workspace (hexagonal layout), SQLx 0.8.6 with Postgres, actix-ws for chat, the a2 crate for APNs, Cloudflare R2 for images via presigned URLs, and Postgres FOR UPDATE SKIP LOCKED for reminder jobs.
- iOS: Swift 6 strict concurrency, @Observable, NavigationStack with a router pattern, SPM-modularized packages, and GRDB/SQLiteData for the offline cache. Deployment floor iOS 17.0 (Decision D4).
- Sections 7-21 turn the spec into a buildable plan: design tokens read off the mockups, specs for the nine screens the mockups omit, the web RSVP path that the conversion thesis depends on, the offline sync protocol, Lagos-specific delivery and network constraints, and a migration ledger. Section 17 settles every conflict between the mockups and the written spec; Section 18 is the cut line if there is a demo deadline.


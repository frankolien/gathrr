# CLAUDE.md: Gathr Engineering Rules

Full specification: `handoff.md`. Section references below point there.

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


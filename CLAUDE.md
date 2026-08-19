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


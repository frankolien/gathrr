# gathr-backend

Rust + Actix Web backend for Gathr. Specification: `../handoff.md`. Engineering rules: `../CLAUDE.md`.

## Run it

```bash
docker compose up -d
cp .env.example .env          # already present in dev
cargo sqlx migrate run        # or let the server do it on boot
cargo run -p gathr-api
```

The server applies migrations on startup and listens on `GATHR_BIND_ADDR` (default `127.0.0.1:8080`).

Postgres binds `127.0.0.1:55432` rather than the usual 5432/5433 because those ports are commonly
occupied by a native install; `DATABASE_URL` in `.env` matches.

## The demo, in four calls

```bash
API=http://127.0.0.1:8080

TOKEN=$(curl -s -X POST $API/v1/auth/dev -H 'content-type: application/json' \
  -d '{"display_name":"Amara Chukwu"}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["access_token"])')

EVENT=$(curl -s -X POST $API/v1/events -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' -H "idempotency-key: $(uuidgen)" \
  -d '{"title":"Amara'"'"'s 26th Birthday","category":"birthday",
       "location_name":"Victoria Island, Lagos","starts_at":"2026-09-08T18:00:00Z",
       "publish_now":true}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["id"])')

curl -s -X POST $API/v1/events/$EVENT/invites -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' -d '{}'
```

Open the returned `url` in a browser. RSVP with no account and no app, then read the guest list back
with `GET /v1/events/{id}/guests`.

## Tests

```bash
cargo test --workspace          # unit + integration; integration needs Postgres up
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo sqlx prepare --workspace --check -- --all-targets
```

`crates/application/tests/capacity.rs` includes the concurrency test that fires 20 simultaneous
RSVPs at a capacity-5 event and asserts exactly 5 are admitted.


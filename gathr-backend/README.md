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


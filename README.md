# module-fpnotes

Google Keep-style notes module for [FuzzyPeanut](https://github.com/fuzzypeanut). Create notes and todo lists, color-label them, pin them, and share them with other users at view or edit permission.

**This module is also the official FuzzyPeanut SDK tutorial.** The step-by-step guide at [fuzzypeanut.io/developers/build-a-module](https://fuzzypeanut.io/developers/build-a-module) walks through building this module from scratch.

---

## Features

- Rich-text notes and checklist-style todo lists
- Color labels, pinning, and archiving
- Per-note sharing with view or edit permission
- Real-time share notifications via the SDK event bus
- Optional contacts picker integration (requires `module-contacts`)
- Full provisioning endpoint — user data is cleaned up automatically on account deletion

## Stack

| Layer | Technology |
|---|---|
| Frontend | Svelte 5 (runes), Vite library mode |
| Backend | Rust / axum 0.8 / sqlx 0.8 |
| Database | PostgreSQL 16 |
| Auth | Authentik OIDC (via `@fuzzypeanut/sdk`) |

## Quick start

**Prerequisites:** A running FuzzyPeanut stack. See the [Quick Start guide](https://fuzzypeanut.io/get-started).

```bash
# 1. Clone this repo into your groupware directory
git clone git@github.com:fuzzypeanut/module-fpnotes.git

# 2. Copy and fill in environment variables
cp .env.example .env
# Edit .env — set JWKS_URL, PROVISIONING_SECRET, FPNOTES_DB_PASSWORD

# 3. Bring it up alongside your existing stack
docker compose \
  -f ../deploy/docker-compose.dev.yml \
  -f docker-compose.yml \
  up -d
```

The module registers itself with the shell automatically. "Notes" will appear in the nav within a few seconds.

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `JWKS_URL` | Yes | Authentik JWKS endpoint: `https://auth.yourdomain.com/application/o/fuzzypeanut/jwks/` |
| `PROVISIONING_SECRET` | Yes | Shared secret — must match `PROVISIONING_SECRET` in your main stack `.env` |
| `FPNOTES_DB_PASSWORD` | Yes | PostgreSQL password for the notes database |
| `FPNOTES_UI_PORT` | No | Host port for the UI container (default: `4001`) |
| `FPNOTES_API_PORT` | No | Host port for the API container (default: `4002`) |
| `VITE_NOTES_API_URL` | No | Browser-accessible URL for the API (default: `http://localhost:4002`) |

## SDK events

| Direction | Event | Payload |
|---|---|---|
| Emits | `notes:shared` | `{ noteId, sharedWithEmail }` |
| Listens | `contacts:picked` | Used by the share modal's contact picker |

## Provisioning

The module implements the standard FuzzyPeanut provisioning contract:

```
POST   /admin/provision-user          (Header: X-Provisioning-Secret)
DELETE /admin/provision-user/{uid}    (Header: X-Provisioning-Secret)
```

These endpoints are internal-only and must not be exposed through nginx.

## Development

```bash
# Install frontend dependencies
npm install

# Run the Vite dev server (outputs to dist/ for use with the shell)
npm run dev

# Build the production bundle
npm run build
```

The backend runs standalone:

```bash
cd api
pip install -r requirements.txt
DATABASE_URL=... JWKS_URL=... uvicorn main:app --reload
```

## License

MIT

# Pebble Web

A self-hosted web-based email client with Docker deployment support. Derived from the [Pebble](https://github.com/QingJ01/Pebble) desktop email client.

## Features

- Web-based email access (inbox, folders, threads, compose)
- IMAP/SMTP support (Gmail, Outlook, generic IMAP)
- Full-text search (Tantivy)
- Background email sync
- Real-time notifications via WebSocket
- Email translation
- Label management
- Dark mode
- Bilingual UI (English / 中文)
- Docker one-command deployment

## Quick Start

### Docker Compose (Recommended)

```bash
git clone https://github.com/QingJ01/Pebble-Web.git
cd Pebble-Web
cp .env.example .env
# Edit .env: set PEBBLE_PASSWORD and PEBBLE_JWT_SECRET
docker-compose up -d
```

Access at http://localhost:8080

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PEBBLE_PASSWORD` | Yes | — | Login password |
| `PEBBLE_JWT_SECRET` | Yes | — | JWT signing secret (random string) |
| `PEBBLE_PORT` | No | 8080 | Server port |
| `PEBBLE_DATA_DIR` | No | /data | Data directory path |
| `PEBBLE_SYNC_INTERVAL` | No | 300 | IMAP sync interval (seconds) |
| `PEBBLE_ENCRYPTION_KEY` | No | auto-generated | Hex-encoded 32-byte encryption key |

### Manual Build

**Prerequisites:** Rust 1.80+, Node.js 20+

```bash
# Build frontend
cd frontend && npm install && npm run build && cd ..

# Build backend
cargo build --release

# Run
export PEBBLE_PASSWORD=your-password
export PEBBLE_JWT_SECRET=your-secret
export PEBBLE_DATA_DIR=./data
export PEBBLE_STATIC_DIR=./frontend/dist
./target/release/pebble-web
```

## Architecture

```
Axum HTTP Server
├── Static Files (React SPA)
├── REST API (/api/v1/...)
├── WebSocket (/api/v1/ws)
└── Background Sync Workers
    ├── pebble-store (SQLite)
    ├── pebble-mail (IMAP/SMTP)
    ├── pebble-search (Tantivy)
    ├── pebble-crypto (AES-256-GCM)
    └── pebble-translate
```

## API Overview

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/auth/login | Login (returns JWT) |
| GET | /api/v1/accounts | List email accounts |
| POST | /api/v1/accounts | Add email account |
| GET | /api/v1/accounts/:id/folders | List folders |
| GET | /api/v1/folders/:id/messages | List messages |
| GET | /api/v1/messages/:id | Get message detail |
| PUT | /api/v1/messages/:id/flags | Update read/star flags |
| POST | /api/v1/compose | Send email |
| POST | /api/v1/search | Full-text search |
| POST | /api/v1/sync/trigger | Trigger manual sync |
| WS | /api/v1/ws?token=JWT | Real-time notifications |

## Data Storage

All data is stored in the `/data` volume:
- `pebble.db` — SQLite database
- `index/` — Tantivy full-text search index
- `attachments/` — Downloaded attachment files
- `encryption.key` — Auto-generated encryption key

## License

MIT

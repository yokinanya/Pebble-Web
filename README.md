# Pebble Web

A self-hosted web-based email client with Docker deployment support. Derived from the [Pebble](https://github.com/QingJ01/Pebble) desktop email client.

## Features

- **Email Management** — Inbox, folders, threads, starred messages, archive, trash
- **Compose** — Rich text / Markdown / HTML editor, attachments, reply/forward, templates
- **IMAP/SMTP** — Gmail, Outlook, and generic IMAP providers
- **Full-Text Search** — Powered by Tantivy, with advanced filters (from, to, date, attachment)
- **Background Sync** — Automatic IMAP sync with configurable interval
- **Real-Time Notifications** — WebSocket push on new mail and sync events
- **Batch Operations** — Multi-select archive, delete, mark read/unread, star
- **Translation** — Integrated translation (DeepLX, DeepL, Generic API, LLM)
- **Labels** — Custom label creation and per-message tagging
- **Attachments** — Download and inline preview, web-based upload staging
- **Privacy** — Configurable remote image blocking and tracker detection
- **Dark Mode** — System-aware theme with manual override
- **Bilingual UI** — English / 中文
- **Docker Deployment** — Single-command deployment with multi-stage build

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

### Manual Build

**Prerequisites:** Rust 1.80+, Node.js 20+

```bash
# Build frontend
cd frontend && npm install && npm run build && cd ..

# Build backend
cargo build --release

# Run
export PEBBLE_PASSWORD=your-password
export PEBBLE_JWT_SECRET=your-random-secret-at-least-32-chars
export PEBBLE_DATA_DIR=./data
export PEBBLE_STATIC_DIR=./frontend/dist
./target/release/pebble-web
```

### Development

```bash
# Terminal 1: Backend (auto-reload with cargo-watch)
cargo run

# Terminal 2: Frontend dev server (proxies API to backend)
cd frontend && npm run dev
```

The Vite dev server proxies `/api` requests to the backend at `http://localhost:8080`.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PEBBLE_PASSWORD` | Yes | — | Login password |
| `PEBBLE_JWT_SECRET` | Yes | — | JWT signing secret (min 32 chars) |
| `PEBBLE_PORT` | No | `8080` | Server port |
| `PEBBLE_DATA_DIR` | No | `/data` | Data directory path |
| `PEBBLE_STATIC_DIR` | No | `./frontend/dist` | Frontend static files path |
| `PEBBLE_SYNC_INTERVAL` | No | `300` | IMAP sync interval in seconds |
| `PEBBLE_ENCRYPTION_KEY` | No | auto-generated | Hex-encoded 32-byte key for credential encryption |

## Architecture

```
┌────────────────────────────────────────────────┐
│                  Browser (SPA)                 │
│  React 19 · Zustand · React Query · TipTap    │
│  Tailwind CSS · i18next · Vite                 │
└──────────────┬──────────────┬──────────────────┘
               │ HTTP REST    │ WebSocket
┌──────────────▼──────────────▼──────────────────┐
│              Axum HTTP Server                  │
│  ┌──────────┐ ┌───────────┐ ┌───────────────┐ │
│  │ REST API │ │ WebSocket │ │ Static Files  │ │
│  │ (JWT)    │ │ (realtime)│ │ (SPA fallback)│ │
│  └────┬─────┘ └─────┬─────┘ └───────────────┘ │
│       │              │                          │
│  ┌────▼──────────────▼──────────────────────┐  │
│  │         Background Sync Workers          │  │
│  │    (per-account IMAP IDLE / polling)     │  │
│  └──────────────────────────────────────────┘  │
│                                                │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────┐ │
│  │pebble-store │ │pebble-search│ │pebble-   │ │
│  │  (SQLite)   │ │  (Tantivy)  │ │ crypto   │ │
│  └─────────────┘ └─────────────┘ │(AES-256) │ │
│  ┌─────────────┐ ┌─────────────┐ └──────────┘ │
│  │ pebble-mail │ │pebble-      │               │
│  │(IMAP/SMTP)  │ │ translate   │               │
│  └─────────────┘ └─────────────┘               │
└────────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 19, TypeScript, Vite, Tailwind CSS 4 |
| State Management | Zustand 5, TanStack React Query 5 |
| Rich Text Editor | TipTap 3 (with Markdown support) |
| Backend | Rust, Axum 0.8, Tokio |
| Database | SQLite (rusqlite) |
| Search | Tantivy 0.22 |
| Email | async-imap, Lettre (SMTP) |
| Encryption | AES-256-GCM (aes-gcm) |
| Auth | Argon2 password hashing, JWT |
| Translation | DeepLX / DeepL / Generic API / LLM |
| Deployment | Docker multi-stage build, Alpine Linux |

## API Reference

All endpoints except `/health`, `/auth/login`, and `/ws` require a JWT token in the `Authorization: Bearer <token>` header.

### Auth

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/login` | Login, returns JWT token |
| GET | `/api/v1/health` | Health check |

### Accounts

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/accounts` | List all email accounts |
| POST | `/api/v1/accounts` | Add email account (IMAP/SMTP) |
| PUT | `/api/v1/accounts/:id` | Update account settings |
| DELETE | `/api/v1/accounts/:id` | Delete account and all data |
| POST | `/api/v1/accounts/:id/test-connection` | Test IMAP connection (existing account) |
| POST | `/api/v1/test-imap-connection` | Test IMAP connection (raw credentials) |

### Folders

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/accounts/:id/folders` | List folders for account |
| GET | `/api/v1/accounts/:id/folder-unread-counts` | Get unread counts per folder |

### Messages

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/folders/:id/messages` | List messages in folder (paginated) |
| GET | `/api/v1/messages/:id` | Get message metadata |
| POST | `/api/v1/messages/:id/with-html` | Get message with rendered HTML body |
| POST | `/api/v1/messages/:id/render` | Re-render HTML with privacy settings |
| PUT | `/api/v1/messages/:id/flags` | Update read/starred flags |
| POST | `/api/v1/messages/:id/move` | Move message to folder |
| POST | `/api/v1/messages/:id/archive` | Archive message |
| POST | `/api/v1/messages/:id/restore` | Restore from trash |
| DELETE | `/api/v1/messages/:id` | Soft-delete message |
| GET | `/api/v1/accounts/:id/starred` | List starred messages |
| POST | `/api/v1/accounts/:id/empty-trash` | Empty trash folder |

### Batch Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/messages/batch` | Get multiple messages by IDs |
| POST | `/api/v1/messages/batch/archive` | Batch archive |
| POST | `/api/v1/messages/batch/delete` | Batch delete |
| POST | `/api/v1/messages/batch/mark-read` | Batch mark read/unread |
| POST | `/api/v1/messages/batch/star` | Batch star/unstar |

### Threads

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/folders/:id/threads` | List threads in folder |
| GET | `/api/v1/threads/:id/messages` | Get all messages in thread |

### Search

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/search` | Full-text search with optional filters |

### Attachments

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/messages/:id/attachments` | List message attachments |
| GET | `/api/v1/attachments/:id/download` | Download attachment file |

### Compose

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/compose` | Send email via SMTP |
| POST | `/api/v1/compose/attachment` | Stage attachment (base64 upload) |

### Labels

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/labels` | List all labels |
| POST | `/api/v1/labels` | Create label |
| DELETE | `/api/v1/labels/:id` | Delete label |
| POST | `/api/v1/messages/:id/labels` | Add label to message |
| DELETE | `/api/v1/messages/:id/labels/:label_id` | Remove label from message |

### Translation

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/translate` | Translate text |
| GET | `/api/v1/translate/config` | Get translation config |
| POST | `/api/v1/translate/config` | Save translation config |
| POST | `/api/v1/translate/test` | Test translation provider |

### Sync

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/sync/trigger` | Trigger manual IMAP sync |
| GET | `/api/v1/pending-ops/summary` | Pending operations summary |
| GET | `/api/v1/pending-ops` | List pending operations |

### WebSocket

| Protocol | Endpoint | Description |
|----------|----------|-------------|
| WS | `/api/v1/ws` | Real-time notifications (first message = JWT token) |

Events: `sync_complete`, `new_mail`, `authenticated`, `error`

## Data Storage

All data is stored in the configured data directory (`PEBBLE_DATA_DIR`):

```
data/
├── pebble.db          # SQLite database (accounts, messages, folders, labels)
├── index/             # Tantivy full-text search index
├── attachments/       # Downloaded and staged attachment files
│   └── staging/       # Temporary upload staging area
└── encryption.key     # Auto-generated AES-256 encryption key
```

## Project Structure

```
Pebble-Web/
├── src/                    # Rust backend
│   ├── main.rs             # Entry point, server startup
│   ├── config.rs           # Environment config validation
│   ├── auth.rs             # Argon2 hashing, JWT, auth middleware
│   ├── state.rs            # Shared application state
│   ├── error.rs            # API error types
│   ├── credentials.rs      # Credential encryption/decryption
│   ├── sync.rs             # Background IMAP sync manager
│   ├── ws.rs               # WebSocket handler and broadcast
│   └── routes/             # API route handlers
├── crates/                 # Workspace crates
│   ├── pebble-core/        # Shared types and traits
│   ├── pebble-store/       # SQLite storage layer
│   ├── pebble-mail/        # IMAP/SMTP implementation
│   ├── pebble-search/      # Tantivy search engine
│   ├── pebble-crypto/      # AES-256-GCM encryption
│   ├── pebble-translate/   # Translation providers
│   └── pebble-oauth/       # OAuth helpers (desktop only)
├── frontend/               # React SPA
│   ├── src/
│   │   ├── app/            # Layout, hooks, routing
│   │   ├── components/     # Shared UI components
│   │   ├── features/       # Feature modules (inbox, compose, settings, ...)
│   │   ├── hooks/          # React Query hooks, mutations
│   │   ├── stores/         # Zustand state stores
│   │   └── lib/            # API client, utilities, types
│   └── vite.config.ts
├── Dockerfile              # Multi-stage Docker build
├── docker-compose.yml
└── .env.example
```

## License

MIT

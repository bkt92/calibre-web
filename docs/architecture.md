# Architecture Overview

This document describes the high-level architecture of Calibre-Web.

## Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         Web Browser                             │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP/WebSocket
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Web Server Layer                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Gevent / Tornado WSGI Server (cps/server.py)            │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Flask Application                            │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Flask App (cps/__init__.py)                             │  │
│  │  - Blueprints: web, opds, admin, editbooks, etc.        │  │
│  │  - Extensions: Babel, Limiter, Login Manager, CSRF      │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
┌─────────────────┐ ┌──────────────┐  ┌──────────────────────┐
│  CalibreDB      │ │  User DB     │  │  File System         │
│  (Read-Only)    │ │  (Read/Write)│  │  - eBooks            │
│  - Books        │ │  - Users     │  │  - Covers            │
│  - Authors      │ │  - Sessions  │  │  - Thumbnails        │
│  - Custom Cols  │ │  - Shelves   │  │  - Cache             │
└─────────────────┘ └──────────────┘  └──────────────────────┘
         │                   │
         └───────────────────┼───────────────────┐
                             ▼                   ▼
                   ┌─────────────────┐  ┌─────────────────┐
                   │  Calibre DB     │  │  External APIs  │
                   │  (metadata.db)  │  │  - Google Books │
                   └─────────────────┘  │  - Amazon       │
                                         │  - Goodreads    │
                                         └─────────────────┘
```

## Layer Descriptions

### Web Server Layer

**File:** `cps/server.py`

The web server handles incoming HTTP requests and forwards them to the Flask application. It supports:

- **Gevent** (preferred): Async WSGI server with greenlets
- **Tornado** (fallback): Event-driven I/O server

Features:
- SSL/TLS support
- Unix socket support (Linux)
- Systemd socket activation
- Reverse proxy support via `ReverseProxied` middleware

### Flask Application Layer

**File:** `cps/__init__.py`

The Flask application is the core of the web interface:

**Blueprints** (modular route organization):
- `web` - Main web interface (`cps/web.py`)
- `basic` - Basic authenticated routes (`cps/basic.py`)
- `opds` - OPDS feed for eBook readers (`cps/opds.py`)
- `admin` - Admin interface (`cps/admin.py`)
- `editbooks` - Book editing (`cps/editbooks.py`)
- `search` - Search functionality (`cps/search.py`)
- `shelf` - Custom book collections (`cps/shelf.py`)
- `gdrive` - Google Drive integration (`cps/gdrive.py`)
- `kobo` / `kobo_auth` - Kobo device sync (`cps/kobo.py`)
- `oauth` - OAuth authentication (`cps/oauth_bb.py`)

**Key Extensions:**
- Flask-Babel - Internationalization (20+ languages)
- Flask-Limiter - Rate limiting
- Flask-Principal - Permission management
- Flask-WTF - CSRF protection
- Custom login manager (`cps/MyLoginManager.py`)

### Database Layer

Calibre-Web uses two separate databases:

#### CalibreDB (Read-Only)

**File:** `cps/db.py`

Connects to the existing Calibre `metadata.db`:

**Core Tables:**
- `books` - Book records
- `authors` - Author information
- `series` - Series information
- `tags` - Tags/categories
- `ratings` - Rating levels
- `languages` - Language codes
- `publishers` - Publisher information
- `identifiers` - ISBN, UUID, etc.
- `custom_column_*` - Dynamic custom columns

**Association Tables:**
- `books_authors_link` - Many-to-many books↔authors
- `books_series_link` - Many-to-many books↔series
- `books_tags_link` - Many-to-many books↔tags
- `books_ratings_link` - Many-to-many books↔ratings
- `books_languages_link` - Many-to-many books↔languages
- `books_publishers_link` - Many-to-many books↔publishers

**Key Features:**
- Read-only access (Calibre desktop app manages writes)
- Dynamic model creation for custom columns
- Connection pooling with SQLAlchemy
- Thread-safe scoped sessions

#### User Database (Read/Write)

**File:** `cps/ub.py`

Application-specific SQLite database:

**Tables:**
- `User` - User accounts and roles
- `User_Sessions` - Active user sessions
- `RemoteAuthToken` - Remote login tokens
- `Shelf` - Custom book collections
- `BookShelf` - Books on shelves
- `Downloads` - Download history
- `Domain` - LDAP domain configuration
- `OAuth` - OAuth user tokens

**Default location:** `app.db` (configurable via `-p` CLI flag)

### Services Layer

**Directory:** `cps/services/`

Background and external service integrations:

- **background_scheduler.py** - APScheduler for scheduled tasks
- **worker.py** - Background task worker thread
- **simpleldap.py** - LDAP authentication
- **gmail.py** - Email sending via Gmail
- **goodreads_support.py** - Goodreads integration

### Background Tasks

**Directory:** `cps/tasks/`

Long-running operations executed asynchronously:

- **convert.py** - eBook conversion (via Calibre binaries)
- **upload.py** - Book upload processing
- **thumbnail.py** - Cover thumbnail generation
- **metadata_backup.py** - Metadata backup
- **mail.py** - Email queuing and sending
- **clean.py** - Cache cleanup

Tasks are queued via `WorkerThread` and status tracked in `tasks_status.py`.

### Metadata Providers

**Directory:** `cps/metadata_provider/`

Extensible metadata download from external sources:

- Amazon
- Google Books
- ComicVine
- Douban
- Lubimyczytac
- Scholar

Add new providers by creating a new module in this directory.

## Request Flow

### Typical Web Request

```
1. Browser → HTTP Request
   ↓
2. Web Server (Gevent/Tornado)
   ↓
3. ReverseProxied Middleware (if behind proxy)
   ↓
4. Flask App → Blueprint Router
   ↓
5. @login_required Decorator
   ↓
6. Route Handler Function
   ↓
7. Database Query (CalibreDB or User DB)
   ↓
8. Template Rendering (Jinja2)
   ↓
9. Response → Browser
```

### OPDS Feed Request

```
1. eBook Reader → OPDS Request
   ↓
2. Rate Limiter (3 requests/minute)
   ↓
3. OPDS Blueprint Handler
   ↓
4. CalibreDB Query
   ↓
5. XML/Atom Response → Reader
```

### Background Task Flow

```
1. User Action (e.g., convert book)
   ↓
2. Task Queued in WorkerThread
   ↓
3. Task Status Added (tasks_status.py)
   ↓
4. Background Worker Processes Task
   ↓
5. Progress Updates via AJAX
   ↓
6. Completion Notification
```

## Configuration

Configuration is stored in the User DB and loaded via `cps/config_sql.py`:

- Database paths
- Authentication settings
- UI preferences
- Feature flags (Kobo, OAuth, etc.)
- Rate limiting rules
- SSL/TLS settings

See `docs/configuration.md` for details.

## Security

- **CSRF Protection:** Flask-WTF for all state-changing requests
- **Rate Limiting:** Flask-Limiter with configurable rules
- **Session Security:** HTTPOnly, SameSite cookies
- **CSP Headers:** Content Security Policy on all responses
- **Password Hashing:** Werkzeug security functions
- **LDAP/OAuth:** Optional external authentication

See `docs/authentication.md` for details.

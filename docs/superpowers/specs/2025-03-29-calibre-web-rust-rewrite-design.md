# Calibre-Web Rust Rewrite - Software Design Document

**Date:** 2025-03-29
**Author:** Claude Code
**Status:** Draft
**Version:** 1.0

## Executive Summary

This document describes the complete rewrite of Calibre-Web (a Python/Flask eBook library manager) in Rust. The primary goal is **performance and scalability** - handling larger libraries (10,000+ books) with more concurrent users (50+) while using less memory (<1GB target).

**Key Decisions:**
- **Architecture:** Monolithic async application (Axum + Tokio)
- **Database:** PostgreSQL with SQLx (compile-time checked queries)
- **Frontend:** Server-Side Rendering with Tera templates (Jinja2-compatible)
- **Feature Scope:** Full parity with existing Calibre-Web
- **Calibre Compatibility:** Import existing Calibre libraries (standalone system)

---

## Table of Contents

1. [Requirements](#requirements)
2. [Architecture Overview](#architecture-overview)
3. [Database Design](#database-design)
4. [Web Layer Design](#web-layer-design)
5. [Background Tasks](#background-tasks)
6. [Error Handling](#error-handling)
7. [Configuration](#configuration)
8. [Performance Optimizations](#performance-optimizations)
9. [Testing Strategy](#testing-strategy)
10. [Deployment](#deployment)

---

## Requirements

### Functional Requirements

**Must-Have Features (Full Feature Parity):**

1. **Web Interface**
   - Browse books by title, author, series, tags, publisher, language
   - Advanced search with filters
   - Custom book shelves (collections)
   - Book metadata editing
   - Cover image management
   - File uploads
   - Download books in multiple formats

2. **User Management**
   - Local authentication with argon2 password hashing
   - LDAP authentication
   - OAuth (Google, GitHub)
   - Role-based permissions (bitmask-based)
   - User settings (sidebar, language, content filtering)

3. **OPDS Feeds**
   - Full OPDS 1.2 specification
   - Authentication support
   - Rate limiting (3 req/min)

4. **Kobo Device Sync**
   - Metadata sync
   - Reading progress tracking
   - Bookshelf sync

5. **Background Tasks**
   - eBook format conversion (via Calibre binaries)
   - Thumbnail generation
   - Metadata downloading from external sources
   - Email sending
   - Cache cleanup
   - Metadata backup

6. **Admin Interface**
   - User management
   - System configuration
   - Task monitoring
   - Calibre library import

### Non-Functional Requirements

**Performance Targets:**
- Response time: <500ms for page loads
- Memory usage: <1GB for 10,000 books
- Concurrency: 50+ concurrent users
- Startup time: <5 seconds

**Quality Attributes:**
- Type safety (Rust guarantees)
- Memory safety (no GC pauses)
- Security (memory-safe string handling)
- Maintainability (clear architecture)
- Testability (layered design)

---

## Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Axum Web Application                │
│                                                           │
│  ┌───────────────────────────────────────────────────┐  │
│  │  HTTP Layer (Axum)                                │  │
│  │  - Routes (books, auth, admin, OPDS, etc.)       │  │
│  │  - Middleware (auth, rate limiting, CORS, CSRF)   │  │
│  │  - Extractors (user, pagination, files)           │  │
│  │  - Static file serving                            │  │
│  └───────────────────────────────────────────────────┘  │
│                          ↓↑                            │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Domain Layer (Business Logic)                   │  │
│  │  - BookService                                    │  │
│  │  - UserService                                    │  │
│  │  - SearchService                                  │  │
│  │  - ImportService (Calibre)                        │  │
│  │  - AuthService                                    │  │
│  └───────────────────────────────────────────────────┘  │
│                          ↓↑                            │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Infrastructure Layer                             │  │
│  │  ┌──────────────┐  ┌─────────────┐               │  │
│  │  │ PostgreSQL   │  │ Moka Cache  │               │  │
│  │  │ (SQLx)       │  │ (in-memory) │               │  │
│  │  └──────────────┘  └─────────────┘               │  │
│  │  ┌──────────────┐  ┌─────────────┐               │  │
│  │  │ Task Queue   │  │ File System │               │  │
│  │  │ (Tokio)      │  │ (Storage)   │               │  │
│  │  └──────────────┘  └─────────────┘               │  │
│  └───────────────────────────────────────────────────┘  │
│                                                           │
│  External Services:                                      │
│  - Calibre ebook-convert (conversion)                  │
│  - LDAP servers (authentication)                         │
│  - OAuth providers (Google, GitHub)                      │
│  - Metadata APIs (Amazon, Google Books, etc.)          │
└─────────────────────────────────────────────────────────┘
```

### Project Structure

```
calibre-web-rust/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library root
│   ├── web/                    # Web layer (Axum)
│   │   ├── routes/              # Route handlers
│   │   ├── middleware/          # Axum middleware
│   │   └── extractors/          # Custom extractors
│   ├── domain/                 # Business logic
│   │   ├── books/               # Book domain logic
│   │   ├── users/               # User domain logic
│   │   └── search/              # Search functionality
│   ├── infrastructure/         # External integrations
│   │   ├── database/            # SQLx database code
│   │   ├── cache/               # Moka caching
│   │   ├── auth/                # Authentication providers
│   │   ├── tasks/               # Background tasks
│   │   └── storage/             # File storage
│   ├── config/                 # Configuration
│   └── error/                  # Error types
├── templates/                   # Tera templates (Jinja2-compatible)
├── migrations/                   # Database migrations
├── tests/                       # Integration tests
├── Cargo.toml
└── README.md
```

### Technology Stack

| Component | Technology | Justification |
|-----------|-----------|---------------|
| Web Framework | Axum 0.7+ | Type-safe routing, Tower middleware, Tokio-native |
| Async Runtime | Tokio 1.35+ | Proven, performant, excellent ecosystem |
| Database | PostgreSQL 15+ | Advanced features, JSONB, full-text search |
| Database Driver | SQLx 0.7+ | Compile-time query checking, async |
| Templating | Tera 0.20+ | Jinja2-compatible syntax (reuse templates) |
| Auth | tower-authz, openidconnect, ldap3 | Modular, ecosystem-standard |
| Password Hashing | argon2 0.5+ | Memory-hard, best practice |
| Caching | Moka 0.12+ | Fast async in-memory cache |
| Sessions | tower-session + encrypted cookies | No Redis needed |
| HTTP Client | reqwest 0.11+ | Async HTTP for metadata APIs |
| Logging | tracing 0.1+ | Structured logging |
| Configuration | config 0.13+ | Multiple formats, layered |
| Async Tasks | Tokio channels | In-process, no external queue needed |

---

## Database Design

### Schema Overview

The database schema is **Calibre-compatible** for easy importing while optimized for Rust/PostgreSQL.

**Core Calibre Tables (Mirrored):**

- `books` - Main book records
- `authors` - Author information
- `series` - Series information
- `tags` - Tags/categories
- `ratings` - Rating levels (0-10 scale)
- `languages` - Language codes
- `publishers` - Publisher information
- `identifiers` - ISBN, UUID, ASIN, Goodreads, etc.
- `comments` - Book descriptions
- `data` - eBook formats
- `custom_columns` - Dynamic Calibre custom columns
- `books_*_link` - Association tables (many-to-many)

**Application Tables (New):**

- `users` - User accounts with role bitmasks
- `user_sessions` - Session management
- `shelves` - Custom book collections
- `shelf_books` - Books on shelves
- `downloads` - Download history
- `tasks` - Background task tracking
- `config` - Key-value configuration
- `metadata_cache` - Metadata provider cache
- `kobo_synced_books` - Kobo sync state
- `kobo_reading_state` - Kobo reading progress

### Key Design Decisions

1. **Calibre Compatibility:**
   - Same table structure (books, authors, series, etc.)
   - Same data types (TEXT with NOCASE, INTEGER for has_cover)
   - Same identifier system (generic identifiers table)
   - Can import directly from Calibre's SQLite metadata.db

2. **PostgreSQL Optimizations:**
   - GIN indexes for full-text search on titles
   - JSONB for flexible data (tasks, config, metadata)
   - Array types for efficient filtering (denied_tags, allowed_tags)
   - Connection pooling for performance

3. **ID Preservation:**
   - Keep Calibre's INTEGER IDs for books, authors, etc.
   - Use UUID for task IDs (security, distributed systems)
   - Use BIGSERIAL for application tables (users, shelves)

---

## Web Layer Design

### Route Structure

```rust
// Public routes
/                    → Home
/login, /logout       → Authentication
/register            → User registration
/opds, /opds/*path    → OPDS feeds

// Authenticated routes
/books               → Book listing
/books/:id           → Book details
/books/:id/edit      → Edit book
/books/:id/download  → Download book
/upload              → Upload book
/search              → Search interface
/shelves             → Custom collections
/profile             → User profile

// Admin routes
/admin               → Dashboard
/admin/users         → User management
/admin/config        → Configuration
/admin/import        → Calibre import
/admin/tasks         → Task monitoring

// API routes
/api/books           → REST API
/api/tasks/:id       → Task status
```

### Middleware Stack

1. **TraceLayer** - Request tracing/logging
2. **CorsLayer** - CORS handling
3. **SessionManager** - Session management
4. **CompressionLayer** - Response compression
5. **Rate Limiting** - Token bucket algorithm
6. **Authentication** - Session validation, user injection
7. **CSRF Protection** - Token validation for POST/PUT/DELETE

### Custom Extractors

- `AuthenticatedUser` - Current logged-in user
- `OptionalUser` - Current user or anonymous
- `Pagination` - Page/per_page/offset parsing
- `WithRole(role)` - Role-based authorization

### Authentication Flow

```
1. User submits login form
   ↓
2. AuthHandler::login()
   ↓
3. Load user from database
   ↓
4. Verify password (argon2)
   ↓
5. Create session (user_id, csrf_token)
   ↓
6. Redirect to home
```

**Session Storage:** Encrypted cookies (no Redis needed)

**CSRF Protection:** Token in session + header validation

---

## Background Tasks

### Task Queue Architecture

**In-Process Tokio Task Queue:**

```
Web Request → Task Submitted → Channel Queue → Worker Thread
                              ↓
                         Task Database
                              ↓
                         Progress Updates
                              ↓
                         Client Polls
                              ↓
                         Completion
```

**Task Types:**

1. **ConvertFormat** - Convert book formats via Calibre
2. **GenerateThumbnail** - Create cover thumbnails
3. **UploadBook** - Process uploaded eBook files
4. **ImportCalibre** - Import Calibre library
5. **MetadataDownload** - Fetch metadata from providers
6. **SendEmail** - Email books to Kindle
7. **BackupMetadata** - Backup library metadata
8. **CleanCache** - Remove old cache entries

### Task Worker

```rust
// 3 worker threads (configurable)
for worker_id in 0..3 {
    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            handle_task(task).await;
        }
    });
}
```

**Progress Tracking:**

- Tasks stored in database
- Progress updated periodically
- Clients poll via AJAX
- Status: pending → running → completed/failed

---

## Error Handling

### Unified Error Type

```rust
pub enum AppError {
    Database(sqlx::Error),
    NotFound,
    Auth(AuthError),
    Validation(String),
    Io(std::io::Error),
    Task(TaskError),
    ExternalService(String),
    Internal(String),
}
```

**HTTP Status Mappings:**

- Database errors → 500 Internal Server Error
- NotFound → 404 Not Found
- Auth errors → 401 Unauthorized
- Validation → 400 Bad Request
- Forbidden → 403 Forbidden

**Logging:**

All errors logged with `tracing`:
- `error!` for unexpected errors
- `warn!` for authentication failures
- `debug!` for expected conditions

---

## Configuration

### Environment-Based Config

**Priority:** `config/default.toml` → `config/local.toml` → Environment vars

**Key Settings:**

```toml
[database]
url = "postgresql://user:pass@localhost/calibre_web"
max_connections = 10

[server]
host = "0.0.0.0"
port = 8083
workers = 4

[auth]
secret_key = "must-be-32+ chars"
session_timeout = 2592000  # 30 days

[cache]
enabled = true
ttl_seconds = 300
max_capacity = 10000

[library]
library_path = "/var/lib/calibre-web/library"
cover_path = "/var/lib/calibre-web/covers"
upload_path = "/var/lib/calibre-web/upload"

[tasks]
max_concurrent = 3
timeout_seconds = 3600
worker_threads = 2
```

**Secret Key Generation:**

```bash
openssl rand -hex 32
```

---

## Performance Optimizations

### Caching Strategy

**Cache Layers:**

1. **Moka In-Memory Cache**
   - Book lists (5 min TTL)
   - User permissions (5 min TTL)
   - Search results (3 min TTL)
   - Templates (1 hour TTL)

2. **Template Caching**
   - Pre-rendered HTML for slow pages
   - Fragment caching for components

### Database Optimizations

**Connection Pooling:**
- Min connections: 1
- Max connections: 10
- Idle timeout: 10 min
- Max lifetime: 30 min

**Query Optimizations:**
- Prepared statements (SQLx compile-time checking)
- GIN indexes for full-text search
- Pagination with LIMIT/OFFSET
- Batch operations for bulk inserts

### Async Throughout

**Tokio Runtime:**
- All I/O operations async
- Non-blocking database queries
- Concurrent request handling
- Background task processing

---

## Testing Strategy

### Test Pyramid

```
        /\
       /E2E\          ← Few (smoke tests)
      /------\
     /Integration\     ← More (API tests)
    /----------\
   /Unit Tests  \     ← Most (domain logic)
  /--------------\
```

### Unit Tests

**Domain Layer:**
- BookService tests
- UserService tests
- SearchService tests
- Repository tests

### Integration Tests

**Web Layer:**
- Route handler tests
- Middleware tests
- Authentication flow tests

### E2E Tests

**Critical Paths:**
- User registration → login → browse → download
- Admin import Calibre library
- Book upload → convert → thumbnail

---

## Deployment

### Docker Deployment

```yaml
services:
  calibre-web:
    image: calibre-web-rust:latest
    ports:
      - "8083:8083"
    volumes:
      - ./library:/var/lib/calibre-web/library
      - ./covers:/var/lib/calibre-web/covers
    environment:
      - CALIBRE_WEB__DATABASE__URL=postgresql://...
      - CALIBRE_WEB__AUTH__SECRET_KEY=${SECRET_KEY}
    depends_on:
      - postgres
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

### Systemd Service

```ini
[Unit]
Description=Calibre-Web Rust
After=network.target postgresql.service

[Service]
Type=simple
User=calibre-web
WorkingDirectory=/opt/calibre-web-rust
ExecStart=/opt/calibre-web-rust/calibre-web-rust
Restart=always

[Install]
WantedBy=multi-user.target
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 443 ssl http2;
    server_name books.example.com;

    ssl_certificate /etc/letsencrypt/live/books.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/books.example.com/privkey.pem;

    location / {
        proxy_pass http://localhost:8083;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)
- Project setup (Cargo, dependencies)
- Database schema + migrations
- Basic configuration
- Error handling
- Logging infrastructure

### Phase 2: Core Features (Weeks 3-5)
- User authentication (local)
- Book browsing (list, detail, search)
- Database repositories
- Template rendering (Tera)
- Static file serving

### Phase 3: Advanced Features (Weeks 6-8)
- Book editing
- File uploads
- Custom shelves
- User management
- Admin interface basics
- Background task system

### Phase 4: Integrations (Weeks 9-10)
- Calibre library import
- LDAP authentication
- OAuth (Google, GitHub)
- Metadata providers
- Kobo device sync
- Email sending

### Phase 5: Polish & Optimization (Weeks 11-12)
- Caching implementation
- Performance tuning
- Security hardening
- Comprehensive testing
- Documentation

### Phase 6: Deployment & Migration (Weeks 13-14)
- Docker packaging
- Deployment scripts
- Migration tools
- Production deployment
- Monitoring setup

---

## Success Criteria

**Functional:**
- [ ] All core features working
- [ ] Can import existing Calibre libraries
- [ ] OPDS feeds functional with popular readers
- [ ] Background tasks complete successfully

**Performance:**
- [ ] <500ms response time for book listing
- [ ] <1GB memory usage with 10,000 books
- [ ] Handles 50+ concurrent users
- [ ] <5 second startup time

**Quality:**
- [ ] All tests passing
- [ ] No unsafe code where unnecessary
- [ ] Comprehensive error handling
- [ ] Security audit passed

---

## Open Questions

1. **Migration Path:** Should we support running Python and Rust versions simultaneously during migration?
2. **Custom Columns:** How deeply should we support Calibre's custom column system?
3. **Plugin System:** Do we need a plugin architecture for extensibility?
4. **Multi-Language:** Which languages to prioritize for i18n?

---

## Appendix

### A. Calibre Schema Compatibility

See `docs/database.md` for detailed Calibre database schema.

### B. OPDS Specification

See `docs/opds-api.md` for OPDS 1.2 specification details.

### C. Performance Benchmarks

TBD after implementation.

---

**Document Version:** 1.0
**Last Updated:** 2025-03-29

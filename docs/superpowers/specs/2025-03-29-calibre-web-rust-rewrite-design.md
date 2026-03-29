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

### Architecture Decision: Single Source of Truth with Synchronization

**CRITICAL DECISION:** This system uses **PostgreSQL as the single source of truth** with Calibre SQLite (metadata.db) as a portable import/export format. Bidirectional synchronization maintains Calibre Desktop compatibility.

```
┌─────────────────────────────────────────────────────────────┐
│              Calibre-Web Rust Application                 │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  PostgreSQL (Single Source of Truth)              │   │
│  │                                                     │   │
│  │  Books, Authors, Series, Tags                       │   │
│  │  Users, Sessions, Shelves, Tasks                    │   │
│  │  Config, Permissions, Custom Columns                │   │
│  │  All Book Formats and Metadata                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ↑↓                                 │
│              ┌──────────────────────┐                      │
│              │  Sync Layer         │                      │
│              │  (Import/Export)    │                      │
│              └──────────────────────┘                      │
│                          ↑↓                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Calibre SQLite (metadata.db)                      │   │
│  │  - Portable Import/Export Format                     │   │
│  │  - Calibre Desktop Compatibility                    │   │
│  │  - Backup/Restore Capability                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  Sync Modes:                                                │
│  - Import: Calibre → PostgreSQL (initial migration)        │
│  - Export: PostgreSQL → Calibre (backup)                  │
│  - Bidirectional: Keep both in sync (optional)            │
└─────────────────────────────────────────────────────────────┘
```

**Why PostgreSQL as Single Source of Truth?**

1. **Full CRUD Control** - Complete control over book add/edit/delete operations
2. **Performance** - All queries in PostgreSQL (no SQLite limitations)
3. **Concurrency** - No SQLite file locking issues
4. **Scalability** - PostgreSQL handles concurrent writes better
5. **Simpler Architecture** - One primary database, no two-database coordination
6. **Advanced Features** - Full-text search, JSONB, triggers, etc.

**Why Keep Calibre SQLite?**

1. **Calibre Compatibility** - Import/export maintains Calibre Desktop interoperability
2. **Portable Format** - metadata.db is a self-contained library backup
3. **Migration Path** - Easy import from existing Calibre libraries
4. **Fallback Option** - Can export to Calibre format if needed

**Sync Strategy:**

See [Calibre Sync Strategy](./2025-03-29-calibre-sync-strategy.md) for complete details on:
- Bidirectional synchronization algorithm
- Conflict resolution strategies (last-write-wins, PostgreSQL-wins, manual)
- Change detection and incremental sync
- Error handling and rollback
- Performance optimization

**Database Technologies:**
- **Primary DB:** PostgreSQL 15+ (via SQLx) - Single source of truth
- **Portable Format:** SQLite (via rusqlite) - Import/export/sync source

---

### Application Database Schema (PostgreSQL)

**Application Tables:**

```sql
-- User management
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,  -- argon2
    role_bitmask BIGINT NOT NULL DEFAULT 0,
    locale VARCHAR(10) DEFAULT 'en',
    kindle_email VARCHAR(255),
    sidebar_settings BIGINT DEFAULT 0,
    denied_tags TEXT[],  -- Array of tag names for content filtering
    allowed_tags TEXT[],
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP
);

CREATE TABLE user_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_sessions_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_expires ON user_sessions(expires_at);

-- Shelves (custom collections)
CREATE TABLE shelves (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    is_public BOOLEAN DEFAULT FALSE,
    kobo_sync BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE TABLE shelf_books (
    shelf_id INTEGER NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL,  -- References Calibre book ID
    added_at TIMESTAMP NOT NULL DEFAULT NOW(),
    order_index INTEGER DEFAULT 0,
    PRIMARY KEY (shelf_id, book_id)
);

-- Downloads tracking
CREATE TABLE downloads (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL,  -- References Calibre book ID
    format TEXT,
    downloaded_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_downloads_user ON downloads(user_id, downloaded_at DESC);

-- Background tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type TEXT NOT NULL,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    status TEXT NOT NULL,  -- pending, running, completed, failed, cancelled
    progress INTEGER DEFAULT 0,
    result JSONB,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3
);

CREATE INDEX idx_tasks_user ON tasks(user_id, created_at DESC);
CREATE INDEX idx_tasks_status ON tasks(status, created_at);

-- Configuration (key-value store)
CREATE TABLE config (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Metadata cache
CREATE TABLE metadata_cache (
    id SERIAL PRIMARY KEY,
    provider TEXT NOT NULL,
    query TEXT NOT NULL,
    result JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_metadata_cache_lookup ON metadata_cache(provider, query, expires_at);

-- Kobo sync state
CREATE TABLE kobo_synced_books (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL,  -- References Calibre book ID
    last_synced TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, book_id)
);

CREATE TABLE kobo_reading_state (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL,  -- References Calibre book ID
    current_bookmark TEXT,
    finished BOOLEAN DEFAULT FALSE,
    priority TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, book_id)
);

-- Calibre library import tracking
CREATE TABLE calibre_imports (
    id SERIAL PRIMARY KEY,
    library_path TEXT NOT NULL UNIQUE,
    last_import_at TIMESTAMP,
    last_book_count INTEGER,
    imported_book_ids INTEGER[],
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Sync state tracking
CREATE TABLE sync_state (
    id SERIAL PRIMARY KEY,
    sync_id UUID UNIQUE NOT NULL,
    source TEXT NOT NULL,  -- 'calibre_sqlite' or 'postgresql'
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    status TEXT NOT NULL,  -- running, completed, failed, rolled_back
    pg_changes_detected INTEGER DEFAULT 0,
    sqlite_changes_detected INTEGER DEFAULT 0,
    conflicts_resolved INTEGER DEFAULT 0,
    errors TEXT[],
    rollback_data JSONB
);

-- Sync conflicts (for manual resolution)
CREATE TABLE sync_conflicts (
    id SERIAL PRIMARY KEY,
    sync_id UUID NOT NULL REFERENCES sync_state(id),
    book_id INTEGER NOT NULL,
    conflict_type TEXT NOT NULL,  -- title, author, tags, cover, etc.
    pg_value JSONB,
    sqlite_value JSONB,
    resolution TEXT,  -- 'pg_wins', 'sqlite_wins', 'pending', 'manual'
    resolved_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

---

### Books Schema (PostgreSQL)

**Primary Book Storage:**

```sql
-- Books (all data from Calibre, now in PostgreSQL)
CREATE TABLE books (
    id SERIAL PRIMARY KEY,
    uuid UUID UNIQUE NOT NULL DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    sort TEXT,  -- Title for sorting (e.g., "Book, The")
    author_sort TEXT,
    timestamp TIMESTAMP DEFAULT NOW(),
    pubdate TIMESTAMP,
    series_index FLOAT,
    last_modified TIMESTAMP DEFAULT NOW(),
    path TEXT NOT NULL,  -- File path to book directory
    has_cover BOOLEAN DEFAULT FALSE,

    -- Calibre compatibility
    calibre_book_id INTEGER,  -- Original Calibre book ID (for tracking)

    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_books_title ON books USING GIN(to_tsvector('english', title));
CREATE INDEX idx_books_author_sort ON books(author_sort);
CREATE INDEX idx_books_timestamp ON books(timestamp DESC);
CREATE INDEX idx_books_last_modified ON books(last_modified);

-- Authors (many-to-many)
CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    sort TEXT,  -- Author name for sorting (e.g., "Name, First")
    UNIQUE(name, sort)
);

CREATE TABLE books_authors_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, author_id)
);

CREATE INDEX idx_books_authors_link_book ON books_authors_link(book_id);
CREATE INDEX idx_books_authors_link_author ON books_authors_link(author_id);

-- Series (many-to-many)
CREATE TABLE series (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    sort TEXT,
    UNIQUE(name)
);

CREATE TABLE books_series_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    series_index FLOAT,
    PRIMARY KEY (book_id, series_id)
);

CREATE INDEX idx_books_series_link_book ON books_series_link(book_id);
CREATE INDEX idx_books_series_link_series ON books_series_link(series_id);

-- Tags (many-to-many)
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE books_tags_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);

CREATE INDEX idx_books_tags_link_book ON books_tags_link(book_id);
CREATE INDEX idx_books_tags_link_tag ON books_tags_link(tag_id);

-- Languages (many-to-many)
CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    language_code TEXT NOT NULL UNIQUE,  -- e.g., 'eng', 'spa'
    name TEXT NOT NULL
);

CREATE TABLE books_languages_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    language_id INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, language_id)
);

-- Publishers (many-to-many)
CREATE TABLE publishers (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    UNIQUE(name)
);

CREATE TABLE books_publishers_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    publisher_id INTEGER NOT NULL REFERENCES publishers(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, publisher_id)
);

-- Identifiers (ISBN, ASIN, Goodreads, etc.)
CREATE TABLE book_identifiers (
    id SERIAL PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    identifier_type TEXT NOT NULL,  -- 'isbn', 'asin', 'goodreads', 'uuid'
    identifier_val TEXT NOT NULL,
    UNIQUE(book_id, identifier_type)
);

CREATE INDEX idx_book_identifiers_book ON book_identifiers(book_id);
CREATE INDEX idx_book_identifiers_type_val ON book_identifiers(identifier_type, identifier_val);

-- Comments (book descriptions)
CREATE TABLE book_comments (
    book_id INTEGER PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
    text TEXT NOT NULL  -- HTML content
);

-- Data (eBook formats)
CREATE TABLE book_data (
    id SERIAL PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    format TEXT NOT NULL,  -- 'EPUB', 'MOBI', 'PDF', 'AZW3', etc.
    uncompressed_size BIGINT,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    UNIQUE(book_id, format)
);

CREATE INDEX idx_book_data_book ON book_data(book_id);
CREATE INDEX idx_book_data_format ON book_data(format);

-- Ratings (many-to-many)
CREATE TABLE ratings (
    id SERIAL PRIMARY KEY,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 0 AND 10),
    name TEXT NOT NULL UNIQUE  -- e.g., '0 stars', '1 star', etc.
);

CREATE TABLE books_ratings_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    rating_id INTEGER NOT NULL REFERENCES ratings(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, rating_id)
);
```

---

### Calibre SQLite Import/Export/Sync

**Calibre Schema (for import/export):**

The application can import from, export to, and sync with Calibre's `metadata.db`:

- `books` - Main book records
- `authors` - Author information
- `series` - Series information
- `tags` - Tags/categories
- `ratings` - Rating levels
- `languages` - Language codes
- `publishers` - Publisher information
- `identifiers` - ISBN, UUID, ASIN, Goodreads, etc.
- `comments` - Book descriptions
- `data` - eBook formats
- `custom_column_*` - Dynamic custom columns
- `books_*_link` - Association tables

**Import/Export Connection:**

```rust
// src/domain/sync/calibre_import.rs

use rusqlite::Connection;
use std::path::PathBuf;

pub struct CalibreImporter {
    pg_pool: PgPool,
}

impl CalibreImporter {
    pub async fn import_from_sqlite(
        &self,
        sqlite_path: &Path,
    ) -> Result<ImportStats, ImportError> {
        // Open Calibre SQLite database
        let conn = Connection::open(sqlite_path)?;

        // Import books with all relations
        self.import_books(&conn).await?;
        self.import_authors(&conn).await?;
        // ... etc

        Ok(ImportStats {
            books_imported: 100,
        })
    }
}
```

**For complete sync strategy details, see:** [Calibre Sync Strategy](./2025-03-29-calibre-sync-strategy.md)
        let pool = r2d2::Pool::builder()
            .max_size(5)  // Read-only needs fewer connections
            .build(rusqlite::SqliteConnection::open(db_path))?;

        Ok(Self { pool })
    }

    pub async fn get_book(&self, id: i32) -> Result<CalibreBook, CalibreError> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT id, title, sort, author_sort, timestamp, pubdate,
                    series_index, last_modified, path, has_cover, uuid
             FROM books WHERE id = ?"
        )?;

        stmt.query_row(id, |row| {
            Ok(CalibreBook {
                id: row.get(0)?,
                title: row.get(1)?,
                sort: row.get(2)?,
                author_sort: row.get(3)?,
                timestamp: row.get(4)?,
                pubdate: row.get(5)?,
                series_index: row.get(6)?,
                last_modified: row.get(7)?,
                path: row.get(8)?,
                has_cover: row.get(9)?,
                uuid: row.get(10)?,
            })
        })
    }
}
```

---

### Custom Columns Architecture

**Challenge:** Calibre custom columns are dynamic user-defined fields. This system must import and support them.

**Storage Strategy: JSONB + Metadata**

```sql
-- Custom column definitions (imported from Calibre)
CREATE TABLE custom_column_definitions (
    id SERIAL PRIMARY KEY,
    label TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    datatype TEXT NOT NULL,  -- text, enum, comments, datetime, rating, bool, int, float, series
    is_multiple BOOLEAN DEFAULT FALSE,
    display_order INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Custom column values (unified storage)
CREATE TABLE custom_column_values (
    id BIGSERIAL PRIMARY KEY,
    column_id INTEGER NOT NULL REFERENCES custom_column_definitions(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL,  -- References Calibre book ID
    value TEXT NOT NULL,
    sort_value TEXT,  -- For sorting
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(column_id, book_id)
);

CREATE INDEX idx_ccv_column_book ON custom_column_values(column_id, book_id);
CREATE INDEX idx_ccv_sort ON custom_column_values(column_id, sort_value);
```

**Type Mapping (Calibre → Rust):**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomColumnValue {
    Text(String),
    Enum(String),
    Comments(String),
    DateTime(chrono::DateTime<chrono::Utc>),
    Rating(i32),  // 0-10 scale
    Bool(bool),
    Int(i64),
    Float(f64),
    Series { id: i32, name: String, index: f32 },
}

impl CustomColumnValue {
    pub fn from_calibre(datatype: &str, value: &str) -> Result<Self, ConversionError> {
        match datatype {
            "text" | "composite" => Ok(Self::Text(value.to_string())),
            "enum" => Ok(Self::Enum(value.to_string())),
            "comments" => Ok(Self::Comments(value.to_string())),
            "datetime" => Ok(Self::DateTime(parse_datetime(value)?)),
            "rating" => Ok(Self::Rating(value.parse()?)),
            "bool" => Ok(Self::Bool(value.parse()?)),
            "int" => Ok(Self::Int(value.parse()?)),
            "float" => Ok(Self::Float(value.parse()?)),
            "series" => Ok(Self::Series { /* parse */ }),
            _ => Err(ConversionError::UnknownType(datatype.to_string())),
        }
    }
}
```

**Import Process:**

```rust
pub async fn import_custom_columns(
    calibre_db: &CalibreDB,
    pg_pool: &PgPool,
) -> Result<ImportStats, ImportError> {
    // 1. Import column definitions from Calibre
    let columns = calibre_db.get_custom_columns().await?;

    for column in columns {
        sqlx::query(
            "INSERT INTO custom_column_definitions (id, label, name, datatype, is_multiple, display_order)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (label) DO UPDATE SET name = EXCLUDED.name, datatype = EXCLUDED.datatype"
        )
        .bind(column.id)
        .bind(&column.label)
        .bind(&column.name)
        .bind(&column.datatype)
        .bind(column.is_multiple)
        .bind(column.display_order)
        .execute(pg_pool)
        .await?;
    }

    // 2. Import column values in batches
    let mut conn = pg_pool.begin().await?;

    for column in &columns {
        let values = calibre_db.get_column_values(column.id).await?;

        for value in values {
            sqlx::query(
                "INSERT INTO custom_column_values (column_id, book_id, value, sort_value)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (column_id, book_id) DO UPDATE
                    SET value = EXCLUDED.value, sort_value = EXCLUDED.sort_value"
            )
            .bind(column.id)
            .bind(value.book_id)
            .bind(&serialize_value(&value.value, &column.datatype)?)
            .bind(&value.sort_value)
            .execute(&mut *conn)
            .await?;
        }
    }

    conn.commit().await?;

    Ok(ImportStats { columns_imported: columns.len() })
}
```

---

### Key Design Decisions

1. **Two-Database Architecture:**
   - PostgreSQL for application state (users, sessions, tasks)
   - SQLite for Calibre library data (read-only)
   - Maintains Calibre desktop compatibility
   - Eliminates sync complexity

2. **Calibre Compatibility:**
   - Direct read access to Calibre's SQLite database
   - Preserve Calibre's schema structure
   - No modification of Calibre data
   - Support all Calibre features (custom columns, etc.)

3. **Custom Columns Support:**
   - Import column definitions dynamically
   - Unified storage with JSONB-style approach
   - Type-safe value conversion
   - Efficient querying with indexes

4. **Performance Optimizations:**
   - Connection pooling for both databases
   - GIN indexes for full-text search (PostgreSQL)
   - Cached metadata for frequently accessed data
   - Lazy loading for large objects

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

## Authentication & Authorization

### Session Architecture

**Session Storage:** Encrypted Cookies

```
┌──────────────┐
│  Browser     │
│              │  Cookie: {"user_id": 123, "csrf_token": "abc"}
│  └────────────┘
│       ↓↑ (encrypted, signed)
┌──────────────┐
│  Axum App    │
│              │  tower-session (encrypted cookies)
│  └────────────┘
│       ↓↑
┌──────────────┐
│  Database    │
│              │  - Session not stored in DB
│  └────────────┘    - Token validated via crypto
```

**Why Encrypted Cookies?**
- No Redis required (simpler deployment)
- No session database queries (faster)
- Stateless (easier horizontal scaling)
- Size limit: ~4KB (sufficient for user_id + csrf_token)

**Session Data Structure:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i32,
    pub username: String,
    pub roles: RoleFlags,
    pub csrf_token: String,
    pub expires_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}
```

### Multi-Provider Authentication

**Supported Providers:**

1. **Local** - Username/password in PostgreSQL
2. **LDAP** - Active Directory, OpenLDAP
3. **OAuth** - Google, GitHub

**Account Linking Strategy:**

```sql
-- OAuth/LDAP account linking
CREATE TABLE user_auth_providers (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,  -- 'local', 'ldap', 'google', 'github'
    provider_user_id TEXT NOT NULL,  -- LDAP DN or OAuth subject
    provider_email TEXT,
    profile JSONB,
    linked_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_used TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, provider, provider_user_id)
);
```

**Authentication Flow (LDAP):**

```
1. User enters username/password
   ↓
2. AuthService::authenticate_ldap()
   ↓
3. Bind to LDAP server, verify credentials
   ↓
4. Check if account exists in user_auth_providers
   - If yes: Update last_used, create session
   - If no: Create new user account (provisioning)
   ↓
5. Create session with encrypted cookie
   ↓
6. Redirect to home
```

**Authentication Flow (OAuth):**

```
1. User clicks "Login with Google"
   ↓
2. Redirect to Google OAuth 2.0
   ↓
3. User authorizes app
   ↓
4. Google redirects with authorization code
   ↓
5. Exchange code for access token
   ↓
6. Fetch user info from Google API
   ↓
7. Check if account exists in user_auth_providers
   - If yes: Update last_used, create session
   - If no: Create new user account (provisioning)
   ↓
8. Create session with encrypted cookie
   ↓
9. Redirect to home
```

### Role-Based Permission System

**Role Bitmask (matching Calibre-Web):**

```rust
bitflags! {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
    pub struct RoleFlags: u64 {
        const ADMIN       = 1 << 0;  // 1
        const DOWNLOAD    = 1 << 1;  // 2
        const UPLOAD      = 1 << 2;  // 4
        const EDIT        = 1 << 3;  // 8
        const PASSWD      = 1 << 4;  // 16
        const ANONYMOUS   = 1 << 5;  // 32
        const EDIT_SHELVES = 1 << 6; // 64
        const DELETE_BOOKS = 1 << 7; // 128
        const VIEWER      = 1 << 8;  // 256
    }
}

// Default role sets
const ADMIN_USER_ROLES: RoleFlags = RoleFlags::all().remove(RoleFlags::ANONYMOUS);
const STANDARD_USER_ROLES: RoleFlags = RoleFlags::DOWNLOAD.union(RoleFlags::VIEWER);
const GUEST_USER_ROLES: RoleFlags = RoleFlags::ANONYMOUS.union(RoleFlags::VIEWER);
```

**Permission Check Extractor:**

```rust
pub struct RequireRole(RoleFlags);

impl<S> FromRequestParts<S> for RequireRole
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthenticatedUser>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if user.roles.contains(self.0) {
            Ok(RequireRole(self.0))
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

// Usage in handler
async fn delete_book(
    RequireRole(RoleFlags::DELETE_BOOKS): RequireRole,
    Path(id): Path<i32>,
) -> Result<Html<String>, AppError> {
    // Handler logic
}
```

---

## File Storage Strategy

### Directory Structure

**Calibre-Compatible Structure:**

```
/var/lib/calibre-web/
├── library/                    # Calibre library root (configurable)
│   ├── author/                  # Calibre's author/title hierarchy
│   │   └── Author Name/
│   │       └── Book Title/
│   │           ├── Book Title.epub
│   │           ├── cover.jpg
│   │           └── metadata.opf
│   └── metadata.db              # Calibre database (SQLite)
├── covers/                      # Generated thumbnails (cache)
│   ├── small/                    # 48x48
│   ├── medium/                   # 128x128
│   └── large/                    # 256x256
└── uploads/                     # Temporary upload staging
    └── temp_123456/
```

### File Upload Flow

```
1. User uploads book via web interface
   ↓
2. File stored in uploads/temp_XXX/
   ↓
3. Extract metadata (title, author, cover, etc.)
   ↓
4. Create Calibre-compatible directory structure
   ↓
5. Move file to library/author/title/ directory
   ↓
6. Generate cover thumbnails
   ↓
7. Update Calibre database (via Calibre CLI or direct)
   ↓
8. Cleanup temp files
```

**Upload Service:**

```rust
pub async fn upload_book(
    mut uploaded_file: Multipart,
    user_id: i32,
    calibre_db: &CalibreDB,
    config: &LibrarySettings,
) -> Result<i32, UploadError> {
    // 1. Save to temp location
    let temp_dir = config.temp_path.join(format!("temp_{}", user_id));
    fs::create_dir_all(&temp_dir)?;
    let file_path = temp_dir.join(&uploaded_file.filename);
    uploaded_file.copy_to(&file_path).await?;

    // 2. Extract metadata
    let metadata = extract_metadata(&file_path).await?;

    // 3. Create Calibre directory
    let author_dir = config.library_path
        .join("author")
        .join(&sanitize_filename(&metadata.author));
    let book_dir = author_dir.join(&sanitize_filename(&metadata.title));
    fs::create_dir_all(book_dir)?;

    // 4. Move file to final location
    let final_path = book_dir.join(&uploaded_file.filename);
    fs::rename(&file_path, &final_path)?;

    // 5. Add to Calibre database
    let book_id = add_to_calibre_db(calibre_db, &metadata, &final_path).await?;

    // 6. Generate thumbnails
    generate_cover_thumbnails(book_id, &metadata.cover_path).await?;

    Ok(book_id)
}
```

### Cover Image Handling

**Cover Storage:**

```
Covers are stored in Calibre's directory structure:
- library/author/title/cover.jpg (original)

Thumbnails are generated and cached:
- covers/small/48x48/BOOK_ID.jpg
- covers/medium/128x128/BOOK_ID.jpg
- covers/large/256x256/BOOK_ID.jpg
```

**Thumbnail Generation:**

```rust
pub async fn generate_cover_thumbnails(
    book_id: i32,
    cover_path: &Path,
) -> Result<(), CoverError> {
    let img = image::open(cover_path)?;

    // Generate thumbnails
    let sizes = [
        (48, "small"),
        (128, "medium"),
        (256, "large"),
    ];

    for (size, dir_name) in sizes {
        let thumbnail_dir = PathBuf::from("/covers").join(dir_name);
        fs::create_dir_all(&thumbnail_dir)?;

        let thumbnail = img.thumbnail_exact(size, size);
        let thumbnail_path = thumbnail_dir.join(format!("{}.jpg", book_id));
        thumbnail.save(&thumbnail_path)?;
    }

    Ok(())
}
```

---

## Background Task System (Detailed)

### Task Database Schema

```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type TEXT NOT NULL,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    status TEXT NOT NULL,  -- pending, running, completed, failed, cancelled
    priority INTEGER DEFAULT 0,  -- 0=low, 1=normal, 2=high
    progress INTEGER DEFAULT 0,
    result JSONB,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    worker_id TEXT,  -- Which worker is processing
    last_heartbeat TIMESTAMP
);

CREATE INDEX idx_tasks_status_priority ON tasks(status, priority DESC, created_at);
CREATE INDEX idx_tasks_user_pending ON tasks(user_id, status) WHERE status = 'pending';
```

### Task Lifecycle State Machine

```
┌──────────┐
│ PENDING  │  Created, waiting for worker
└─────┬────┘
      │
      ▼
┌──────────┐
│ RUNNING  │  Worker processing, progress updates
└─────┬────┘
      │
      ├───────────────┬──────────────┐
      ▼                   ▼              ▼
┌──────────┐      ┌──────────┐  ┌──────────┐
│ COMPLETED│      │  FAILED   │  │ CANCELLED│
└──────────┘      └──────────┘  └──────────┘
     │                  │              │
     ▼                  ▼              ▼
  Result stored     Retry?        Cleanup
```

**Task Recovery on Restart:**

```rust
// On application startup
pub async fn recover_stuck_tasks(pool: &PgPool) -> Result<(), TaskError> {
    // Find tasks stuck in 'running' status
    let stuck_tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks
         WHERE status = 'running'
         AND (last_heartbeat < NOW() - INTERVAL '5 minutes'
              OR started_at < NOW() - INTERVAL '1 hour')"
    )
    .fetch_all(pool)
    .await?;

    for task in stuck_tasks {
        // Reset to pending with retry count increment
        sqlx::query(
            "UPDATE tasks
             SET status = 'pending',
                 worker_id = NULL,
                 retry_count = retry_count + 1,
                 updated_at = NOW()
             WHERE id = $1"
        )
        .bind(task.id)
        .execute(pool)
        .await?;

        // Max retries exceeded?
        if task.retry_count >= task.max_retries {
            sqlx::query(
                "UPDATE tasks
                 SET status = 'failed',
                     error_message = 'Max retries exceeded',
                     completed_at = NOW()
                 WHERE id = $1"
            )
            .bind(task.id)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
```

### Resource Management

**Concurrency Limits:**

```rust
// Max concurrent tasks per type
pub struct TaskLimits {
    pub max_conversions: usize,     // 3 (CPU intensive)
    pub max_thumbnails: usize,       // 5 (I/O bound)
    pub max_uploads: usize,          // 2 (file I/O)
    pub max_metadata: usize,        // 10 (network I/O)
}

// Worker resource tracking
pub struct WorkerState {
    pub active_conversions: Semaphore,
    pub active_thumbnails: Semaphore,
    pub active_uploads: Semaphore,
}
```

**Timeout Handling:**

```rust
pub async fn execute_with_timeout<F, T>(
    task_id: Uuid,
    pool: &PgPool,
    timeout: Duration,
    f: F,
) -> Result<T, TaskError>
where
    F: Future<Output = Result<T, TaskError>>,
{
    let result = tokio::time::timeout(timeout, f).await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            // Mark task as failed due to timeout
            sqlx::query(
                "UPDATE tasks
                 SET status = 'failed',
                     error_message = 'Task timeout',
                     completed_at = NOW()
                 WHERE id = $1"
            )
            .bind(task_id)
            .execute(pool)
            .await?;

            Err(TaskError::Timeout)
        }
    }
}
```

---

## Security Considerations

### Threat Model

**Assets to Protect:**
1. User credentials (passwords, sessions)
2. eBook files (copyrighted content)
3. User data (reading history, preferences)
4. Administrative access

**Threat Actors:**
1. **Unauthorized users** - Attempting to access without login
2. **Authorized users** - Accessing beyond permissions
3. **Anonymous users** - Exploiting vulnerabilities
4. **Insiders** - Legitimate users with malicious intent

### Security Checklist

**Input Validation:**
- [ ] All user input sanitized (SQL injection prevention)
- [ ] File upload validation (type, size limits, magic bytes)
- [ ] URL parameter validation
- [ ] Form field validation (garde or validator)

**Authentication:**
- [ ] Argon2 password hashing (memory-hard)
- [ ] Secure session generation (random 32 bytes)
- [ ] Session timeout (30 days configurable)
- [ ] Password strength requirements (enforced in UI)
- [ ] Account lockout after failed attempts

**Authorization:**
- [ ] Role-based access control on all operations
- [ ] Content filtering per user (denied_tags, allowed_tags)
- [ ] Shelf visibility checks
- [ ] Admin operation audit logging

**Data Protection:**
- [ ] HTTPS enforced in production
- [ ] CSP headers configured
- [ ] XSS prevention (template auto-escaping)
- [ ] CSRF protection on all state-changing operations
- [ ] SQL injection prevention (SQLx compile-time checks)
- [ ] File system path traversal prevention

**Rate Limiting:**
- [ ] OPDS: 3 req/min per IP
- [ ] Login: 5 attempts/min per IP
- [ ] API: 10 req/min per user
- [ ] Uploads: 10 per hour per user

**Secrets Management:**
- [ ] SECRET_KEY generated with `openssl rand -hex 32`
- [ ] Secrets never in code (environment variables only)
- [ ] Secret rotation policy documented
- [ ] API keys encrypted in database

**Dependency Security:**
- [ ] Regular dependency updates (`cargo audit`)
- [ ] Vulnerability scanning (cargo-audit)
- [ ] Only necessary dependencies included

### Security Headers

```rust
// src/web/middleware/security.rs

use axum::{
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

pub async fn security_headers() -> impl IntoResponse {
    let headers = [
        ("Strict-Transport-Security", "max-age=31536000; includeSubDomains"),
        ("X-Content-Type-Options", "nosniff"),
        ("X-Frame-Options", "DENY"),
        ("X-XSS-Protection", "1; mode=block"),
        ("Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        ("Permissions-Policy", "geolocation=(), microphone=()"),
    ];

    // Apply to all responses via middleware
}
```

---

## Migration Strategy

### Migration Tool Design

**Command-Line Tool:**

```bash
# Import from existing Calibre library
calibre-web-rust import \
    --calibre-db /path/to/metadata.db \
    --library-path /path/to/calibre/library \
    --users-from-csv users.csv \
    --create-admin-admin \
    --dry-run
```

**Migration Process:**

```rust
// src/domain/import/migration.rs

pub struct CalibreMigrator {
    calibre_db: CalibreDB,
    app_db: PgPool,
    config: MigrationConfig,
}

impl CalibreMigrator {
    pub async fn migrate(&self) -> Result<MigrationReport, MigrationError> {
        let report = MigrationReport::default();

        // 1. Validate Calibre database
        self.validate_calibre_db().await?;

        // 2. Migrate books with relations
        report.books = self.migrate_books().await?;

        // 3. Migrate custom columns
        report.custom_columns = self.migrate_custom_columns().await?;

        // 4. Import users (if CSV provided)
        if let Some(users_csv) = &self.config.users_csv {
            report.users = self.import_users(users_csv).await?;
        }

        // 5. Validate migration
        self.validate_migration().await?;

        Ok(report)
    }

    async fn migrate_books(&self) -> Result<i32, MigrationError> {
        let mut imported = 0;

        // Read books from Calibre
        let books = self.calibre_db.get_all_books().await?;

        // Use transaction for atomicity
        let mut tx = self.app_db.begin().await?;

        for book in &books {
            // Check if book exists (by UUID)
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM books WHERE uuid = $1"
            )
            .bind(&book.uuid)
            .fetch_one(&mut *tx)
            .await?;

            if exists == 0 {
                // Book doesn't exist, import it
                // Note: We DON'T copy book data to PostgreSQL
                // We only track metadata for faster searching
                sqlx::query(
                    "INSERT INTO imported_books (book_id, uuid, title, author_sort, path)
                     VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(book.id)
                .bind(&book.uuid)
                .bind(&book.title)
                .bind(&book.author_sort)
                .bind(&book.path)
                .execute(&mut *tx)
                .await?;

                imported += 1;
            }

            // Import relations (authors, tags, series, etc.)
            self.import_book_relations(&mut tx, book).await?;
        }

        tx.commit().await?;

        Ok(imported)
    }
}
```

**Imported Books Tracking Table:**

```sql
CREATE TABLE imported_books (
    book_id INTEGER PRIMARY KEY,  -- Calibre book ID
    uuid TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    author_sort TEXT,
    path TEXT NOT NULL,
    imported_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_synced TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_imported_books_uuid ON imported_books(uuid);
CREATE INDEX idx_imported_books_title ON imported_books USING GIN(to_tsvector('english', title));
```

**Migration Validation:**

```rust
pub async fn validate_migration(&self) -> Result<ValidationReport, MigrationError> {
    let mut report = ValidationReport::default();

    // 1. Check all books are accessible
    let total_books = self.calibre_db.count_books().await?;
    let accessible_books = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM imported_books")
        .fetch_one(&self.app_db)
        .await?;

    report.books_accessible = (accessible_books == total_books);

    // 2. Check file paths are valid
    let books_with_missing_files = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM imported_books
         WHERE NOT EXISTS (SELECT 1 FROM pg_stat_file(path) WHERE path = imported_books.path)"
    )
    .fetch_one(&self.app_db)
    .await?;

    report.all_files_accessible = (books_with_missing_files == 0);

    // 3. Check custom columns imported
    let calibre_columns = self.calibre_db.count_custom_columns().await?;
    let imported_columns = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM custom_column_definitions")
        .fetch_one(&self.app_db)
        .await?;

    report.custom_columns_imported = (imported_columns == calibre_columns);

    Ok(report)
}
```

**Zero-Downtime Migration (Optional):**

```
1. Deploy Rust version alongside Python version (different port)
2. Configure reverse proxy to route to both
3. Run import process (reads from Calibre DB)
4. Validate migration
5. Switch proxy to Rust version
6. Monitor for issues
7. Deprecate Python version
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

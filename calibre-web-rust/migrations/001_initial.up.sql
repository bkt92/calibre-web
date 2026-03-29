-- migrations/001_initial.up.sql

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- User management
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role_bitmask BIGINT NOT NULL DEFAULT 0,
    locale VARCHAR(10) DEFAULT 'en',
    kindle_email VARCHAR(255),
    sidebar_settings BIGINT DEFAULT 0,
    denied_tags TEXT[],
    allowed_tags TEXT[],
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- Sessions (for tracking session metadata only - actual data in encrypted cookies)
CREATE TABLE user_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_sessions_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_expires ON user_sessions(expires_at);

-- Books (imported from Calibre)
CREATE TABLE books (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    sort TEXT,
    author_sort TEXT,
    timestamp TIMESTAMP DEFAULT NOW(),
    pubdate TIMESTAMP,
    series_index FLOAT,
    last_modified TIMESTAMP NOT NULL DEFAULT NOW(),
    path TEXT NOT NULL,
    has_cover BOOLEAN DEFAULT FALSE,
    uuid UUID NOT NULL DEFAULT uuid_generate_v4(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_books_timestamp ON books(timestamp DESC);
CREATE INDEX idx_books_last_modified ON books(last_modified);
CREATE INDEX idx_books_uuid ON books(uuid);

-- Authors
CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    sort TEXT,
    link TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_authors_name ON authors(name);

-- Series
CREATE TABLE series (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    sort TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_series_name ON series(name);

-- Tags
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tags_name ON tags(name);

-- Languages
CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    lang_code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Publishers
CREATE TABLE publishers (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_publishers_name ON publishers(name);

-- Many-to-many relationships
CREATE TABLE books_authors_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, author_id)
);

CREATE TABLE books_series_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    series_index FLOAT,
    PRIMARY KEY (book_id, series_id)
);

CREATE TABLE books_tags_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);

CREATE TABLE books_languages_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    lang_id INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, lang_id)
);

CREATE TABLE books_publishers_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    publisher_id INTEGER NOT NULL REFERENCES publishers(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, publisher_id)
);

-- Book identifiers (ISBN, etc.)
CREATE TABLE book_identifiers (
    id SERIAL PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    val TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_book_identifiers_book ON book_identifiers(book_id);

-- Book comments
CREATE TABLE book_comments (
    id SERIAL PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Book data (formats)
CREATE TABLE book_data (
    id SERIAL PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    format TEXT NOT NULL,
    uncompressed_size BIGINT,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_book_data_book ON book_data(book_id);

-- Ratings
CREATE TABLE ratings (
    id SERIAL PRIMARY KEY,
    rating FLOAT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE books_ratings_link (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    rating_id INTEGER NOT NULL REFERENCES ratings(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, rating_id)
);

-- Sync state tracking
CREATE TABLE sync_state (
    id BIGSERIAL PRIMARY KEY,
    last_sync_at TIMESTAMP,
    last_book_id_synced INTEGER DEFAULT 0,
    sync_status TEXT DEFAULT 'idle',
    conflict_resolution_strategy TEXT DEFAULT 'last_write_wins',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Sync conflicts (for manual resolution)
CREATE TABLE sync_conflicts (
    id BIGSERIAL PRIMARY KEY,
    sync_id BIGINT REFERENCES sync_state(id),
    table_name TEXT NOT NULL,
    record_id INTEGER NOT NULL,
    conflict_type TEXT NOT NULL,
    postgresql_data JSONB,
    sqlite_data JSONB,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_conflicts_resolved ON sync_conflicts(resolved);

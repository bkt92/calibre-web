# Calibre-Web Rust Sync Strategy

**Date:** 2025-03-29
**Author:** Claude Code
**Status:** Draft
**Version:** 1.0

## Overview

This document defines the synchronization strategy between Calibre-Web Rust (PostgreSQL) and Calibre Desktop (SQLite metadata.db). The system uses **PostgreSQL as the single source of truth** with bidirectional synchronization capabilities.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Calibre-Web Rust (PostgreSQL)                │
│                    ┌───────────────────────────┐               │
│                    │  Single Source of Truth   │               │
│                    │  - Books, Authors, Tags    │               │
│                    │  - Users, Sessions, Tasks  │               │
│                    │  - All App State          │               │
│                    └───────────────────────────┘               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                    ┌────────┴────────┐
                    │  Sync Layer    │
                    │  (Bidirectional)│
                    └────────┬────────┘
                             │
┌────────────────────────────┴────────────────────────────────────┐
│                  Calibre Desktop (metadata.db)                  │
│                  ┌───────────────────────────┐                   │
│                  │  Portable Import/Export   │                   │
│                  │  - Initial Import Source  │                   │
│                  │  - Backup Export Target  │                   │
│                  │  - Calibre Compatibility │                   │
│                  └───────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Sync Modes

### Mode 1: Import (Calibre → PostgreSQL)

**Purpose:** Initial migration from Calibre library

**Flow:**
```
1. Open Calibre's metadata.db (read-only)
2. Parse schema and data
3. Transform to PostgreSQL schema
4. Import with transaction safety
5. Verify import integrity
6. Record sync state
```

**Use Cases:**
- First-time setup
- Migrating from Python Calibre-Web
- Periodic refresh from Calibre Desktop

**Command:**
```bash
calibre-web-rust import \
  --calibre-db /path/to/metadata.db \
  --library-path /path/to/calibre/library \
  --mode import \
  --dry-run
```

### Mode 2: Export (PostgreSQL → Calibre)

**Purpose:** Create Calibre-compatible backup

**Flow:**
```
1. Export books from PostgreSQL
2. Transform to Calibre schema
3. Create new metadata.db
4. Copy book files to Calibre directory structure
5. Verify Calibre can open database
```

**Use Cases:**
- Backup to Calibre format
- Transfer library to Calibre Desktop
- Portable library export

**Command:**
```bash
calibre-web-rust export \
  --output /path/to/new-metadata.db \
  --library-path /path/to/library \
  --include-files
```

### Mode 3: Bidirectional Sync (Recommended)

**Purpose:** Keep both systems in sync

**Flow:**
```
1. Detect changes in PostgreSQL (since last sync)
2. Detect changes in Calibre SQLite (since last sync)
3. Compare timestamps (last_modified)
4. Resolve conflicts (configurable strategy)
5. Apply changes to both sides
6. Update sync state
7. Generate sync report
```

**Use Cases:**
- Ongoing synchronization
- Multi-user scenarios (Calibre + Web)
- Continuous integration

**Configuration:**
```toml
[sync]
enabled = true
interval_minutes = 5
auto_sync = true
conflict_resolution = "last_write_wins"  # or "postgresql_wins", "manual"
```

---

## Conflict Resolution Strategies

### Strategy 1: Last-Write-Wins (Default)

**Rule:** Most recent `last_modified` timestamp wins

**Implementation:**
```rust
pub fn resolve_conflict_lww(pg_book: &Book, sqlite_book: &CalibreBook) -> Book {
    if pg_book.last_modified > sqlite_book.last_modified {
        // PostgreSQL wins
        pg_book.clone()
    } else {
        // SQLite wins, convert to Book
        Book::from_calibre(sqlite_book)
    }
}
```

**Pros:**
- ✅ Simple, deterministic
- ✅ No user intervention required
- ✅ Works for automated sync

**Cons:**
- ⚠️ Can overwrite newer data if clocks are skewed
- ⚠️ No user control

### Strategy 2: PostgreSQL Wins

**Rule:** PostgreSQL is always authoritative

**Implementation:**
```rust
pub fn resolve_conflict_pg_wins(pg_book: &Book, _sqlite_book: &CalibreBook) -> Book {
    pg_book.clone()
}
```

**Pros:**
- ✅ Predictable behavior
- ✅ Web app is primary interface
- ✅ No accidental overwrites

**Cons:**
- ⚠️ Calibre Desktop changes are lost
- ⚠️ Confuses users who edit in Calibre

### Strategy 3: Manual Resolution

**Rule:** Flag conflicts for user review

**Implementation:**
```rust
pub struct SyncConflict {
    pub book_id: i32,
    pub pg_version: Book,
    pub sqlite_version: CalibreBook,
    pub conflict_type: ConflictType,
}

pub enum ConflictType {
    TitleMismatch,
    AuthorMismatch,
    TagMismatch,
    CoverMismatch,
}

// Store conflicts in database for UI display
sqlx::query!(
    "INSERT INTO sync_conflicts (book_id, conflict_type, pg_data, sqlite_data)
     VALUES ($1, $2, $3, $4)"
)
// ... user resolves via admin UI
```

**Pros:**
- ✅ Full user control
- ✅ No data loss
- ✅ Transparent process

**Cons:**
- ⚠️ Requires user intervention
- ⚠️ Blocks automated sync
- ⚠️ More complex UI needed

---

## Change Detection

### PostgreSQL Change Detection

```sql
-- Track changes via updated_at timestamp
CREATE INDEX idx_books_updated_at ON books(updated_at);

-- Query for changes since last sync
SELECT id, title, author_sort, updated_at
FROM books
WHERE updated_at > $1  -- last_sync_at
ORDER BY updated_at ASC;
```

### SQLite Change Detection

```rust
// Calibre's last_modified column
let changes = conn.prepare(
    "SELECT id, title, sort, last_modified
     FROM books
     WHERE last_modified > ?"
)?;

// Note: Calibre uses TEXT for timestamps (ISO8601 format)
```

### Change Types

```rust
pub enum ChangeType {
    Added,      // New record in one DB only
    Modified,   // Same ID, different data
    Deleted,    // Missing in one DB
    Unchanged,  // Same data in both
}

pub struct Change {
    pub id: i32,
    pub change_type: ChangeType,
    pub source: DataSource,  // PostgreSQL or SQLite
    pub timestamp: DateTime<Utc>,
    pub data: Option<Book>,
}
```

---

## Sync Algorithm

### Step-by-Step Process

```rust
pub async fn sync_bidirectional(
    pg_pool: &PgPool,
    sqlite_path: &Path,
    config: &SyncConfig,
) -> Result<SyncReport, SyncError> {
    let mut report = SyncReport::default();

    // 1. Load sync state
    let last_sync = load_sync_state(pg_pool).await?;
    let sync_id = Uuid::new_v4();

    // 2. Begin transaction (both databases)
    let mut pg_tx = pg_pool.begin().await?;
    let sqlite_conn = Connection::open(sqlite_path)?;
    sqlite_conn.execute("BEGIN IMMEDIATE")?;

    // 3. Detect changes
    let pg_changes = detect_pg_changes(&mut pg_tx, last_sync).await?;
    let sqlite_changes = detect_sqlite_changes(&sqlite_conn, last_sync)?;

    // 4. Classify changes by book ID
    let mut changes_by_book: HashMap<i32, BookChanges> = HashMap::new();
    for change in pg_changes {
        changes_by_book.entry(change.id)
            .or_insert_with(BookChanges::default)
            .pg_change = Some(change);
    }
    for change in sqlite_changes {
        changes_by_book.entry(change.id)
            .or_insert_with(BookChanges::default)
            .sqlite_change = Some(change);
    }

    // 5. Resolve conflicts and apply changes
    for (book_id, changes) in changes_by_book {
        match resolve_book_changes(&changes, &config.conflict_strategy) {
            Ok(sync_action) => {
                match sync_action {
                    SyncAction::UpdatePg(book) => {
                        update_book_in_pg(&mut pg_tx, &book).await?;
                        report.pg_updated += 1;
                    }
                    SyncAction::UpdateSqlite(book) => {
                        update_book_in_sqlite(&sqlite_conn, &book)?;
                        report.sqlite_updated += 1;
                    }
                    SyncAction::UpdateBoth(book) => {
                        update_book_in_pg(&mut pg_tx, &book).await?;
                        update_book_in_sqlite(&sqlite_conn, &book)?;
                        report.both_updated += 1;
                    }
                    SyncAction::Conflict(conflict) => {
                        report.conflicts.push(conflict);
                    }
                    SyncAction::Skip => {
                        report.skipped += 1;
                    }
                }
            }
            Err(e) => {
                report.errors.push(SyncError::BookError {
                    book_id,
                    error: e.to_string(),
                });
            }
        }
    }

    // 6. Commit transactions
    pg_tx.commit().await?;
    sqlite_conn.execute("COMMIT")?;

    // 7. Update sync state
    save_sync_state(pg_pool, sync_id, &report).await?;

    Ok(report)
}
```

---

## Data Mapping

### PostgreSQL → SQLite

```rust
pub fn book_to_calibre(book: &Book) -> CalibreBookRecord {
    CalibreBookRecord {
        id: book.calibre_book_id.unwrap_or_else(|| book.id),
        title: book.title.clone(),
        sort: book.sort.clone().unwrap_or_else(|| sort_title(&book.title)),
        author_sort: book.author_sort.clone(),
        timestamp: book.timestamp.map(|t| t.to_rfc3339()),
        pubdate: book.pubdate.map(|t| t.to_rfc3339()),
        series_index: book.series_index,
        last_modified: book.last_modified.map(|t| t.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        path: book.path.clone(),
        has_cover: book.has_cover,
        uuid: book.uuid.to_string(),
    }
}
```

### SQLite → PostgreSQL

```rust
pub fn calibre_to_book(calibre_book: &CalibreBook) -> Book {
    Book {
        id: next_id(),  // Generate new ID or map existing
        uuid: Uuid::parse_str(&calibre_book.uuid).unwrap_or_else(|_| Uuid::new_v4()),
        title: calibre_book.title.clone(),
        sort: calibre_book.sort.clone(),
        author_sort: calibre_book.author_sort.clone(),
        timestamp: parse_datetime(&calibre_book.timestamp),
        pubdate: parse_datetime(&calibre_book.pubdate),
        series_index: calibre_book.series_index,
        last_modified: parse_datetime(&calibre_book.last_modified)
            .unwrap_or_else(|| Utc::now()),
        path: calibre_book.path.clone(),
        has_cover: calibre_book.has_cover,
        calibre_book_id: Some(calibre_book.id),
        ..Default::default()
    }
}
```

---

## Custom Columns Sync

**Challenge:** Calibre custom columns are dynamic user-defined fields

**Solution:** Import column definitions + values

```rust
pub async fn sync_custom_columns(
    pg_pool: &PgPool,
    sqlite_conn: &Connection,
) -> Result<CustomColumnSyncStats, SyncError> {
    // 1. Import column definitions
    let columns = sqlite_conn.prepare(
        "SELECT id, label, name, datatype, is_multiple, display_order
         FROM custom_columns"
    )?;

    for column in columns {
        sqlx::query(
            "INSERT INTO custom_column_definitions (calibre_id, label, name, datatype, is_multiple, display_order)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (calibre_id) DO UPDATE SET label = EXCLUDED.label"
        )
        .bind(column.id)
        .bind(&column.label)
        .execute(pg_pool)
        .await?;
    }

    // 2. Import column values
    for column in &columns {
        let values = sqlite_conn.prepare(
            "SELECT book, value FROM custom_column_? WHERE book IS NOT NULL"
        )?;

        for value in values {
            sqlx::query(
                "INSERT INTO custom_column_values (column_id, book_id, value, sort_value)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (column_id, book_id) DO UPDATE
                    SET value = EXCLUDED.value, sort_value = EXCLUDED.sort_value"
            )
            .bind(column.id)
            .bind(value.book)
            .bind(&value.value)
            .execute(pg_pool)
            .await?;
        }
    }

    Ok(CustomColumnSyncStats {
        columns_imported: column_count,
        values_imported: value_count,
    })
}
```

---

## File Synchronization

**Strategy:** Files are NOT synced, only metadata

**Rationale:**
- Book files are large (MBs each)
- File storage location may differ
- No file modification in web UI (Phase 1-2)
- File sync added in Phase 3 (upload/delete)

**Configuration:**
```toml
[sync.filesync]
enabled = false  # Future feature
copy_files = false
verify_checksums = true
```

**Future Implementation (Phase 3):**
```rust
pub async fn sync_book_files(
    config: &SyncConfig,
    report: &mut SyncReport,
) -> Result<(), SyncError> {
    if !config.filesync_enabled {
        return Ok(());
    }

    // Compare file lists
    let pg_files = list_pg_files(&pg_pool).await?;
    let sqlite_files = list_sqlite_files(&sqlite_conn)?;

    // Copy missing files
    for file in pg_files {
        if !sqlite_files.contains(&file) {
            copy_file_to_calibre(&file, &config.calibre_library_path).await?;
            report.files_copied += 1;
        }
    }

    Ok(())
}
```

---

## Error Handling

### Transaction Rollback

```rust
pub async fn sync_with_rollback(
    pg_pool: &PgPool,
    sqlite_path: &Path,
) -> Result<SyncReport, SyncError> {
    let pg_tx = pg_pool.begin().await?;
    let sqlite_conn = Connection::open(sqlite_path)?;

    if let Err(e) = (async {
        // ... sync logic ...
        Ok::<(), SyncError>(())
    }).await {
        // Rollback both transactions
        pg_tx.rollback().await?;
        sqlite_conn.execute("ROLLBACK")?;
        Err(e)
    } else {
        // Commit both
        pg_tx.commit().await?;
        sqlite_conn.execute("COMMIT")?;
        Ok(report)
    }
}
```

### Retry Logic

```rust
pub struct SyncConfig {
    pub max_retries: usize,
    pub retry_delay_seconds: u64,
    pub retry_on_conflict: bool,
}

impl SyncConfig {
    pub fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_seconds: 5,
            retry_on_conflict: true,
        }
    }
}
```

---

## Performance Considerations

### Batch Processing

```rust
// Process books in batches to reduce memory
const BATCH_SIZE: usize = 100;

for batch in books.chunks(BATCH_SIZE) {
    sync_batch(batch, pg_pool, sqlite_conn).await?;
}
```

### Parallel Processing

```rust
// Sync custom columns in parallel
let (columns_task, values_task) = tokio::join!(
    sync_custom_column_definitions(pg_pool, sqlite_conn),
    sync_custom_column_values(pg_pool, sqlite_conn)
);
```

### Incremental Sync

```sql
-- Only sync changes since last sync
CREATE INDEX idx_books_last_sync ON books(last_modified)
WHERE last_modified > COALESCE(
    (SELECT last_sync_at FROM sync_state WHERE source = 'calibre_sqlite'),
    '1970-01-01'::timestamp
);
```

---

## Monitoring and Logging

### Sync Metrics

```rust
#[derive(Debug, Serialize)]
pub struct SyncMetrics {
    pub sync_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_secs: f64,

    pub pg_changes_detected: usize,
    pub sqlite_changes_detected: usize,

    pub pg_updated: usize,
    pub sqlite_updated: usize,
    pub both_updated: usize,
    pub conflicts: usize,
    pub skipped: usize,
    pub errors: Vec<SyncError>,

    pub bytes_transferred: u64,
    pub books_processed: usize,
}
```

### Logging Strategy

```rust
tracing::info!(
    sync_id = %sync_id,
    pg_changes = pg_changes.len(),
    sqlite_changes = sqlite_changes.len(),
    "Starting bidirectional sync"
);

tracing::debug!(
    sync_id = %sync_id,
    book_id = book_id,
    conflict_type = ?conflict,
    "Conflict detected"
);

tracing::error!(
    sync_id = %sync_id,
    book_id = book_id,
    error = %e,
    "Failed to sync book"
);
```

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_resolve_conflict_last_write_wins() {
    let pg_book = Book {
        id: 1,
        title: "New Title".to_string(),
        last_modified: Utc::now(),
        ..Default::default()
    };

    let sqlite_book = CalibreBook {
        id: 1,
        title: "Old Title".to_string(),
        last_modified: Utc::now() - Duration::hours(1),
        ..Default::default()
    };

    let resolved = resolve_conflict_lww(&pg_book, &sqlite_book);
    assert_eq!(resolved.title, "New Title");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_import_from_sqlite() {
    let temp_dir = TempDir::new().unwrap();
    let sqlite_path = create_test_calibre_db(&temp_dir);
    let pg_pool = create_test_pg_pool().await;

    let stats = import_calibre_sqlite(&pg_pool, &sqlite_path).await.unwrap();

    assert_eq!(stats.books_imported, 10);
    assert_eq!(stats.authors_imported, 5);

    // Verify books in PostgreSQL
    let books = sqlx::query_as::<_, Book>("SELECT * FROM books")
        .fetch_all(&pg_pool)
        .await
        .unwrap();

    assert_eq!(books.len(), 10);
}
```

### Sync Tests

```rust
#[tokio::test]
async fn test_bidirectional_sync() {
    // Setup: Both databases have same initial data
    let (pg_pool, sqlite_path) = setup_sync_test().await;

    // Modify PostgreSQL
    sqlx::query("UPDATE books SET title = 'PG Title' WHERE id = 1")
        .execute(&pg_pool)
        .await
        .unwrap();

    // Modify SQLite
    let conn = Connection::open(&sqlite_path).unwrap();
    conn.execute("UPDATE books SET title = 'SQLite Title' WHERE id = 1")?;

    // Run sync
    let config = SyncConfig {
        conflict_resolution: ConflictResolution::PostgreSQLWins,
        ..Default::default()
    };

    let report = sync_bidirectional(&pg_pool, &sqlite_path, &config).await.unwrap();

    // Verify PostgreSQL won
    let pg_book = fetch_book_from_pg(&pg_pool, 1).await;
    assert_eq!(pg_book.title, "PG Title");

    // Verify SQLite was updated
    let sqlite_book = fetch_book_from_sqlite(&conn, 1)?;
    assert_eq!(sqlite_book.title, "PG Title");
}
```

---

## Rollback and Recovery

### Sync State Tracking

```sql
CREATE TABLE sync_state (
    id SERIAL PRIMARY KEY,
    sync_id UUID UNIQUE NOT NULL,
    source TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    status TEXT NOT NULL,  -- running, completed, failed, rolled_back
    pg_changes_detected INTEGER DEFAULT 0,
    sqlite_changes_detected INTEGER DEFAULT 0,
    conflicts_resolved INTEGER DEFAULT 0,
    errors TEXT[],
    rollback_data JSONB
);
```

### Rollback Strategy

```rust
pub async fn rollback_sync(
    pg_pool: &PgPool,
    sync_id: Uuid,
) -> Result<(), SyncError> {
    // 1. Load sync state
    let state = sqlx::query_as::<_, SyncState>(
        "SELECT * FROM sync_state WHERE sync_id = $1"
    )
    .bind(sync_id)
    .fetch_one(pg_pool)
    .await?;

    // 2. Apply rollback data
    if let Some(rollback_data) = state.rollback_data {
        for change in rollback_data.changes {
            restore_previous_state(pg_pool, &change).await?;
        }
    }

    // 3. Mark as rolled back
    sqlx::query(
        "UPDATE sync_state SET status = 'rolled_back', completed_at = NOW()
         WHERE sync_id = $1"
    )
    .bind(sync_id)
    .execute(pg_pool)
    .await?;

    Ok(())
}
```

---

## Configuration Examples

### Development

```toml
[sync]
enabled = true
auto_sync = false  # Manual only
interval_minutes = 0
conflict_resolution = "manual"
dry_run = true
log_level = "debug"
```

### Production

```toml
[sync]
enabled = true
auto_sync = true
interval_minutes = 5
conflict_resolution = "last_write_wins"
dry_run = false
log_level = "info"

[sync.calibre]
sqlite_path = "/var/lib/calibre-web/library/metadata.db"
verify_checksums = false

[sync.performance]
batch_size = 100
parallel_threads = 4
timeout_seconds = 300
```

### Calibre-Primary Mode

```toml
[sync]
enabled = true
auto_sync = true
interval_minutes = 10
conflict_resolution = "sqlite_wins"  # Calibre wins
direction = "import_only"  # One-way from Calibre
```

---

## Security Considerations

### File Access

```rust
// Validate file paths are within library directory
fn validate_library_path(path: &Path, library_root: &Path) -> Result<(), SyncError> {
    let canonical_path = path.canonicalize()?;
    let canonical_root = library_root.canonicalize()?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(SyncError::PathTraversalDetected);
    }

    Ok(())
}
```

### SQL Injection Prevention

```rust
// Use parameterized queries (rusqlite)
stmt.execute("UPDATE books SET title = ? WHERE id = ?", &[title, id])?;

// Never concatenate user input
// BAD: format!("UPDATE books SET title = '{}'", title)
```

### Transaction Isolation

```rust
// Use IMMEDIATE transaction to prevent deadlocks
sqlite_conn.execute("BEGIN IMMEDIATE")?;

// Set appropriate isolation level for PostgreSQL
pg_pool.execute("SET TRANSACTION ISOLATION LEVEL READ COMMITTED").await?;
```

---

## Migration Guide

### From Python Calibre-Web

```bash
# 1. Install Rust version
# 2. Run initial import
calibre-web-rust import \
  --calibre-db /var/lib/calibre-web/app.db \
  --library-path /var/lib/calibre-web/library \
  --users-from-csv users.csv

# 3. Enable sync
# Edit config/local.toml
[sync]
enabled = true
auto_sync = true

# 4. Start service
systemctl start calibre-web-rust
```

### From Fresh Calibre Library

```bash
# 1. Create PostgreSQL database
createdb calibre_web

# 2. Run migrations
calibre-web-rust migrate up

# 3. Import from Calibre
calibre-web-rust import \
  --calibre-db ~/Calibre\ Library/metadata.db \
  --library-path ~/Calibre\ Library

# 4. Start application
calibre-web-rust serve
```

---

## Troubleshooting

### Sync Fails with "Database Locked"

**Problem:** Calibre Desktop is open and has metadata.db locked

**Solution:**
```bash
# Option 1: Close Calibre Desktop
# Option 2: Use WAL mode (allows concurrent reads)
# In SQLite:
PRAGMA journal_mode=WAL;

# Option 3: Retry with backoff
retry_sync_with_backoff(&config).await?;
```

### Timestamp Conflicts

**Problem:** Clock skew causes wrong conflict resolution

**Solution:**
```rust
// Use NTP-synchronized timestamps
// Allow tolerance window (e.g., 5 seconds)
if pg_book.last_modified - sqlite_book.last_modified < Duration::seconds(5) {
    // Treat as concurrent, use PostgreSQL wins
}
```

### Large Library Performance

**Problem:** Sync takes too long for 10,000+ books

**Solution:**
```toml
[sync.performance]
batch_size = 500  # Larger batches
parallel_threads = 8  # More parallelism
incremental = true  # Only sync changes
skip_unmodified = true  # Don't check unchanged books
```

---

## Appendix A: Sync Report Format

```json
{
  "sync_id": "550e8400-e29b-41d4-a716-446655440000",
  "started_at": "2025-03-29T10:00:00Z",
  "completed_at": "2025-03-29T10:02:30Z",
  "duration_secs": 150.5,

  "changes_detected": {
    "postgresql": 15,
    "sqlite": 8
  },

  "actions_taken": {
    "pg_updated": 5,
    "sqlite_updated": 12,
    "both_updated": 3,
    "skipped": 2,
    "conflicts": 1
  },

  "conflicts": [
    {
      "book_id": 123,
      "field": "title",
      "pg_value": "New Title",
      "sqlite_value": "Old Title",
      "resolution": "postgresql_wins"
    }
  ],

  "errors": [],

  "statistics": {
    "books_processed": 1000,
    "authors_processed": 150,
    "tags_processed": 300,
    "bytes_transferred": 52428800
  }
}
```

---

## Appendix B: API Endpoints

### POST /api/sync/trigger

Manually trigger sync

**Request:**
```json
{
  "mode": "bidirectional",
  "conflict_resolution": "last_write_wins"
}
```

**Response:**
```json
{
  "sync_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "started"
}
```

### GET /api/sync/status

Get sync status

**Response:**
```json
{
  "last_sync": "2025-03-29T10:00:00Z",
  "next_sync": "2025-03-29T10:05:00Z",
  "auto_sync_enabled": true,
  "pending_changes": 5
}
```

### GET /api/sync/report/:sync_id

Get detailed sync report

**Response:** (See Appendix A format)

---

**Document Version:** 1.0
**Last Updated:** 2025-03-29

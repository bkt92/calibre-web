# Implementation Plan Supplement - PostgreSQL Single Source of Truth

**Date:** 2025-03-29
**Purpose:** Supplement to Phase 1 & 2 Implementation Plan
**Status:** Draft

## Overview

This document supplements the implementation plan (`2025-03-29-calibre-web-rust-rewrite-phase1-2.md`) with critical architectural changes based on the decision to use **PostgreSQL as the single source of truth** instead of read-only SQLite access.

---

## Architectural Changes

### Original Design (REPLACED)

- PostgreSQL for application state only
- SQLite (Calibre) for books, read-only
- Two-database coordination complexity
- Book CRUD via Calibre CLI or direct SQLite writes

### New Design (CURRENT)

- PostgreSQL for **ALL data** (single source of truth)
- SQLite (Calibre) as portable import/export format only
- Bidirectional sync maintains Calibre compatibility
- Full CRUD control over books in PostgreSQL

---

## Updated File Structure

**Changes from original plan:**

```
REMOVED:
├── src/infrastructure/database/calibre.rs  (read-only SQLite access)

ADDED:
├── src/infrastructure/sync/
│   ├── mod.rs                  # Sync module root
│   ├── calibre_import.rs       # Import from Calibre SQLite
│   ├── calibre_export.rs       # Export to Calibre SQLite
│   └── bidirectional_sync.rs   # Bidirectional sync
├── src/domain/sync/
│   └── mod.rs                  # Sync domain logic
├── src/web/session/
│   └── mod.rs                  # Session management
└── tests/sync_tests.rs         # Sync integration tests
```

---

## New/Updated Tasks

### Task 5: Calibre Import Tool (REPLACES OLD Task 5)

**Original:** Database Layer - Calibre SQLite (Read-Only)
**New:** Calibre Import Tool

**Purpose:** Import books from Calibre's metadata.db into PostgreSQL

**Implementation Steps:**

1. **Update migration schema** (add book tables to PostgreSQL)
   - Add `books` table with all Calibre fields
   - Add association tables: `authors`, `series`, `tags`, `languages`, `publishers`
   - Add `books_*_link` tables for many-to-many relationships
   - Add `book_identifiers`, `book_comments`, `book_data`, `books_ratings_link`

2. **Create Calibre importer** (`src/infrastructure/sync/calibre_import.rs`)
   - Read from Calibre SQLite database
   - Transform data to PostgreSQL schema
   - Handle custom columns dynamically
   - Map Calibre data types (TEXT timestamps → PostgreSQL TIMESTAMP)
   - Import with transaction safety (rollback on error)

3. **Import command-line interface**
   ```bash
   calibre-web-rust import \
     --calibre-db /path/to/metadata.db \
     --library-path /path/to/library \
     --dry-run
   ```

4. **Testing:**
   - Create test Calibre database with sample data
   - Verify import preserves all fields
   - Test import of custom columns
   - Test rollback on error

**Estimated Time:** 1-2 days

---

### Task 11.5: Session Management (UNCHANGED)

Already added in previous plan update. No changes needed.

---

### NEW: Task 11.9: Export Tool (PostgreSQL → Calibre)

**Purpose:** Export books from PostgreSQL to Calibre-compatible SQLite format

**Files:**
- Create: `src/infrastructure/sync/calibre_export.rs`
- Create: `tests/export_tests.rs`

**Implementation Steps:**

1. **Create Calibre schema exporter**
   ```rust
   pub struct CalibreExporter {
       pg_pool: PgPool,
   }

   impl CalibreExporter {
       pub async fn export_to_sqlite(
           &self,
           sqlite_path: &Path,
           include_files: bool,
       ) -> Result<ExportStats, ExportError> {
           // Create new SQLite database
           let conn = Connection::open(sqlite_path)?;

           // Create Calibre-compatible schema
           self.create_calibre_schema(&conn)?;

           // Export books
           let books = self.export_books(&conn).await?;

           // Export relations (authors, tags, series, etc.)
           self.export_relations(&conn).await?;

           // Copy files if requested
           if include_files {
               self.copy_book_files(&conn).await?;
           }

           Ok(ExportStats {
               books_exported: books.len(),
           })
       }
   }
   ```

2. **Export command-line interface**
   ```bash
   calibre-web-rust export \
     --output /path/to/metadata.db \
     --library-path /path/to/calibre/library \
     --include-files
   ```

**Estimated Time:** 1 day

---

### NEW: Task 11.10: Bidirectional Sync

**Purpose:** Keep PostgreSQL and Calibre SQLite synchronized

**Files:**
- Create: `src/infrastructure/sync/bidirectional_sync.rs`
- Create: `src/domain/sync/mod.rs`
- Modify: `config/default.toml` (add sync configuration)
- Create: `tests/sync_tests.rs`

**Implementation Steps:**

1. **Sync configuration**
   ```toml
   [sync]
   enabled = true
   auto_sync = true
   interval_minutes = 5
   conflict_resolution = "last_write_wins"  # or "postgresql_wins", "manual"
   calibre_sqlite_path = "/var/lib/calibre-web/library/metadata.db"
   ```

2. **Change detection**
   - Detect changes in PostgreSQL (via `last_modified` timestamp)
   - Detect changes in Calibre SQLite (via `last_modified` field)
   - Classify changes: Added, Modified, Deleted, Unchanged

3. **Conflict resolution**
   - Implement selected strategy (last-write-wins, PostgreSQL-wins, manual)
   - For manual: Store conflicts in database for admin UI resolution

4. **Sync execution**
   - Run in background task (Tokio spawn)
   - Transaction safety (rollback both DBs on error)
   - Incremental sync (only changed records)
   - Generate sync report

5. **API endpoints**
   - `POST /api/sync/trigger` - Manual sync trigger
   - `GET /api/sync/status` - Current sync status
   - `GET /api/sync/report/:sync_id` - Detailed sync report

**Estimated Time:** 2-3 days

---

### Task 9 Update: Books Repository (CHANGED)

**Original:** Books repository reads from Calibre SQLite (read-only)
**New:** Books repository reads/writes to PostgreSQL

**Changes:**

```rust
// src/domain/books/repository.rs

pub struct BookRepository {
    pg_pool: PgPool,  // Changed from calibre_db: CalibreDB
}

impl BookRepository {
    pub fn new(pg_pool: PgPool) -> Self {  // Updated constructor
        Self { pg_pool }
    }

    pub async fn get_all_books(&self) -> Result<Vec<Book>, sqlx::Error> {
        sqlx::query_as!(
            Book,
            "SELECT id, title, sort, author_sort, timestamp, pubdate,
                    series_index, last_modified, path, has_cover, uuid
             FROM books
             ORDER BY timestamp DESC"
        )
        .fetch_all(&self.pg_pool)
        .await
    }

    pub async fn get_book(&self, id: i32) -> Result<Option<Book>, sqlx::Error> {
        sqlx::query_as!(
            Book,
            "SELECT id, title, sort, author_sort, timestamp, pubdate,
                    series_index, last_modified, path, has_cover, uuid
             FROM books WHERE id = $1",
            id
        )
        .fetch_optional(&self.pg_pool)
        .await
    }

    // NEW: Create book
    pub async fn create_book(&self, book: &CreateBook) -> Result<i32, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO books (title, sort, author_sort, path, has_cover, uuid)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
            book.title,
            book.sort,
            book.author_sort,
            book.path,
            book.has_cover,
            book.uuid
        )
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(row.id)
    }

    // NEW: Update book
    pub async fn update_book(&self, id: i32, book: &UpdateBook) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE books SET title = COALESCE($2, title),
                              sort = COALESCE($3, sort),
                              author_sort = COALESCE($4, author_sort),
                              last_modified = NOW(),
                              updated_at = NOW()
             WHERE id = $1",
            id,
            book.title,
            book.sort,
            book.author_sort
        )
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    // NEW: Delete book
    pub async fn delete_book(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM books WHERE id = $1")
            .bind(id)
            .execute(&self.pg_pool)
            .await?;

        Ok(())
    }
}
```

**Estimated Time:** 1 day (updates to existing task)

---

## Task Dependencies

### Updated Dependency Chain

```
Task 4 (PostgreSQL setup) must include book schema
    ↓
Task 5 (Import tool) - can now import books
    ↓
Task 9 (Books repository) - now uses PostgreSQL
    ↓
Task 11.5 (Sessions)
    ↓
Task 11.9 (Export tool) - NEW
    ↓
Task 11.10 (Bidirectional sync) - NEW
    ↓
Task 12 (Books routes) - works with PostgreSQL books
```

---

## Migration Path

### For Existing Implementation

If you started with the original plan (read-only SQLite):

1. **Stop at Task 4** (PostgreSQL setup)
2. **Update migration** to include book tables (see Task 5, Step 1)
3. **Skip old Task 5** (Calibre SQLite read-only)
4. **Use new Task 5** (Import tool)
5. **Continue with Task 6** (Authentication infrastructure)

### Fresh Start

If starting fresh with this supplement:

1. **Follow original plan** through Task 4
2. **Apply Task 5 from this supplement** (not original)
3. **Apply Task 9 updates** (use PostgreSQL instead of CalibreDB)
4. **Add Tasks 11.9 and 11.10** after Task 11.5
5. **Continue with Task 12+** (unchanged)

---

## Testing Strategy

### Import Tests

```rust
#[tokio::test]
async fn test_import_from_calibre() {
    let temp_dir = TempDir::new().unwrap();
    let sqlite_path = temp_dir.path().join("metadata.db");

    // Create test Calibre database
    create_test_calibre_db(&sqlite_path);

    // Import to PostgreSQL
    let importer = CalibreImporter::new(pg_pool);
    let stats = importer.import_from_sqlite(&sqlite_path).await.unwrap();

    // Verify
    assert_eq!(stats.books_imported, 10);

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
    // Setup: Both databases have same data
    setup_sync_test().await;

    // Modify PostgreSQL
    sqlx::query("UPDATE books SET title = 'PG Title' WHERE id = 1")
        .execute(&pg_pool)
        .await
        .unwrap();

    // Run sync
    let report = sync_bidirectional(&pg_pool, &sqlite_path, &config).await.unwrap();

    // Verify sync
    assert_eq!(report.pg_updated, 0);  // PG has newest
    assert_eq!(report.sqlite_updated, 1);  // SQLite updated
}
```

---

## Configuration Examples

### Development (Manual Sync)

```toml
[sync]
enabled = true
auto_sync = false  # Manual trigger only
conflict_resolution = "manual"
dry_run = true
```

### Production (Auto Sync)

```toml
[sync]
enabled = true
auto_sync = true
interval_minutes = 5
conflict_resolution = "last_write_wins"
calibre_sqlite_path = "/var/lib/calibre-web/library/metadata.db"
```

### PostgreSQL-Wins (Web App Primary)

```toml
[sync]
enabled = true
auto_sync = true
interval_minutes = 10
conflict_resolution = "postgresql_wins"
direction = "export_only"  # PG → Calibre one-way
```

---

## Rollback Plan

If sync fails or causes issues:

1. **Disable sync** in configuration:
   ```toml
   [sync]
   enabled = false
   ```

2. **Rollback last sync:**
   ```bash
   calibre-web-rust sync rollback --sync-id <uuid>
   ```

3. **Export PostgreSQL snapshot:**
   ```bash
   calibre-web-rust export --output backup-$(date +%Y%m%d).db
   ```

4. **Re-import from known good state**

---

## Performance Considerations

### Import Performance

- **Batch processing:** Import 100-500 books at a time
- **Parallel processing:** Import relations (authors, tags) in parallel
- **Transaction size:** Use transactions per 100 books (avoid huge transactions)

### Sync Performance

- **Incremental sync:** Only sync changes since last sync
- **Batch size:** Process changes in batches of 50-100
- **Index usage:** Ensure `last_modified` indexes exist
- **Rate limiting:** Don't sync more frequently than every 1 minute

---

## Documentation Updates

### Update README.md

Add section on Calibre import/sync:

```markdown
## Calibre Integration

### Initial Import

\`\`\`bash
calibre-web-rust import \
  --calibre-db ~/Calibre\ Library/metadata.db \
  --library-path ~/Calibre\ Library
\`\`\`

### Ongoing Sync

The application can automatically sync with Calibre Desktop to keep both systems in sync. Configure in `config/local.toml`:

\`\`\`toml
[sync]
enabled = true
auto_sync = true
interval_minutes = 5
\`\`\`

### Export to Calibre Format

\`\`\`bash
calibre-web-rust export \
  --output backup-metadata.db \
  --include-files
\`\`\`
```

---

## Summary

**Key Changes:**
1. ✅ PostgreSQL is now single source of truth (ALL data)
2. ✅ Calibre SQLite is portable import/export format
3. ✅ Import tool replaces read-only SQLite access
4. ✅ Books repository now uses PostgreSQL (full CRUD)
5. ✅ Added export tool and bidirectional sync
6. ✅ Session management remains unchanged

**New Tasks Added:**
- Task 5: Calibre Import Tool (replaces old Task 5)
- Task 11.9: Export Tool (new)
- Task 11.10: Bidirectional Sync (new)

**Tasks Updated:**
- Task 4: Migration schema includes book tables
- Task 9: Books repository uses PostgreSQL instead of CalibreDB
- Task 12+: No changes (routes work with PostgreSQL)

**Total Additional Time:** 5-6 days for new sync functionality

**Compatibility:** Fully backward compatible with existing Calibre libraries through import/export.

---

**Next Steps:**

1. Review this supplement
2. Update main implementation plan if desired
3. Begin execution with updated Task 5
4. Test import/sync functionality thoroughly
5. Deploy with sync disabled initially, enable after validation

---

**Reference Documents:**
- `docs/superpowers/specs/2025-03-29-calibre-web-rust-rewrite-design.md` (updated)
- `docs/superpowers/specs/2025-03-29-calibre-sync-strategy.md` (new)

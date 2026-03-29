# Database Documentation

Calibre-Web uses two separate databases: the Calibre database (read-only) and the application database (read-write).

## Database Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Calibre-Web                              │
├──────────────────────┬──────────────────────────────────────┤
│   Calibre Database   │      Application Database            │
│   (metadata.db)      │      (app.db)                        │
│                      │                                      │
│   Read-Only          │      Read-Write                      │
│   Managed by         │      Managed by Calibre-Web          │
│   Calibre Desktop    │                                      │
└──────────────────────┴──────────────────────────────────────┘
```

## Calibre Database (metadata.db)

**Location:** User's Calibre library directory

**Access:** Read-only via SQLAlchemy ORM (`cps/db.py`)

**Purpose:** Stores eBook library data managed by Calibre desktop application

### Core Tables

#### Books (`books`)

Main book records.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| title | String | Book title |
| sort | String | Title sort string |
| author_sort | String | Author sort string |
| timestamp | TIMESTAMP | Last modified |
| pubdate | TIMESTAMP | Publication date |
| series_index | Float | Position in series |
| last_modified | TIMESTAMP | Last sync timestamp |
| path | String | Relative path to eBook file |
| uuid | String | Unique identifier (UUID) |
| has_cover | Boolean | Cover image exists |
| last_read | TIMESTAMP | Last read date (user-specific, not in Calibre) |

#### Authors (`authors`)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| name | String | Author name |
| sort | String | Author sort string |
| link | String | Author link URL |

#### Series (`series`)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| name | String | Series name |
| sort | String | Series sort string |
| link | String | Series link URL |

#### Tags (`tags`)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| name | String | Tag/category name |

#### Ratings (`ratings`)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| rating | Integer | Rating value (0-5) |

#### Languages (`languages`)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| lang_code | String | ISO 639 language code |

#### Publishers (`publishers`)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| name | String | Publisher name |
| sort | String | Publisher sort string |

#### Identifiers (`identifiers`)

External identifiers (ISBN, UUID, etc.)

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| book | Integer | Foreign key to books.id |
| type | String | Identifier type (isbn, uuid, etc.) |
| val | String | Identifier value |

#### Comments (`comments`)

Book descriptions/synopses.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| book | Integer | Foreign key to books.id |
| text | String | HTML formatted description |

#### Data (`data`)

eBook format records.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| book | Integer | Foreign key to books.id |
| format | String | File extension (EPUB, MOBI, etc.) |
| uncompressed_size | Integer | Size in bytes |
| name | String | File name |

### Association Tables (Many-to-Many)

#### books_authors_link

Links books to authors.

| Column | Type | Description |
|--------|------|-------------|
| book | Integer | FK to books.id (PK) |
| author | Integer | FK to authors.id (PK) |

#### books_series_link

Links books to series.

| Column | Type | Description |
|--------|------|-------------|
| book | Integer | FK to books.id (PK) |
| series | Integer | FK to series.id (PK) |

#### books_tags_link

Links books to tags.

| Column | Type | Description |
|--------|------|-------------|
| book | Integer | FK to books.id (PK) |
| tag | Integer | FK to tags.id (PK) |

#### books_ratings_link

Links books to ratings.

| Column | Type | Description |
|--------|------|-------------|
| book | Integer | FK to books.id (PK) |
| rating | Integer | FK to ratings.id (PK) |

#### books_languages_link

Links books to languages.

| Column | Type | Description |
|--------|------|-------------|
| book | Integer | FK to books.id (PK) |
| lang_code | Integer | FK to languages.id (PK) |

#### books_publishers_link

Links books to publishers.

| Column | Type | Description |
|--------|------|-------------|
| book | Integer | FK to books.id (PK) |
| publisher | Integer | FK to publishers.id (PK) |

### Custom Columns

Calibre supports custom columns with dynamic types:

| Column Type | Python Type | Description |
|-------------|-------------|-------------|
| text | String | Text values |
| enumeration | String | Predefined values |
| comments | String | Long text (HTML) |
| datetime | TIMESTAMP | Date/time |
| rating | Integer | Rating (0-5 stars) |
| bool | Boolean | Yes/No |
| int | Integer | Integer values |
| float | Float | Decimal values |
| series | Integer | Link to custom series |
| composite | String | Computed from other columns |

Custom columns are dynamically loaded as:
- `cc_classes[<column_id>]` - ORM class
- Table name: `custom_column_<column_id>`
- Link table: `books_custom_column_<column_id>_link`

**Excluded from dynamic loading:** `composite`, `series` (see `cc_exceptions` in `cps/db.py`)

### Library ID

`library_id` table stores unique UUID for the Calibre library.

## Application Database (app.db)

**Location:** Configurable (default: same directory as application)

**Access:** Read-write via SQLAlchemy ORM (`cps/ub.py`)

**Purpose:** Stores Calibre-Web application state

### Tables

#### User

User accounts and authentication.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| nickname | String | Display name |
| email | String | Email address (unique) |
| role | Integer | Bitmask of user roles |
| password | String | Hashed password |
| kindle_mail | String | Kindle email address |
| shelf | JSON | Shelf display settings |
| view_settings | JSON | UI view settings |
| locale | String | Language code |
| default_language | String | Default book language |
| mature_limit | Boolean | Mature content filter |
| sidebar_view | Integer | Sidebar element visibility |
| denied_tags | String | Restricted tags (comma-separated) |
| allowed_tags | String | Allowed tags (comma-separated) |
| denied_column_value | String | Custom column restrictions |
| allowed_column_value | String | Custom column allowances |
| remote_auth_token | String | Remote authentication token |

#### User_Sessions

Active user sessions for "remember me" functionality.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| user_id | Integer | FK to User.id |
| session_key | String | Session identifier |
| random | String | Random token |
| expiry | TIMESTAMP | Session expiration |

#### RemoteAuthToken

Tokens for remote login API.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| user_id | Integer | FK to User.id |
| token | String | Auth token (8 chars) |
| verified | Boolean | Token verified |
| expiration | TIMESTAMP | Token expiration |
| auth_type | Integer | Auth type (e.g., OPDS) |

#### Shelf

Custom book collections (shelves).

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| name | String | Shelf name (unique per user) |
| is_public | Boolean | Publicly visible |
| user_id | Integer | FK to User.id |
| created | TIMESTAMP | Creation date |
| last_modified | TIMESTAMP | Last modification |
| kobo_sync | Boolean | Synced to Kobo shelf |

#### BookShelf

Books on shelves.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| book_id | Integer | FK to Books.id (Calibre) |
| shelf | Integer | FK to Shelf.id |
| order | Integer | Display order |
| date_added | TIMESTAMP | Added date |

#### Downloads

Download history.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| user_id | Integer | FK to User.id |
| book_id | Integer | FK to Books.id (Calibre) |
| download_time | TIMESTAMP | Download timestamp |

#### Domain

LDAP domain configuration.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| name | String | Domain name |
| ldap_provider_url | String | LDAP server URL |
| ldap_dn | String | LDAP distinguished name |
| ldap_serv_username | String | LDAP service account username |
| ldap_serv_password | String | LDAP service account password |
| ldap_user_object | String | LDAP user object filter |
| ldap_cert_path | String | LDAP certificate path |

#### OAuth

OAuth user tokens (Flask-Dance).

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| user_id | Integer | FK to User.id |
| provider | String | OAuth provider (google, github) |
| token | JSON | OAuth token (encrypted) |
| provider_user_id | String | Provider user ID |

#### KoboSyncedBooks

Kobo device sync status.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| user_id | Integer | FK to User.id |
| book_id | Integer | FK to Books.id (Calibre) |
| last_synced | TIMESTAMP | Last sync timestamp |

#### KoboReadingState

Kobo reading progress.

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |
| user_id | Integer | FK to User.id |
| book_id | Integer | FK to Books.id (Calibre) |
| current_bookmark | String | Current position |
| finished | Boolean | Book finished |

## Database Access Patterns

### CalibreDB Access

```python
from cps import calibre_db
from cps.db import Books

# Get book by ID
book = calibre_db.get_book(book_id)

# Search with filters
books = calibre_db.session.query(Books).filter(
    Books.title.ilike(f'%{query}%')
).all()

# Get with relationships
book = calibre_db.session.query(Books).filter(
    Books.id == book_id
).options(
    selectinload(Books.authors),
    selectinload(Books.tags),
    selectinload(Books.series),
    selectinload(Books.data)
).first()

# Filtered search (respects user restrictions)
books = calibre_db.get_filtered_books(
    shelf=shelf_id,
    tag=tag_id,
    author=author_id
)
```

### User DB Access

```python
from cps import ub
from flask_login import current_user

# Get current user
user = current_user

# Query users
users = ub.session.query(ub.User).all()

# Create user
new_user = ub.User(
    nickname="username",
    email="user@example.com",
    role=ub.ROLE_USER
)
new_user.password = generate_password_hash("password")
ub.session.add(new_user)
ub.session.commit()

# Update user
user.nickname = "New Name"
ub.session.commit()
```

## Connection Management

### CalibreDB

- **Engine:** Created with SQLite driver
- **Session:** Thread-safe scoped session
- **Connection:** Reused across requests
- **Pool:** StaticPool (single connection)

```python
# Connection string
engine = create_engine(
    'sqlite:///' + db_path,
    connect_args={'check_same_thread': False},
    poolclass=StaticPool
)
```

### User DB

- **Engine:** Created with SQLite driver
- **Session:** Thread-safe scoped session
- **Connection:** Reused across requests
- **Pool:** Default pool

```python
# Connection string
engine = create_engine('sqlite:///' + ub_DB_path)
```

## Best Practices

1. **Use CalibreDB for read operations only** - Never modify Calibre database
2. **Use User DB for application state** - All user-specific data
3. **Close sessions properly** - Use scoped sessions, close on app teardown
4. **Filter user access** - Use `get_filtered_books()` to respect permissions
5. **Handle connection errors** - Database may be locked by Calibre
6. **Use selectinload for relationships** - Avoid N+1 queries
7. **Commit transactions promptly** - Don't leave transactions open
8. **Rollback on error** - Always rollback in exception handlers

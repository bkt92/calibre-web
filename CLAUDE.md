# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Calibre-Web is a Flask-based web application for browsing, reading, and downloading eBooks stored in a Calibre database. It provides a web interface to an existing Calibre library without needing the full Calibre desktop application.

## Running the Application

```bash
# Run directly
python cps.py

# Run with CLI options
python cps.py -p /path/to/settings.db -i 0.0.0.0 -o /path/to/logfile

# After pip install
cps
```

### CLI Options

- `-p` : Path to settings database (default: `app.db` in CONFIG_DIR)
- `-g` : Path to Google Drive database
- `-c` / `-k` : SSL certificate/key file paths (must be used together)
- `-o` : Path to logfile
- `-i` : IP address to listen on
- `-m` : Use memory backend for rate limiter
- `-s user:pass` : Set user password and exit
- `-l` : Allow loading covers from localhost
- `-d` : Dry run of updater (check file permissions)
- `-r` : Enable public database reconnect route (`/reconnect`)

### Default Access

- URL: `http://localhost:8083`
- Default credentials: `admin` / `admin123`
- OPDS feed: `http://localhost:8083/opds`

## Architecture

### Application Structure

The application uses Flask blueprints to organize routes. Key blueprints:

- **web** (`cps/web.py`) - Main web interface routes
- **basic** (`cps/basic.py`) - Basic/authenticated routes
- **opds** (`cps/opds.py`) - OPDS feed for eBook readers
- **admin** (`cps/admin.py`) - Admin interface
- **editbooks** (`cps/editbooks.py`) - Book editing functionality
- **search** (`cps/search.py`) - Search functionality
- **search_metadata** (`cps/search_metadata.py`) - Metadata search
- **shelf** (`cps/shelf.py`) - Custom book collections (shelves)
- **gdrive** (`cps/gdrive.py`) - Google Drive integration
- **kobo** / **kobo_auth** (`cps/kobo.py`, `cps/kobo_auth.py`) - Kobo device sync
- **oauth** (`cps/oauth_bb.py`) - OAuth authentication (Google/GitHub)

### Database Layer

The application uses SQLAlchemy with two separate databases:

1. **CalibreDB** (`cps/db.py`) - Read-only access to Calibre's `metadata.db`
   - Contains: Books, Authors, Series, Tags, Ratings, Languages, Publishers
   - Uses Calibre's existing schema with additional association tables
   - Dynamically creates models for Calibre custom columns

2. **User DB** (`cps/ub.py`) - Application-specific SQLite database
   - Contains: Users, Sessions, Remote tokens, Shelf relationships, Download stats
   - Default: `app.db` (configurable via `-p` flag)

### Web Server

The application can use either Gevent or Tornado as the WSGI server (`cps/server.py`):

- **Gevent** - Preferred, used when available
- **Tornado** - Fallback option (required on Windows for Python 3.8+)

Server selection happens automatically based on installed dependencies.

### Services (`cps/services/`)

- **background_scheduler.py** - APScheduler-based task scheduling
- **worker.py** - Background task worker thread
- **simpleldap.py** - LDAP authentication service
- **gmail.py** - Gmail integration for sending eBooks
- **goodreads_support.py** - Goodreads integration

### Background Tasks (`cps/tasks/`)

Long-running operations are handled as background tasks:

- **convert.py** - eBook format conversion via Calibre binaries
- **upload.py** - Book upload processing
- **thumbnail.py** - Cover thumbnail generation
- **metadata_backup.py** - Metadata backup
- **mail.py** - Email sending tasks
- **clean.py** - Cache cleanup

### Metadata Providers (`cps/metadata_provider/`)

Extensible metadata download from:
- Amazon, Google Books, ComicVine, Douban, Lubimyczytac, Scholar

### Authentication & Authorization

Uses custom login manager (`cps/MyLoginManager.py`) with Flask-Principal.

User roles are bitflags (defined in `cps/constants.py`):
- `ROLE_ADMIN` - Administrative access
- `ROLE_DOWNLOAD` - Download books
- `ROLE_UPLOAD` - Upload books
- `ROLE_EDIT` - Edit book metadata
- `ROLE_PASSWD` - Change password
- `ROLE_EDIT_SHELFS` - Edit custom shelves
- `ROLE_DELETE_BOOKS` - Delete books
- `ROLE_VIEWER` - View books

Login methods:
- `LOGIN_STANDARD` - Local database
- `LOGIN_LDAP` - LDAP authentication
- `LOGIN_OAUTH` - OAuth (Google/GitHub)

### Custom Components

- **ReverseProxied** (`cps/reverseproxy.py`) - WSGI middleware for reverse proxy support
- **CacheBuster** (`cps/cache_buster.py`) - Static asset cache busting in development
- **render_template** (`cps/render_template.py`) - Custom template rendering wrapper

## Development Notes

### Dependencies

Install with optional features:
```bash
pip install -e .
pip install -e ".[gdrive,gmail,goodreads,ldap,oauth,metadata,comics,kobo]"
```

### JavaScript

Frontend uses jQuery with Bootstrap 3. ESLint configuration in `.eslintrc`.

### Translations

Uses Flask-Babel. Translation files in `cps/translations/`. Language names in `cps/iso_language_names.py` (auto-generated, do not edit).

### Calibre Custom Columns

Custom columns from Calibre are dynamically loaded. The `cc_exceptions` list in `cps/db.py` defines columns excluded from dynamic loading (default: `composite`, `series`).

### Security

- CSRF protection via Flask-WTF
- Rate limiting via Flask-Limiter
- Content Security Policy headers
- Session security with HTTPOnly and SameSite cookies

### Testing

Tests are maintained in a separate repository: https://github.com/OzzieIsaacs/calibre-web-test

### File Locations

- Static files: `cps/static/`
- Templates: `cps/templates/`
- Translations: `cps/translations/`
- Cache: `cps/cache/` (or `CACHE_DIRECTORY` env var)
- Config: `CONFIG_DIR` (defaults to app directory, or `~/.calibre-web` if `.HOMEDIR` exists)

### Important Constants

Default port: `8083` (configurable via `CALIBRE_PORT` env var)
Supported eBook formats for upload: See `EXTENSIONS_UPLOAD` in `cps/constants.py`
Audio formats: See `EXTENSIONS_AUDIO` in `cps/constants.py`

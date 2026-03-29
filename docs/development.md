# Development Guide

This guide covers setting up a development environment and contributing to Calibre-Web.

## Prerequisites

- Python 3.8+ (3.12+ recommended for new features)
- Git
- SQLite support (usually included with Python)
- ImageMagick (for cover extraction)

### Optional Dependencies

- **Calibre desktop** - For on-the-fly conversion
- **Kepubify** - For Kobo device support
- **LDAP server** - For LDAP authentication testing

## Setting Up Development Environment

### 1. Clone Repository

```bash
git clone https://github.com/janeczku/calibre-web.git
cd calibre-web
```

### 2. Create Virtual Environment

```bash
python3 -m venv venv
source venv/bin/activate  # Linux/Mac
# or
venv\Scripts\activate  # Windows
```

### 3. Install Dependencies

```bash
# Basic dependencies
pip install -r requirements.txt

# All optional features
pip install -e ".[gdrive,gmail,goodreads,ldap,oauth,metadata,comics,kobo]"

# Or install selectively
pip install -e ".[gdrive,oauth]"
```

### 4. Create Test Database

For development, you'll need a Calibre database:

```bash
# Download sample database
wget https://github.com/janeczku/calibre-web/raw/master/library/metadata.db
```

Or create one with the Calibre desktop application.

### 5. Run Development Server

```bash
# Basic run
python cps.py

# With custom settings
python cps.py -p dev.db -i 127.0.0.1 -o logs/dev.log

# With SSL
python cps.py -c cert.pem -k key.pem
```

Access at: `http://localhost:8083`

Default credentials: `admin` / `admin123`

## Project Structure

```
calibre-web/
├── cps.py                 # Entry point
├── cps/                   # Main application package
│   ├── __init__.py        # Flask app factory
│   ├── server.py          # Web server (Gevent/Tornado)
│   ├── cli.py             # CLI argument parser
│   ├── constants.py       # Constants and configuration
│   ├── db.py              # CalibreDB (books database)
│   ├── ub.py              # User database
│   ├── web.py             # Main web blueprint
│   ├── admin.py           # Admin blueprint
│   ├── editbooks.py       # Book editing blueprint
│   ├── opds.py            # OPDS feed blueprint
│   ├── basic.py           # Basic routes blueprint
│   ├── search.py          # Search blueprint
│   ├── shelf.py           # Shelves blueprint
│   ├── kobo.py            # Kobo sync blueprint
│   ├── oauth_bb.py        # OAuth blueprint
│   ├── gdrive.py          # Google Drive integration
│   ├── config_sql.py      # Configuration management
│   ├── helper.py          # Helper functions
│   ├── logger.py          # Logging setup
│   ├── services/          # Background services
│   ├── tasks/             # Background tasks
│   ├── metadata_provider/ # Metadata sources
│   ├── static/            # Static assets (CSS, JS, images)
│   ├── templates/         # Jinja2 templates
│   └── translations/      # i18n files (Babel)
├── docs/                  # Documentation
├── tests/                 # Tests (separate repo)
└── requirements.txt       # Dependencies
```

## Development Workflow

### Making Changes

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes:**
   - Edit Python code in `cps/`
   - Edit templates in `cps/templates/`
   - Edit static files in `cps/static/`

3. **Test your changes:**
   - Restart the server
   - Test in browser at `http://localhost:8083`
   - Check logs for errors

4. **Lint JavaScript:**
   ```bash
   eslint cps/static/js/*.js
   ```

### Code Style

**Python:**
- Follow PEP 8
- Use 4 spaces for indentation
- Maximum line length: 140 (soft), 160 (hard)
- Use meaningful variable names

**JavaScript:**
- Configuration in `.eslintrc`
- Use double quotes
- 4-space indentation
- jQuery for DOM manipulation

### Adding New Routes

1. Choose the appropriate blueprint (or create new)
2. Add route in blueprint file:

```python
# cps/my_blueprint.py
from flask import Blueprint, render_template
from .usermanagement import login_required

my_bp = Blueprint('my_bp', __name__)

@my_bp.route('/my-route')
@login_required
def my_route():
    return render_template('my_template.html')
```

3. Register in `cps/main.py`:

```python
from .my_blueprint import my_bp
app.register_blueprint(my_bp)
```

### Adding New Background Tasks

1. Create task function in `cps/tasks/`:

```python
# cps/tasks/my_task.py
from . import logger
log = logger.create()

def my_task(task_id, book_id, user_id):
    """Task description"""
    try:
        # Do work
        log.info(f"Processing task {task_id}")
        # Return result
        return {"result": "success"}
    except Exception as e:
        log.error(f"Task failed: {e}")
        return {"result": "error", "error": str(e)}
```

2. Add to WorkerThread task queue:

```python
from cps.services.worker import WorkerThread
WorkerThread.add(task_id, 'my_task', task_id, book_id, user_id)
```

### Adding Metadata Providers

1. Create provider in `cps/metadata_provider/`:

```python
# cps/metadata_provider/mysource.py
from flask import request
from cps import logger

log = logger.create()

def search(query):
    """Search for metadata"""
    try:
        # Make API request
        # Return metadata dict
        return {
            'title': 'Book Title',
            'author': 'Author Name',
            'cover': 'cover_url',
            'description': 'Description',
            'tags': ['tag1', 'tag2'],
            'series': 'Series Name',
            'series_id': 1,
            'languages': ['eng'],
            'publisher': 'Publisher',
            'pubdate': '2024-01-01',
            'identifiers': {'isbn': '1234567890'}
        }
    except Exception as e:
        log.error(f"Search failed: {e}")
        return None
```

2. Register in admin interface or add to metadata provider list

## Database Access

### Reading from CalibreDB

```python
from cps import calibre_db
from cps.db import Books

# Get book by ID
book = calibre_db.get_book(book_id)

# Search books
books = calibre_db.session.query(Books).filter(
    Books.title.ilike(f'%{query}%')
).all()

# Get with relationships
book = calibre_db.session.query(Books).filter(
    Books.id == book_id
).options(
    selectinload(Books.authors),
    selectinload(Books.tags)
).first()
```

### Reading/Writing to User DB

```python
from cps import ub
from flask_login import current_user

# Get current user
user = current_user

# Update user
user.nickname = "New Name"
ub.session.commit()

# Query users
users = ub.session.query(ub.User).all()

# Create new user
new_user = ub.User(
    nickname="username",
    email="user@example.com",
    role=ub.ROLE_USER
)
new_user.password = generate_password_hash("password")
ub.session.add(new_user)
ub.session.commit()
```

## Internationalization

### Adding New Translations

1. Extract strings:
   ```bash
   pybabel extract -F babel.cfg -o cps/messages.pot .
   ```

2. Create new translation:
   ```bash
   pybabel init -i cps/messages.pot -d cps/translations -l <language_code>
   ```

3. Edit `.po` file in `cps/translations/<language_code>/LC_MESSAGES/`

4. Compile:
   ```bash
   pybabel compile -d cps/translations
   ```

### Using Translations in Code

```python
from flask_babel import gettext as _

# Simple
_("Translate this text")

# With variables
_("Hello %(name)s", name=user.name)

# Lazy (for decorators)
lazy_gettext = gettext
```

### Using Translations in Templates

```jinja2
{{ _("Translate this") }}
{{ _("Hello %(name)s", name=user.name) }}
```

## Testing

Tests are maintained in a separate repository:

https://github.com/OzzieIsaacs/calibre-web-test

### Running Tests

```bash
git clone https://github.com/OzzieIsaacs/calibre-web-test.git
cd calibre-web-test
pytest
```

### Writing Tests

See test repository for examples.

## Debugging

### Enable Debug Mode

```bash
export FLASK_DEBUG=1
python cps.py
```

This enables:
- Detailed error pages
- Cache busting for static files
- Auto-reload (if using Flask dev server)

### Logging

Logs are written to:
- Console (stdout/stderr)
- Logfile (if `-o` flag used)
- `logs/` directory (if configured)

Log levels: DEBUG, INFO, WARNING, ERROR, CRITICAL

### Common Issues

**Database locked:**
- Close Calibre desktop app
- Check for other processes

**Import errors:**
- Ensure virtual environment is activated
- Reinstall dependencies: `pip install -r requirements.txt`

**Permission errors:**
- Check file permissions on database
- Verify cache directory is writable

**SSL errors:**
- Verify cert/key file paths
- Check file permissions on cert/key

## Submitting Changes

1. **Create Pull Request:**
   - Fork repository
   - Create feature branch
   - Make changes
   - Push to fork
   - Create PR from fork

2. **PR Description:**
   - Clearly describe problem and solution
   - Reference related issues
   - Include screenshots for UI changes
   - List testing performed

3. **Code Review:**
   - Address review feedback
   - Update tests if needed
   - Update documentation

4. **Large Changes:**
   - Target `development` branch
   - Allows testing before merge to `master`

## Resources

- **Main Repo:** https://github.com/janeczku/calibre-web
- **Wiki:** https://github.com/janeczku/calibre-web/wiki
- **Tests:** https://github.com/OzzieIsaacs/calibre-web-test
- **Discord:** https://discord.gg/h2VsJ2NEfB
- **Issues:** https://github.com/janeczku/calibre-web/issues

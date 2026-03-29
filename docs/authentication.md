# Authentication and Authorization

This document covers Calibre-Web's authentication and authorization system.

## Overview

Calibre-Web supports multiple authentication methods and fine-grained permission control:

- **Local authentication** (default)
- **LDAP authentication**
- **OAuth** (Google, GitHub)
- **Remote login tokens** (for OPDS/eReaders)
- **Bitmask-based roles** for permissions

## Authentication Methods

### Local Authentication (Default)

Uses username/password stored in the application database.

**Configuration:** Admin UI → Basic Configuration → Authentication

**Default credentials:**
- Username: `admin`
- Password: `admin123`

**Password storage:** Werkzeug password hashing (PBKDF2)

**Implementation:** `cps/MyLoginManager.py`, `cps/ub.py`

### LDAP Authentication

Authenticates against an LDAP/Active Directory server.

**Configuration:** Admin UI → Basic Configuration → LDAP Configuration

**Required fields:**
- LDAP provider URL (e.g., `ldap://server:389`)
- LDAP distinguished name (DN)
- LDAP user object filter
- Service account username/password (for binding)

**Implementation:** `cps/services/simpleldap.py`

**Authentication modes:**
- `LDAP_AUTH_ANONYMOUS` (0) - Anonymous bind
- `LDAP_AUTH_UNAUTHENTICATE` (1) - Unauthenticated bind

**Supported:** StartTLS, LDAPS

### OAuth Authentication

Authenticates via OAuth 2.0 providers.

**Supported providers:**
- Google
- GitHub

**Configuration:** Admin UI → Basic Configuration → OAuth Configuration

**Required:**
- OAuth client ID
- OAuth client secret
- Callback URL

**Implementation:** `cps/oauth_bb.py`

**User mapping:**
- OAuth email → Calibre-Web email
- OAuth name → Calibre-Web nickname
- First login → Auto-creates user

### Remote Login

Token-based authentication for OPDS feeds and external applications.

**Implementation:** `cps/remotelogin.py`

**Flow:**
1. User generates token in Admin UI
2. Token sent to user via email
3. User authenticates with token: `http://host/opds/authenticate/<token>`
4. Browser stores session cookie

**Token expiration:** Configurable (default: 10 minutes)

## Session Management

### Session Storage

Sessions stored in `User_Sessions` table (`cps/ub.py`).

**Fields:**
- `user_id` - User ID
- `session_key` - Session identifier
- `random` - Random token for security
- `expiry` - Session expiration (31 days)

### Session Security

**Cookie settings:**
- `SESSION_COOKIE_HTTPONLY` - True (prevents XSS)
- `SESSION_COOKIE_SAMESITE` - 'Lax' (CSRF protection)
- `REMEMBER_COOKIE_SAMESITE` - 'Strict' (CSRF protection)
- Cookie prefix - Configurable via `COOKIE_PREFIX` env var

**Session protection:**
- Strong (default) - IP and User-Agent validation
- Basic - Session token only

### Anonymous Users

Unauthenticated users have limited access:

**Permissions:**
- Browse public shelves
- Search books (if allowed)
- Download (if configured)

**Implementation:** `ub.Anonymous` class in `cps/ub.py`

## Authorization (Roles and Permissions)

### Role System

Permissions are assigned via bitmask roles in `cps/constants.py`:

| Role Constant | Bit Value | Description |
|---------------|-----------|-------------|
| `ROLE_USER` | 0 | Basic authenticated user |
| `ROLE_ADMIN` | 1 << 0 (1) | Full administrative access |
| `ROLE_DOWNLOAD` | 1 << 1 (2) | Download eBooks |
| `ROLE_UPLOAD` | 1 << 2 (4) | Upload eBooks |
| `ROLE_EDIT` | 1 << 3 (8) | Edit book metadata |
| `ROLE_PASSWD` | 1 << 4 (16) | Change own password |
| `ROLE_ANONYMOUS` | 1 << 5 (32) | Anonymous access |
| `ROLE_EDIT_SHELFS` | 1 << 6 (64) | Create/edit shelves |
| `ROLE_DELETE_BOOKS` | 1 << 7 (128) | Delete books |
| `ROLE_VIEWER` | 1 << 8 (256) | View books in browser |

### Default Role Sets

**Admin User:**
```python
ADMIN_USER_ROLES = ADMIN | DOWNLOAD | UPLOAD | EDIT | PASSWD |
                   EDIT_SHELFS | DELETE_BOOKS | VIEWER
```

**New User (default):**
```python
ROLE_USER | ROLE_DOWNLOAD | ROLE_VIEWER | ROLE_PASSWD | ROLE_EDIT_SHELFS
```

**Guest User:**
```python
ROLE_ANONYMOUS | ROLE_DOWNLOAD | ROLE_VIEWER
```

### Checking Permissions

**In templates:**
```jinja2
{% if current_user.role_edit() %}
    <!-- Show edit button -->
{% endif %}
```

**In Python:**
```python
from cps import ub
from cps.constants import ROLE_EDIT, ROLE_DELETE_BOOKS

# Check single permission
if current_user.role_edit():
    # Allow edit

# Check multiple permissions
if current_user.role_edit() and current_user.role_upload():
    # Allow edit and upload

# Check raw bitmask
if current_user.role & ROLE_EDIT:
    # Has edit permission
```

**Decorator:**
```python
from cps.usermanagement import login_required
from flask_login import current_user

@app.route('/admin')
@login_required
def admin_panel():
    if not current_user.role_admin():
        abort(403)
    # ...
```

### Content Restrictions

Users can be restricted from seeing certain content:

**Tags:**
- `denied_tags` - Comma-separated list of blocked tags
- `allowed_tags` - Comma-separated list of allowed tags (whitelist mode)

**Custom Columns:**
- `denied_column_value` - JSON object of custom column restrictions
- `allowed_column_value` - JSON object of custom column allowances

**Implementation:** `get_filtered_books()` in `cps/db.py`

### Shelf Visibility

Users can only see:
- Their own shelves
- Public shelves (`is_public = True`)

## Login Flow

### Standard Login

```
1. User enters credentials
   ↓
2. MyLoginManager.authenticate()
   ↓
3. Check LDAP/OAuth if enabled
   ↓
4. Validate password (local auth)
   ↓
5. login_user(user, remember=bool)
   ↓
6. Store session (User_Sessions table)
   ↓
7. Redirect to home
```

### OAuth Login

```
1. User clicks "Login with Google/GitHub"
   ↓
2. Redirect to OAuth provider
   ↓
3. User authorizes app
   ↓
4. Provider redirects with code
   ↓
5. Exchange code for access token
   ↓
6. Get user info from provider
   ↓
7. Find/create user in database
   ↓
8. login_user(user, remember=bool)
   ↓
9. Redirect to home
```

### Remote Login (OPDS)

```
1. User requests protected OPDS feed
   ↓
2. Redirect to login page with token
   ↓
3. User enters email
   ↓
4. Send email with login link
   ↓
5. User clicks link (validates token)
   ↓
6. login_user(user, remember=True)
   ↓
7. Redirect to OPDS feed
```

## Security Features

### CSRF Protection

**Implementation:** Flask-WTF CSRFProtect

**Protected:** All POST/PUT/DELETE requests

**Exemptions:** OPDS endpoints (rate-limited instead)

```python
from flask_wtf.csrf import csrf_exempt

@opds.route('/feed')
@csrf_exempt
def opds_feed():
    # ...
```

### Rate Limiting

**Implementation:** Flask-Limiter

**Default rules:**
- OPDS: 3 requests/minute per IP
- Login: 5 attempts/minute per IP
- Admin: 10 requests/minute per user

**Configuration:**
- Backend: Redis or in-memory
- Configurable via Admin UI

### Password Requirements

**Minimum length:** None (enforced by UI)

**Validation:** None (enforced by UI)

**Storage:** PBKDF2 with SHA-256

### Session Expiration

**Remember me:** 31 days

**Standard session:** Browser session (closed on browser exit)

**Remote login tokens:** 10 minutes (one-time use)

## Configuration

### Via Admin UI

1. Login as admin
2. Admin → Basic Configuration
3. Scroll to "Authentication"
4. Configure:
   - Authentication type (LDAP/OAuth)
   - Public registration
   - Remote login timeout

### Via Environment Variables

```bash
# Session cookie prefix
export COOKIE_PREFIX="cw_"

# Rate limiter backend (memory or Redis)
export RATELIMIT_STORAGE_URI="redis://localhost:6379"
```

### Via Database

Direct manipulation of `User` table:

```python
from cps import ub
from werkzeug.security import generate_password_hash

user = ub.User.query.filter_by(email='user@example.com').first()
user.password = generate_password_hash('new_password')
ub.session.commit()
```

## Best Practices

1. **Change default password** - Change admin password immediately
2. **Use HTTPS** - Required for LDAP/OAuth in production
3. **Enable rate limiting** - Prevent brute force attacks
4. **Limit admin access** - Only give ROLE_ADMIN to trusted users
5. **Use strong passwords** - Enforce via external policy if needed
6. **Configure LDAP properly** - Use StartTLS or LDAPS
7. **Set secure cookie flags** - HttpOnly and SameSite enabled by default
8. **Regular password rotation** - No built-in enforcement, implement via policy
9. **Audit user access** - Review user list and roles regularly
10. **Test authentication** - Verify all auth methods before production

## Troubleshooting

### Login fails with correct password

- Check user is not disabled
- Verify LDAP/OAuth configuration
- Check logs for errors (`cps/logs/`)

### CSRF token errors

- Clear browser cookies
- Check `COOKIE_PREFIX` setting
- Verify CSRF protection enabled

### Sessions not persisting

- Check `REMEMBER_COOKIE_SAMESITE` setting
- Verify User_Sessions table exists
- Check session expiration settings

### OAuth callback fails

- Verify callback URL matches OAuth app configuration
- Check OAuth client ID/secret
- Ensure HTTPS is used (OAuth requirement)

### LDAP authentication fails

- Test LDAP connection with ldapsearch
- Verify DN and filter syntax
- Check service account credentials
- Enable LDAP logging for debugging

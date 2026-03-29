# OPDS API Documentation

This document covers Calibre-Web's OPDS (Open Publication Distribution System) feed API for eBook reader applications.

## Overview

OPDS is a syndication format for eBooks, similar to RSS but optimized for book catalogs. Calibre-Web implements OPDS 1.2, allowing eBook readers to browse, search, and download books.

```
┌─────────────────────────────────────────────────────────────┐
│                   eBook Reader App                          │
│  - Aldiko                                                   │
│  - Marvin                                                  │
│  - KyBook 3                                                │
│  - Moon+ Reader                                            │
│  - etc.                                                    │
└────────────────────────────┬────────────────────────────────┘
                             │ OPDS Request
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Calibre-Web OPDS                         │
│  - Authentication (if required)                             │
│  - Rate limiting (3 req/min)                               │
│  - XML/Atom feed generation                                │
└────────────────────────────┬────────────────────────────────┘
                             │ XML/Atom Response
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Reader Displays                          │
│  - Catalog of books                                        │
│  - Book covers                                             │
│  - Download options                                        │
└─────────────────────────────────────────────────────────────┘
```

## Base URL

```
http://your-server:8083/opds
```

## Authentication

OPDS feeds support authentication via:

1. **Basic Auth** - Username/password in HTTP headers
2. **Session cookies** - From web login
3. **Remote login tokens** - Token-based authentication

### Basic Auth

```
GET /opds
Authorization: Basic base64(username:password)
```

### Remote Login

```
1. GET /opds/authenticate/<token>
2. Redirect to login page
3. User enters email
4. Email sent with login link
5. User clicks link → Session created
6. Redirect to OPDS feed
```

## Rate Limiting

OPDS endpoints are rate-limited to prevent abuse:

- **Limit:** 3 requests per minute per IP
- **Configurable:** Via Flask-Limiter
- **Bypass:** Admin users exempt

## Endpoints

### Root Feed

```
GET /opds
```

Returns the root OPDS catalog with navigation links.

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"
      xmlns:dc="http://purl.org/dc/elements/1.1/"
      xmlns:opds="http://opds-spec.org/2010/catalog">
  <id>urn:uuid:...</id>
  <title>Calibre-Web Library</title>
  <updated>2024-01-01T00:00:00Z</updated>
  <author>
    <name>Calibre-Web</name>
    <uri>http://your-server:8083</uri>
  </author>

  <!-- Navigation links -->
  <link rel="self" href="/opds" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
  <link rel="start" href="/opds" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>

  <!-- Feeds -->
  <entry>
    <title>By Authors</title>
    <link href="/opds/author" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  </entry>
  <entry>
    <title>By Series</title>
    <link href="/opds/series" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  </entry>
  <entry>
    <title>Recent Additions</title>
    <link href="/opds/new" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  </entry>
</feed>
```

### Search Feed

```
GET /opds/search?q={query}
```

Search books by title, author, or tags.

**Parameters:**
- `q` - Search query

**Response:**
```xml
<feed>
  <title>Search: fantasy</title>
  <entry>
    <title>The Hobbit</title>
    <author>J.R.R. Tolkien</author>
    <link href="/opds/download/1/epub" type="application/epub+zip" rel="http://opds-spec.org/acquisition"/>
    <link href="/opds/cover/1" type="image/jpeg" rel="http://opds-spec.org/image"/>
    <summary>A fantasy novel...</summary>
  </entry>
</feed>
```

### Authors Feed

```
GET /opds/author
```

List all authors.

**Response:**
```xml
<feed>
  <title>Authors</title>
  <entry>
    <title>Isaac Asimov</title>
    <link href="/opds/author/1" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  </entry>
</feed>
```

### Author Detail Feed

```
GET /opds/author/{author_id}
```

List all books by an author.

**Response:**
```xml
<feed>
  <title>Isaac Asimov</title>
  <entry>
    <title>Foundation</title>
    <link href="/opds/download/1/epub" type="application/epub+zip"/>
  </entry>
</feed>
```

### Series Feed

```
GET /opds/series
```

List all series.

**Response:**
```xml
<feed>
  <title>Series</title>
  <entry>
    <title>Foundation Series</title>
    <link href="/opds/series/1" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  </entry>
</feed>
```

### Series Detail Feed

```
GET /opds/series/{series_id}
```

List all books in a series.

**Response:**
```xml
<feed>
  <title>Foundation Series</title>
  <entry>
    <title>Foundation</title>
    <title>1</title>  <!-- Series index -->
    <link href="/opds/download/1/epub" type="application/epub+zip"/>
  </entry>
</feed>
```

### Recent Books

```
GET /opds/new
```

List recently added books (sorted by date added).

**Response:**
```xml
<feed>
  <title>Recent Additions</title>
  <updated>2024-01-01T00:00:00Z</updated>
  <entry>
    <title>New Book</title>
    <published>2024-01-01T00:00:00Z</published>
    <link href="/opds/download/1/epub" type="application/epub+zip"/>
  </entry>
</feed>
```

### Hot Books

```
GET /opds/hot
```

List most popular books (by download count).

### Random Books

```
GET /opds/random
```

List random books.

### Book Detail Feed

```
GET /opds/book/{book_id}
```

Detailed information about a specific book.

**Response:**
```xml
<entry>
  <title>Book Title</title>
  <author>Author Name</author>
  <id>urn:uuid:...</id>
  <updated>2024-01-01T00:00:00Z</updated>
  <published>2024-01-01T00:00:00Z</published>
  <summary>Book description...</summary>

  <!-- Download links -->
  <link href="/opds/download/1/epub" type="application/epub+zip" rel="http://opds-spec.org/acquisition"/>
  <link href="/opds/download/1/mobi" type="application/x-mobipocket-ebook" rel="http://opds-spec.org/acquisition"/>
  <link href="/opds/download/1/pdf" type="application/pdf" rel="http://opds-spec.org/acquisition"/>

  <!-- Cover -->
  <link href="/opds/cover/1" type="image/jpeg" rel="http://opds-spec.org/image"/>
  <link href="/opds/cover/1/120" type="image/jpeg" rel="http://opds-spec.org/image/thumbnail"/>

  <!-- Thumbnails -->
  <link href="/opds/thumbnail/1" type="image/jpeg" rel="http://opds-spec.org/image/thumbnail"/>
</entry>
```

### Download Book

```
GET /opds/download/{book_id}/{format}
```

Download a book in a specific format.

**Formats:**
- `epub` - EPUB file
- `mobi` - MOBI file
- `azw3` - AZW3 file
- `pdf` - PDF file
- `cbz` - Comic book archive
- `cbr` - Comic book RAR
- etc.

**Response:**
- Content-Type: Based on format
- Content-Disposition: attachment

### Cover Image

```
GET /opds/cover/{book_id}
GET /opds/cover/{book_id}/{width}
```

Get book cover image.

**Parameters:**
- `width` - Optional width (120, 240, 360, 480, 720)

**Response:**
- Content-Type: image/jpeg
- Image data

### Thumbnail

```
GET /opds/thumbnail/{book_id}
```

Get small thumbnail of cover.

**Response:**
- Content-Type: image/jpeg
- 48x48 image

## MIME Types

Calibre-Web uses standard MIME types for eBook formats:

| Format | MIME Type |
|--------|-----------|
| EPUB | application/epub+zip |
| MOBI | application/x-mobipocket-ebook |
| AZW3 | application/x-mobipocket-ebook |
| PDF | application/pdf |
| CBZ | application/x-cbz |
| CBR | application/x-cbr |
| DJVU | image/vnd.djvu |

## Pagination

Large feeds support pagination via OpenSearch.

**Response:**
```xml
<feed>
  <link rel="next" href="/opds/author/1?page=2" type="application/atom+xml"/>
  <link rel="last" href="/opds/author/1?page=5" type="application/atom+xml"/>
  <itemsPerPage>20</itemsPerPage>
  <totalResults>100</totalResults>
</feed>
```

## Authentication Flows

### Basic Auth Flow

```
1. Reader requests /opds
2. Server returns 401 Unauthorized
3. Reader sends Basic Auth header
4. Server validates credentials
5. Server returns OPDS feed
```

### Remote Login Flow

```
1. Reader requests /opds
2. Server redirects to /opds/authenticate
3. User enters email
4. Server sends email with login link
5. User clicks link (validates token)
6. Server creates session
7. Server redirects to /opds with session cookie
```

## Configuration

### Enable OPDS

OPDS is enabled by default. Configure via Admin UI:

1. Admin → Basic Configuration
2. Scroll to "OPDS Feed"
3. Configure:
   - Enable/disable OPDS
   - Require authentication
   - Enable public downloads
   - Rate limiting

### Rate Limiting

Configure rate limiting:

```python
# cps/opds.py

# Default: 3 requests per minute
@limiter.limit("3/minute", key_func=get_remote_address)
```

### Access Control

Restrict OPDS access:

- **Require login** - All requests need authentication
- **Public read** - Anyone can browse, download requires login
- **Full public** - No authentication required

## Supported Readers

Calibre-Web OPDS has been tested with:

### Android

- **Aldiko** - Full support
- **Moon+ Reader** - Full support
- **FBReader** - Full support
- **PocketBook** - Full support
- **Cool Reader** - Basic support

### iOS

- **Marvin** - Full support
- **KyBook 3** - Full support
- **BookReader** - Basic support
- **Gerty** - Basic support

### Windows

- **Calibre** - Full support (native)
- **Thorium Reader** - Full support
- **BookVisor** - Basic support

### macOS

- **Calibre** - Full support (native)
- **Thorium Reader** - Full support
- **BookReader** - Basic support

### E-ink Devices

- **Kindle** - Basic support (via conversion)
- **Kobo** - Full support (native Kobo sync)
- **PocketBook** - Full support

## Best Practices

### For Users

1. **Use authenticated OPDS** - Secure your library
2. **Check rate limits** - Don't overload server
3. **Download preferred format** - Choose reader-compatible format
4. **Cache covers** - Reduce bandwidth
5. **Use pagination** - Don't fetch entire library at once

### For Developers

1. **Implement pagination** - Handle large libraries
2. **Cache responses** - Reduce server load
3. **Handle errors** - Graceful degradation
4. **Validate input** - Sanitize all parameters
5. **Log access** - Track usage patterns

## Troubleshooting

### Reader Can't Connect

- Check server URL is correct
- Verify authentication credentials
- Check network connectivity
- Verify OPDS is enabled

### Download Fails

- Check book format is available
- Verify user has download permission
- Check file permissions
- Verify storage space

### Cover Images Not Loading

- Check cover exists in database
- Verify image URL is correct
- Check for hotlink protection
- Verify network access

### Rate Limiting Issues

- Increase rate limit if needed
- Implement exponential backoff
- Cache responses longer
- Use pagination

### Authentication Failures

- Verify username/password
- Check session is valid
- Verify Basic Auth encoding
- Check remote login token

## OPDS Specification

Calibre-Web implements OPDS 1.2:

- **Spec:** https://opds.io/
- **Feed format:** Atom Syndication Format (RFC 4287)
- **Catalog format:** OPDS Catalog

## Extensions

Calibre-Web supports OPDS extensions:

- **OpenSearch** - Search and pagination
- **Acquisition feeds** - Download links
- **Image thumbnails** - Cover previews
- **Partial content** - Book previews

## Future Enhancements

Potential improvements:

- OPDS 2.0 support
- Better pagination
- Search suggestions
- Book previews
- Reading progress sync
- Annotations support

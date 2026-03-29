# Metadata Providers

This document covers Calibre-Web's metadata provider system for downloading book information from external sources.

## Overview

Calibre-Web can download book metadata (title, author, cover, description, tags, etc.) from various online sources. This is useful when:

- Uploading new books without metadata
- Updating existing book metadata
- Finding missing information

```
┌─────────────────────────────────────────────────────────────┐
│                     User Action                             │
│  User clicks "Download Metadata" for a book                 │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              Metadata Search Form                           │
│  - Title/Author/ISBN query                                  │
│  - Select provider(s)                                        │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                 Provider Selection                          │
│  Amazon, Google Books, ComicVine, Douban, Scholar, etc.    │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              Metadata Providers                            │
│  - Query external APIs                                      │
│  - Parse responses                                          │
│  - Return standardized metadata                             │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                 Merge and Display                          │
│  - Show results from all providers                          │
│  - User selects metadata source                             │
│  - Merge with existing book data                            │
└─────────────────────────────────────────────────────────────┘
```

## Architecture

### Provider Interface

All metadata providers implement a common interface:

```python
def search(query):
    """Search for metadata

    Args:
        query: Search query (title, author, ISBN, etc.)

    Returns:
        dict: Metadata dict or None if not found:
        {
            'title': 'Book Title',
            'author': 'Author Name',
            'cover': 'cover_url',
            'description': 'Book description',
            'tags': ['tag1', 'tag2'],
            'series': 'Series Name',
            'series_id': 1,
            'languages': ['eng'],
            'publisher': 'Publisher',
            'pubdate': '2024-01-01',
            'identifiers': {'isbn': '1234567890'}
        }
    """
    pass
```

### Provider Registry

Providers are registered in `cps/services/Metadata.py`:

```python
from cps.metadata_provider import amazon, google, comicvine

PROVIDERS = {
    'amazon': amazon,
    'google': google,
    'comicvine': comicvine,
    # ... more providers
}
```

## Built-in Providers

### Amazon

**File:** `cps/metadata_provider/amazon.py`

**Search types:**
- Title + Author
- ISBN
- ASIN (Amazon Standard Identification Number)

**Requirements:**
- None (uses public Amazon search)

**Data returned:**
- Title, Author
- Cover image
- Description
- Rating
- Publication date
- Publisher
- ISBN/ASIN

**Limitations:**
- May be blocked by Amazon
- Requires cookies for some regions

### Google Books

**File:** `cps/metadata_provider/google.py`

**Search types:**
- Title + Author
- ISBN
- General query

**Requirements:**
- None (uses Google Books API)

**Data returned:**
- Title, Author
- Cover image
- Description
- Categories
- Published date
- Publisher
- ISBN

**Advantages:**
- No API key required
- Good coverage
- Fast response

### ComicVine

**File:** `cps/metadata_provider/comicvine.py`

**Search types:**
- Comic title
- Volume
- Issue number

**Requirements:**
- API key (https://comicvine.gamespot.com/api/)

**Data returned:**
- Title
- Cover image
- Description
- Publisher
- Issue number
- Volume

**Use case:**
- Comic books (CBZ, CBR)

### Douban

**File:** `cps/metadata_provider/douban.py`

**Search types:**
- Title + Author
- ISBN

**Requirements:**
- None (uses public Douban API)

**Data returned:**
- Title, Author
- Cover image
- Description
- Rating
- Tags
- Publication date

**Use case:**
- Chinese books

### Lubimyczytac

**File:** `cps/metadata_provider/lubimyczytac.py`

**Search types:**
- Title + Author
- ISBN

**Requirements:**
- None (uses public API)

**Data returned:**
- Title, Author
- Cover image
- Description
- Series
- Tags
- Publication date

**Use case:**
- Polish books

### Scholar

**File:** `cps/metadata_provider/scholar.py`

**Search types:**
- Academic papers
- DOI

**Requirements:**
- None (uses scholarly library)

**Data returned:**
- Title, Author
- Description
- Publication year
- Citation count
- DOI

**Use case:**
- Academic papers, textbooks

## Adding a New Provider

### 1. Create Provider Module

Create file: `cps/metadata_provider/myprovider.py`

```python
from cps import logger

log = logger.create()

def search(query):
    """Search MyProvider for book metadata

    Args:
        query: Search query string

    Returns:
        dict: Metadata dict or None
    """
    try:
        # Make API request
        url = f"https://api.myprovider.com/search?q={query}"
        response = requests.get(url, timeout=10)
        response.raise_for_status()

        # Parse response
        data = response.json()

        # Extract metadata
        if not data.get('results'):
            return None

        result = data['results'][0]

        # Return standardized format
        return {
            'title': result.get('title'),
            'author': ', '.join(result.get('authors', [])),
            'cover': result.get('cover_url'),
            'description': result.get('description'),
            'tags': result.get('tags', []),
            'series': result.get('series'),
            'series_id': result.get('series_index'),
            'languages': result.get('languages', ['eng']),
            'publisher': result.get('publisher'),
            'pubdate': result.get('publication_date'),
            'identifiers': {
                'isbn': result.get('isbn'),
                'myprovider': result.get('id')
            }
        }

    except Exception as e:
        log.error(f"MyProvider search failed: {e}")
        return None
```

### 2. Register Provider

Add to `cps/services/Metadata.py`:

```python
from cps.metadata_provider import myprovider

PROVIDERS = {
    # ... existing providers
    'myprovider': myprovider,
}
```

### 3. Add to Admin UI

Edit `cps/templates/config_edit.html` to include provider checkbox.

### 4. Test

```python
from cps.metadata_provider.myprovider import search

result = search("The Great Gatsby")
print(result)
```

## Usage

### Via Web UI

1. Go to book details page
2. Click "Download Metadata"
3. Enter search query (title, author, ISBN)
4. Select provider(s)
5. Click "Search"
6. Review results
7. Select best match
8. Click "Merge"

### Via API (Future)

```python
from cps.services.Metadata import metadata

# Search all providers
results = metadata.search(title="The Great Gatsby")

# Search specific provider
results = metadata.search(provider='amazon', isbn='1234567890')
```

## Configuration

### Provider Priority

Set provider search order in Admin UI:

1. Admin → Basic Configuration
2. Scroll to "Metadata Providers"
3. Reorder providers (drag and drop)
4. Save

### Provider Settings

Per-provider configuration (where applicable):

- **ComicVine:** API key
- **Google Books:** API key (optional, for higher rate limits)
- **Amazon:** None (uses public search)

## Best Practices

### Writing Providers

1. **Handle errors gracefully:**
   ```python
   try:
       # API request
   except requests.Timeout:
       log.error("Provider timeout")
       return None
   except Exception as e:
       log.error(f"Provider error: {e}")
       return None
   ```

2. **Use timeouts:**
   ```python
   response = requests.get(url, timeout=10)
   ```

3. **Cache results:**
   ```python
   # Implement caching to reduce API calls
   ```

4. **Normalize data:**
   ```python
   # Standardize date formats
   # Clean up author names
   # Validate ISBNs
   ```

5. **Respect rate limits:**
   ```python
   import time
   time.sleep(1)  # Delay between requests
   ```

6. **Use user agents:**
   ```python
   headers = {
       'User-Agent': 'Calibre-Web/0.6.0'
   }
   response = requests.get(url, headers=headers)
   ```

### Security

- Sanitize all user input
- Validate all URLs
- Don't expose API keys in logs
- Use HTTPS for API calls
- Rate limit requests

## Troubleshooting

### Provider Not Returning Results

- Check API is accessible
- Verify query format
- Check logs for errors
- Test API manually

### Cover Images Not Loading

- Check image URL is valid
- Verify image format (JPEG, PNG)
- Check for hotlink protection
- Verify network access

### Slow Response Times

- Increase timeout value
- Implement caching
- Reduce number of providers searched
- Check network connectivity

### API Rate Limiting

- Implement exponential backoff
- Cache results longer
- Reduce search frequency
- Use API keys for higher limits

## Performance

### Optimization Tips

1. **Parallel searches** - Search multiple providers concurrently
2. **Caching** - Cache results to reduce API calls
3. **Lazy loading** - Load covers on demand
4. **Debouncing** - Delay searches while user types
5. **Batching** - Search multiple books at once

### Monitoring

Track provider performance:

- Response time
- Success rate
- Error types
- Rate limit hits

## Future Enhancements

Potential improvements:

- User-defined providers
- Provider plugins
- API key management
- Result ranking/scoring
- Automatic metadata matching
- Bulk metadata download
- Provider failover
- Result caching

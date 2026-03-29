# Background Tasks

This document covers Calibre-Web's background task system for long-running operations.

## Overview

Calibre-Web uses a background task worker to handle operations that take too long for a typical HTTP request:

- **eBook conversion** - Format conversion via Calibre
- **Upload processing** - Parsing uploaded eBooks
- **Thumbnail generation** - Creating cover thumbnails
- **Metadata backup** - Backing up book metadata
- **Email sending** - Queuing and sending emails
- **Cache cleanup** - Cleaning old cached files

```
┌─────────────────────────────────────────────────────────────┐
│                     Web Request                             │
│  User clicks "Convert" → Task Queued → Immediate Response   │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Task Queue                                │
│  Task ID: 123, Type: convert, Status: queued               │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                 WorkerThread                                │
│  Background thread processes tasks sequentially             │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Task Execution                            │
│  - Call Calibre ebook-convert                               │
│  - Wait for completion                                      │
│  - Update progress                                          │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                  Task Status                                │
│  - Progress updates via AJAX                                │
│  - Completion notification                                  │
│  - Result display                                           │
└─────────────────────────────────────────────────────────────┘
```

## Architecture

### Components

**WorkerThread** (`cps/services/worker.py`)

Background thread that processes tasks from a queue.

- Runs continuously
- Processes tasks sequentially
- Updates task status
- Handles task errors

**Task Queue**

In-memory queue of pending tasks.

**Task Status** (`cps/tasks_status.py`)

Stores task progress for AJAX polling.

### Task Types

| Task | File | Description |
|------|------|-------------|
| `convert` | `convert.py` | Convert eBook formats |
| `upload` | `upload.py` | Process uploaded eBooks |
| `thumbnail` | `thumbnail.py` | Generate cover thumbnails |
| `metadata_backup` | `metadata_backup.py` | Backup book metadata |
| `mail` | `mail.py` | Send emails via SMTP |
| `clean` | `clean.py` | Clean old cache files |

## Task Flow

### 1. Task Creation

User action triggers task creation:

```python
from cps.services.worker import WorkerThread

# Queue task
task_id = WorkerThread.add(
    user_id=current_user.id,
    task_type='convert',
    task_message='Converting book',
    book_id=book_id,
    from_format='EPUB',
    to_format='MOBI'
)

# Return task_id to client
return jsonify({'task_id': task_id})
```

### 2. Task Processing

WorkerThread picks up task and executes:

```python
def convert_calibre(book_id, from_format, to_format, user_id, task_id):
    """Convert eBook format"""
    try:
        # Get book data
        book = calibre_db.get_book(book_id)
        file_path = get_book_path(book, from_format)

        # Execute Calibre
        cmd = [
            'ebook-convert',
            file_path,
            f'{file_path}.{to_format.lower()}'
        ]

        # Run with progress updates
        result = run_calibre_command(cmd, task_id)

        # Update database
        add_format_to_book(book, to_format)

        return {'result': 'success', 'message': 'Conversion complete'}

    except Exception as e:
        log.error(f"Conversion failed: {e}")
        return {'result': 'error', 'error': str(e)}
```

### 3. Progress Updates

Task updates progress via AJAX:

```javascript
// Poll task status
setInterval(function() {
    $.get('/get_task_status/' + taskId, function(data) {
        updateProgressBar(data.progress);
        if (data.status === 'finished') {
            showResult(data.result);
        }
    });
}, 1000);
```

### 4. Task Completion

Task finishes and stores result:

```python
# Update task status
WorkerThread.update_task_status(
    task_id=task_id,
    status='finished',
    progress=100,
    result={'result': 'success', 'message': 'Done'}
)
```

## Task Types

### Convert Task

Converts eBook format using Calibre's `ebook-convert` tool.

**File:** `cps/tasks/convert.py`

**Configuration:**
- Calibre binary path (Admin UI → Basic Configuration)
- Output format
- Input format

**Process:**
1. Locate source file
2. Execute `ebook-convert`
3. Parse progress from Calibre output
4. Store new format in database
5. Update task status

**Example:**
```python
task_id = WorkerThread.add(
    user_id=user.id,
    task_type='convert',
    task_message='Converting to MOBI',
    book_id=book_id,
    from_format='EPUB',
    to_format='MOBI'
)
```

### Upload Task

Processes uploaded eBook files.

**File:** `cps/tasks/upload.py`

**Process:**
1. Extract metadata from file
2. Generate cover thumbnail
3. Add to Calibre database
4. Move file to library
5. Index for search

**Supported formats:** See `EXTENSIONS_UPLOAD` in `cps/constants.py`

### Thumbnail Task

Generates cover thumbnails for faster loading.

**File:** `cps/tasks/thumbnail.py`

**Sizes:**
- Original (0) - Full size
- Small (1) - 48x48
- Medium (2) - 128x128
- Large (4) - 256x256

**Process:**
1. Extract cover from eBook
2. Resize with ImageMagick/Wand
3. Store in cache
4. Update database

### Metadata Backup Task

Backs up book metadata to JSON.

**File:** `cps/tasks/metadata_backup.py`

**Process:**
1. Export all books to JSON
2. Include authors, tags, series
3. Save to backup directory
4. Compress if large

### Mail Task

Sends eBooks via email.

**File:** `cps/tasks/mail.py`

**Process:**
1. Get book file
2. Attach to email
3. Send via SMTP
4. Update sent status

**Configuration:**
- SMTP server
- SMTP credentials
- From address

### Clean Task

Cleans old cached files.

**File:** `cps/tasks/clean.py`

**Process:**
1. Scan cache directory
2. Delete files older than threshold
3. Update database references
4. Log deleted files

## Scheduled Tasks

Background scheduler (APScheduler) runs periodic tasks.

**File:** `cps/services/background_scheduler.py`

**Scheduled tasks:**
- Metadata backup (daily/weekly)
- Cache cleanup (daily)
- Thumbnail generation (nightly)

**Configuration:** Admin UI → Scheduled Tasks

## Task Status API

### Get Task Status

```
GET /get_task_status/<task_id>
```

**Response:**
```json
{
    "task_id": 123,
    "status": "running",
    "progress": 45,
    "message": "Converting...",
    "result": null
}
```

### List All Tasks

```
GET /get_task_status
```

**Response:**
```json
{
    "tasks": [
        {
            "task_id": 123,
            "task_type": "convert",
            "status": "running",
            "progress": 45
        },
        {
            "task_id": 124,
            "task_type": "upload",
            "status": "finished",
            "progress": 100
        }
    ]
}
```

### Cancel Task

```
POST /tasks/cancel/<task_id>
```

**Note:** Not all tasks can be cancelled (depends on implementation)

## Best Practices

### Writing Task Functions

1. **Accept task_id parameter:**
   ```python
   def my_task(task_id, *args, **kwargs):
       pass
   ```

2. **Update progress:**
   ```python
   WorkerThread.update_task_status(task_id, progress=50)
   ```

3. **Handle errors:**
   ```python
   try:
       # Do work
       return {'result': 'success'}
   except Exception as e:
       log.error(f"Task failed: {e}")
       return {'result': 'error', 'error': str(e)}
   ```

4. **Use long-running operations carefully:**
   - Avoid blocking the worker thread
   - Check for cancellation
   - Clean up resources

### Task Timeouts

- Default: No timeout
- Configure per task if needed

### Error Handling

- Log all errors
- Return error result
- Update task status with error message
- Don't crash WorkerThread

### Resource Management

- Clean up temporary files
- Close file handles
- Release database connections
- Limit concurrent conversions

## Troubleshooting

### Task Not Starting

- Check WorkerThread is running
- Verify task queue not full
- Check logs for errors

### Task Stuck

- Check Calibre binary path
- Verify file permissions
- Check database locks

### Progress Not Updating

- Verify task_id is correct
- Check AJAX polling interval
- Inspect browser console

### Conversion Fails

- Verify Calibre installed
- Check ebook-convert path
- Test conversion manually
- Check input file format

### High Memory Usage

- Limit queue size
- Process tasks in batches
- Clean up completed tasks
- Monitor worker thread

## Configuration

### Via Admin UI

1. Admin → Basic Configuration
2. Set Calibre binary path
3. Configure task timeout
4. Set concurrent task limit

### Via Code

```python
# cps/services/worker.py

# Maximum concurrent tasks
MAX_CONCURRENT_TASKS = 1

# Task timeout (seconds)
TASK_TIMEOUT = 3600
```

## Performance

### Optimization Tips

1. **Batch similar tasks** - Process multiple conversions together
2. **Use thumbnails** - Generate once, cache forever
3. **Limit queue size** - Prevent memory overflow
4. **Schedule during off-hours** - Run heavy tasks at night
5. **Monitor resources** - Check CPU/memory usage

### Monitoring

Check task status in:

- Admin UI → Tasks
- Logs: `cps/logs/`
- Task status API

## Security

### Task Permissions

- Tasks run with web server permissions
- Validate user has permission to execute task
- Sanitize file paths
- Check file permissions

### Resource Limits

- Limit file size for uploads
- Limit conversion time
- Limit concurrent tasks per user
- Clean up temporary files

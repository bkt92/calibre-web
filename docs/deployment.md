# Deployment Guide

This guide covers deploying Calibre-Web to production environments.

## Overview

Calibre-Web can be deployed in various ways:

- **Direct Python** - Run with Python directly
- **Systemd service** - Linux service with auto-restart
- **Docker** - Containerized deployment
- **Reverse proxy** - Behind Nginx/Apache
- **Cloud platforms** - VPS, cloud hosting

## System Requirements

### Minimum

- **CPU:** 1 core
- **RAM:** 512 MB
- **Storage:** 1 GB + library size
- **Python:** 3.8+

### Recommended

- **CPU:** 2+ cores
- **RAM:** 2 GB
- **Storage:** SSD + library size
- **Python:** 3.12+
- **ImageMagick:** For cover extraction

### Optional

- **Calibre desktop:** For on-the-fly conversion
- **Kepubify:** For Kobo device support
- **LDAP server:** For LDAP authentication
- **Redis:** For rate limiting (large deployments)

## Installation

### Method 1: pip (Recommended)

```bash
# Create virtual environment
python3 -m venv /opt/calibre-web
source /opt/calibre-web/bin/activate

# Install with optional features
pip install calibreweb[gdrive,oauth,ldap]

# Create app directory
mkdir -p /var/lib/calibre-web
cd /var/lib/calibre-web

# Create settings database
python -c "from cps import ub; ub.init_db('app.db')"
```

### Method 2: From Source

```bash
# Clone repository
git clone https://github.com/janeczku/calibre-web.git /opt/calibre-web
cd /opt/calibre-web

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
pip install -e ".[gdrive,oauth,ldap]"
```

### Method 3: Docker

```bash
# Pull image
docker pull linuxserver/calibre-web

# Run container
docker run -d \
  --name calibre-web \
  -p 8083:8083 \
  -v /path/to/library:/books \
  -v /path/to/config:/config \
  linuxserver/calibre-web
```

See [Docker Images](README.md#docker-images) for details.

## Configuration

### Basic Setup

1. **Create Calibre library:**
   ```bash
   # Use existing Calibre library
   cp /path/to/calibre/library/metadata.db /var/lib/calibre-web/

   # Or create new one with Calibre desktop
   ```

2. **Set permissions:**
   ```bash
   # Create user
   useradd -r -s /bin/false calibre-web

   # Set ownership
   chown -R calibre-web:calibre-web /var/lib/calibre-web
   chown -R calibre-web:calibre-web /opt/calibre-web
   ```

3. **Create configuration:**
   ```bash
   # Run app once to create config
   sudo -u calibre-web cps -p /var/lib/calibre-web/app.db
   ```

### Environment Variables

```bash
# Configuration
export CALIBRE_DBPATH="/var/lib/calibre-web"
export CALIBRE_PORT="8083"
export CACHE_DIRECTORY="/var/cache/calibre-web"

# Security
export SECRET_KEY="your-secret-key-here"
export COOKIE_PREFIX="cw_"

# Rate limiting (Redis)
export RATELIMIT_STORAGE_URI="redis://localhost:6379"
export RATELIMIT_STORAGE_OPTIONS="{'connection_pool_size': 10}"
```

### Command Line Options

```bash
cps \
  -p /var/lib/calibre-web/app.db \    # Settings database
  -i 127.0.0.1 \                      # Listen address
  -o /var/log/calibre-web/app.log \   # Log file
  -c /etc/ssl/certs/cw.crt \          # SSL cert
  -k /etc/ssl/private/cw.key          # SSL key
```

## Systemd Service

### Create Service File

`/etc/systemd/system/calibre-web.service`:

```ini
[Unit]
Description=Calibre-Web
After=network.target

[Service]
Type=simple
User=calibre-web
Group=calibre-web
WorkingDirectory=/var/lib/calibre-web
ExecStart=/opt/calibre-web/bin/cps \
  -p /var/lib/calibre-web/app.db \
  -i 127.0.0.1 \
  -o /var/log/calibre-web/app.log
Restart=always
RestartSec=10

# Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/calibre-web /var/cache/calibre-web /var/log/calibre-web

# Security
# Uncomment if Calibre binaries are needed:
# ReadWritePaths=/usr/bin

[Install]
WantedBy=multi-user.target
```

### Enable and Start

```bash
# Reload systemd
systemctl daemon-reload

# Enable service
systemctl enable calibre-web

# Start service
systemctl start calibre-web

# Check status
systemctl status calibre-web

# View logs
journalctl -u calibre-web -f
```

## Reverse Proxy

### Nginx

`/etc/nginx/sites-available/calibre-web`:

```nginx
server {
    listen 80;
    server_name books.example.com;

    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name books.example.com;

    # SSL certificates
    ssl_certificate /etc/letsencrypt/live/books.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/books.example.com/privkey.pem;

    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256';

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;

    # Upload limit
    client_max_body_size 100M;

    # Proxy settings
    location / {
        proxy_pass http://127.0.0.1:8083;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support (if needed)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # Timeouts
        proxy_connect_timeout 600;
        proxy_send_timeout 600;
        proxy_read_timeout 600;
        send_timeout 600;
    }
}
```

### Apache

`/etc/apache2/sites-available/calibre-web.conf`:

```apache
<VirtualHost *:80>
    ServerName books.example.com
    Redirect permanent / https://books.example.com/
</VirtualHost>

<VirtualHost *:443>
    ServerName books.example.com

    # SSL
    SSLEngine on
    SSLCertificateFile /etc/letsencrypt/live/books.example.com/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/books.example.com/privkey.pem

    # Security headers
    Header always set Strict-Transport-Security "max-age=31536000"
    Header always set X-Frame-Options "DENY"
    Header always set X-Content-Type-Options "nosniff"

    # Proxy
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8083/
    ProxyPassReverse / http://127.0.0.1:8083/

    # Upload limit
    LimitRequestBody 104857600

    # Timeouts
    ProxyTimeout 600
</VirtualHost>
```

## SSL/TLS Setup

### Let's Encrypt (Certbot)

```bash
# Install certbot
apt install certbot python3-certbot-nginx

# Get certificate
certbot --nginx -d books.example.com

# Auto-renewal (cron)
echo "0 0 * * * certbot renew --quiet" | crontab -
```

### Self-Signed Certificate

```bash
# Generate certificate
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/cw.key \
  -out /etc/ssl/certs/cw.crt

# Set permissions
chmod 600 /etc/ssl/private/cw.key
chmod 644 /etc/ssl/certs/cw.crt

# Use with Calibre-Web
cps -c /etc/ssl/certs/cw.crt -k /etc/ssl/private/cw.key
```

## Security

### Firewall

```bash
# UFW (Ubuntu)
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable

# firewalld (CentOS)
firewall-cmd --permanent --add-service=http
firewall-cmd --permanent --add-service=https
firewall-cmd --reload
```

### AppArmor

`/etc/apparmor.d/usr.local.bin.cps`:

```
#include <tunables/global>

/opt/calibre-web/bin/cps {
  #include <abstractions/base>
  #include <abstractions/python>

  /opt/calibre-web/** r,
  /var/lib/calibre-web/** rw,
  /var/cache/calibre-web/** rw,
  /var/log/calibre-web/** w,

  deny /root/** rw,
  deny /home/** rw,
}
```

### Fail2Ban

`/etc/fail2ban/jail.local`:

```ini
[calibre-web]
enabled = true
port = http,https
filter = calibre-web
logpath = /var/log/calibre-web/app.log
maxretry = 5
bantime = 3600
findtime = 600
```

`/etc/fail2ban/filter.d/calibre-web.conf`:

```ini
[Definition]
failregex = ^.* Login failed for user ".*" IP: <HOST>
ignoreregex =
```

## Backup

### Database Backup

```bash
#!/bin/bash
# Backup script: /usr/local/bin/backup-calibre-web.sh

BACKUP_DIR="/backup/calibre-web"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR/$DATE"

# Backup Calibre database
cp /var/lib/calibre-web/metadata.db "$BACKUP_DIR/$DATE/"

# Backup app database
cp /var/lib/calibre-web/app.db "$BACKUP_DIR/$DATE/"

# Backup Google Drive database (if used)
cp /var/lib/calibre-web/gdrive.db "$BACKUP_DIR/$DATE/" 2>/dev/null

# Compress
tar -czf "$BACKUP_DIR/calibre-web-$DATE.tar.gz" -C "$BACKUP_DIR" "$DATE"
rm -rf "$BACKUP_DIR/$DATE"

# Keep last 30 days
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete
```

### Cron Job

```bash
# Daily backup at 2 AM
0 2 * * * /usr/local/bin/backup-calibre-web.sh
```

### Restore

```bash
# Extract backup
tar -xzf /backup/calibre-web/calibre-web-20240101_020000.tar.gz -C /tmp

# Stop service
systemctl stop calibre-web

# Restore databases
cp /tmp/calibre-web-20240101_020000/metadata.db /var/lib/calibre-web/
cp /tmp/calibre-web-20240101_020000/app.db /var/lib/calibre-web/

# Start service
systemctl start calibre-web
```

## Monitoring

### Log Monitoring

```bash
# View logs
journalctl -u calibre-web -f

# Rotate logs
logrotate /etc/logrotate.d/calibre-web
```

`/etc/logrotate.d/calibre-web`:

```
/var/log/calibre-web/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 calibre-web calibre-web
}
```

### Health Check

```bash
#!/bin/bash
# Health check script

# Check if service is running
if ! systemctl is-active --quiet calibre-web; then
    echo "Service not running"
    exit 1
fi

# Check if port is listening
if ! nc -z localhost 8083; then
    echo "Port not listening"
    exit 1
fi

# Check HTTP response
if ! curl -f http://localhost:8083/ > /dev/null; then
    echo "HTTP check failed"
    exit 1
fi

echo "OK"
exit 0
```

### Performance Monitoring

Use tools like:
- **htop** - System resources
- **iotop** - Disk I/O
- **netstat** - Network connections
- **Prometheus + Grafana** - Advanced monitoring

## Scaling

### Multiple Instances

For high-availability, run multiple instances behind load balancer:

```nginx
upstream calibre-web {
    server 127.0.0.1:8083;
    server 127.0.0.1:8084;
    server 127.0.0.1:8085;
}

server {
    location / {
        proxy_pass http://calibre-web;
    }
}
```

**Note:** Use shared database and cache for all instances.

### Caching

Use Redis for rate limiting and session storage:

```bash
# Install Redis
apt install redis-server

# Configure Calibre-Web
export RATELIMIT_STORAGE_URI="redis://localhost:6379"
```

### CDN

Offload static assets to CDN:

```nginx
# Static files from CDN
location /static/ {
    proxy_pass https://cdn.example.com/calibre-web/static/;
}

# Everything else to app
location / {
    proxy_pass http://127.0.0.1:8083;
}
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
journalctl -u calibre-web -n 50

# Check configuration
cps -p /var/lib/calibre-web/app.db -d

# Check permissions
ls -la /var/lib/calibre-web/
```

### Database Locked

```bash
# Close Calibre desktop app
# Check for other processes
lsof | grep app.db

# Restart service
systemctl restart calibre-web
```

### High Memory Usage

```bash
# Check memory usage
ps aux | grep cps

# Reduce cache size
export CACHE_DIRECTORY="/tmp/calibre-web-cache"

# Restart service
systemctl restart calibre-web
```

### Slow Performance

```bash
# Check disk I/O
iotop

# Check database size
ls -lh /var/lib/calibre-web/

# Optimize database
sqlite3 /var/lib/calibre-web/metadata.db "VACUUM;"
```

## Production Checklist

- [ ] Change default admin password
- [ ] Enable SSL/TLS
- [ ] Configure firewall
- [ ] Set up reverse proxy
- [ ] Configure backups
- [ ] Set up log rotation
- [ ] Configure rate limiting
- [ ] Enable authentication
- [ ] Set up monitoring
- [ ] Test disaster recovery
- [ ] Document configuration
- [ ] Review security settings
- [ ] Set up fail2ban
- [ ] Configure AppArmor
- [ ] Test SSL certificate
- [ ] Check permissions

## Resources

- **Main repository:** https://github.com/janeczku/calibre-web
- **Wiki:** https://github.com/janeczku/calibre-web/wiki
- **Docker Hub:** https://hub.docker.com/r/linuxserver/calibre-web
- **Discord:** https://discord.gg/h2VsJ2NEfB
- **Issues:** https://github.com/janeczku/calibre-web/issues

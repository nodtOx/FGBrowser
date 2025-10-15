# FitBoy Server Deployment

This folder contains everything needed to deploy and run the FitBoy database server.

## Files

- **`deploy.sh`** - One-command deployment script
- **`update.sh`** - Database update script (runs on server)
- **`nginx.conf`** - Web server configuration

## Quick Start

### 1. Setup SSH Access

```bash
ssh-copy-id root@157.230.16.45
```

### 2. Build CLI (GitHub Actions)

Push to main branch or manually trigger:

- **Automatic**: Push changes to `main` branch
- **Manual**: GitHub → Actions → Build CLI Binary → Run workflow

Builds on GitHub's servers (fast, free) and creates a release.

### 3. Deploy

```bash
cd server
./deploy.sh
```

The script will:

- Download pre-built CLI from GitHub releases (or build locally as fallback)
- Upload files to server
- Install nginx
- Configure auto-updates (every 6 hours)
- Run initial database update

## What Gets Deployed

**Server:** 157.230.16.45

**Directories:**

- `/opt/fitboy/` - CLI binary and scripts
- `/var/www/fitboy/` - Public web directory
- `/var/log/fitboy/` - Update logs

**Endpoints:**

- `http://157.230.16.45/repacks.db` - Database file
- `http://157.230.16.45/metadata.json` - Metadata (version, size, game count)

**Cron Job:**

- Runs `/opt/fitboy/update.sh` every 6 hours
- Logs to `/var/log/fitboy/update.log`

## Server Commands

SSH into the server:

```bash
ssh root@157.230.16.45
```

**Manual update:**

```bash
/opt/fitboy/update.sh
```

**View logs:**

```bash
tail -f /var/log/fitboy/update.log
```

**Check cron jobs:**

```bash
crontab -l
```

**View database stats:**

```bash
/opt/fitboy/cli stats
```

**Test nginx config:**

```bash
nginx -t
```

**Reload nginx:**

```bash
systemctl reload nginx
```

## Updating Configuration

### Update nginx config:

1. Edit `server/nginx.conf` locally
2. Run `./deploy.sh` again

### Update update script:

1. Edit `server/update.sh` locally
2. Run `./deploy.sh` again

## Monitoring

**Check if service is running:**

```bash
curl -I http://157.230.16.45/metadata.json
```

**View metadata:**

```bash
curl http://157.230.16.45/metadata.json | jq
```

**Check database size:**

```bash
ssh root@157.230.16.45 "ls -lh /var/www/fitboy/repacks.db"
```

**View recent log entries:**

```bash
ssh root@157.230.16.45 "tail -20 /var/log/fitboy/update.log"
```

## Troubleshooting

### Database not updating

```bash
ssh root@157.230.16.45
/opt/fitboy/update.sh  # Run manually to see errors
```

### nginx not responding

```bash
ssh root@157.230.16.45
systemctl status nginx
nginx -t  # Test config
```

### Can't connect to server

```bash
ping 157.230.16.45
ssh -v root@157.230.16.45  # Verbose SSH
```

## Architecture

```
Local Machine                    Server (157.230.16.45)
┌────────────────┐              ┌─────────────────────────┐
│                │              │                         │
│  deploy.sh     │─────────────>│  /opt/fitboy/           │
│                │   uploads    │    ├── cli              │
│  ├─ Builds CLI │              │    └── update.sh        │
│  ├─ Uploads    │              │                         │
│  └─ Configures │              │  nginx                  │
│                │              │    └─> /var/www/fitboy/ │
└────────────────┘              │          ├── repacks.db │
                                │          └── metadata   │
                                └─────────────────────────┘

                                Cron: Every 6 hours
                                └─> /opt/fitboy/update.sh
                                      ├─ Crawls site
                                      ├─ Updates DB
                                      └─ Publishes
```

## Cost

**DigitalOcean Droplet:**

- $6/month for smallest droplet
- Enough for database hosting

**Bandwidth:**

- ~50-200MB per database download
- Most downloads happen once per user
- Minimal ongoing cost

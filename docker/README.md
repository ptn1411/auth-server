# Docker Setup for Auth Server

Hướng dẫn chạy Auth Server với Docker Compose.

## Development với Domain Ảo (giống Laragon)

Setup domain ảo cho dev environment:

```bash
# 1. Chạy script setup hosts (Run as Admin)
scripts\setup-hosts.bat

# 2. Start services
docker-compose -f docker-compose.dev.yml up -d

# 3. Truy cập
# Frontend: http://auth.local
# Backend:  http://api.auth.local
```

### Manual setup hosts (nếu không dùng script)

Thêm vào `C:\Windows\System32\drivers\etc\hosts`:
```
127.0.0.1 auth.local
127.0.0.1 api.auth.local
```

### Commands cho dev

```bash
# Start
docker-compose -f docker-compose.dev.yml up -d

# Logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop
docker-compose -f docker-compose.dev.yml down

# Rebuild
docker-compose -f docker-compose.dev.yml up -d --build
```

---

## Quick Start (localhost)

### 1. Chuẩn bị

```bash
# Copy environment file
cp docker/.env.example .env

# Generate JWT keys (nếu chưa có)
mkdir -p keys
openssl genrsa -out keys/private.pem 2048
openssl rsa -in keys/private.pem -pubout -out keys/public.pem
```

### 2. Chạy Development

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### 3. Truy cập

- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:3000
- **MySQL**: localhost:3306

## Services

| Service | Port | Description |
|---------|------|-------------|
| `mysql` | 3306 | MySQL 8.0 database |
| `backend` | 3000 | Rust Auth Server |
| `frontend` | 5173 | React Admin UI |
| `nginx` | 80/443 | Reverse proxy (production) |

## Commands

```bash
# Start services
docker-compose up -d

# Start with rebuild
docker-compose up -d --build

# View logs
docker-compose logs -f
docker-compose logs -f backend

# Stop services
docker-compose down

# Stop and remove volumes (reset database)
docker-compose down -v

# Restart a service
docker-compose restart backend

# Execute command in container
docker-compose exec backend /bin/sh
docker-compose exec mysql mysql -u root -p

# View running containers
docker-compose ps
```

## Production Deployment

### 1. Enable Nginx reverse proxy

```bash
# Start with production profile
docker-compose --profile production up -d
```

### 2. Configure SSL

1. Place SSL certificates in `docker/nginx/ssl/`:
   - `fullchain.pem`
   - `privkey.pem`

2. Uncomment HTTPS server block in `docker/nginx/nginx.conf`

3. Update `.env`:
   ```env
   APP_URL=https://your-domain.com
   WEBAUTHN_RP_ORIGIN=https://your-domain.com
   ```

### 3. Using Let's Encrypt

```bash
# Install certbot
docker run -it --rm \
  -v ./docker/nginx/ssl:/etc/letsencrypt \
  -v ./docker/nginx/www:/var/www/certbot \
  certbot/certbot certonly \
  --webroot -w /var/www/certbot \
  -d your-domain.com
```

## Environment Variables

### MySQL

| Variable | Default | Description |
|----------|---------|-------------|
| `MYSQL_ROOT_PASSWORD` | rootpassword | Root password |
| `MYSQL_DATABASE` | auth_server | Database name |
| `MYSQL_USER` | authserver | Database user |
| `MYSQL_PASSWORD` | authpassword | User password |

### Backend

| Variable | Default | Description |
|----------|---------|-------------|
| `ACCESS_TOKEN_EXPIRY_SECS` | 900 | Access token TTL |
| `REFRESH_TOKEN_EXPIRY_SECS` | 604800 | Refresh token TTL |
| `RUST_LOG` | info | Log level |

### Frontend

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_API_URL` | http://localhost:3000 | Backend API URL |

## Troubleshooting

### Database connection failed

```bash
# Check MySQL is running
docker-compose ps mysql

# Check MySQL logs
docker-compose logs mysql

# Wait for MySQL to be ready
docker-compose exec mysql mysqladmin ping -h localhost -u root -p
```

### Backend won't start

```bash
# Check backend logs
docker-compose logs backend

# Verify JWT keys exist
ls -la keys/

# Check database connection
docker-compose exec backend curl http://localhost:3000/health
```

### Frontend build fails

```bash
# Rebuild frontend
docker-compose build --no-cache frontend

# Check SDK is built
docker-compose exec frontend ls -la /app/sdk/dist
```

### Reset everything

```bash
# Stop and remove all containers, volumes, networks
docker-compose down -v --remove-orphans

# Remove images
docker-compose down --rmi all

# Start fresh
docker-compose up -d --build
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Docker Network                        │
│                                                              │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
│  │  Nginx   │───▶│ Frontend │    │  MySQL   │              │
│  │  :80/443 │    │   :80    │    │  :3306   │              │
│  └────┬─────┘    └──────────┘    └────▲─────┘              │
│       │                               │                     │
│       │          ┌──────────┐         │                     │
│       └─────────▶│ Backend  │─────────┘                     │
│                  │  :3000   │                               │
│                  └──────────┘                               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
         │              │
         ▼              ▼
    Port 80/443    Port 3000/5173
    (Production)   (Development)
```

## Volumes

| Volume | Path | Description |
|--------|------|-------------|
| `mysql_data` | /var/lib/mysql | MySQL data persistence |
| `./keys` | /app/keys | JWT keys (read-only) |

## Health Checks

All services have health checks configured:

```bash
# Check all services health
docker-compose ps

# Manual health check
curl http://localhost:3000/health
curl http://localhost:5173/health
```

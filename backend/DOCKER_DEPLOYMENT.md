# Docker Deployment Guide

This guide explains how to deploy the forum backend using Docker.

## Prerequisites

- Docker and Docker Compose installed
- At least 2GB of free disk space
- Git (optional, for cloning the repository)

## Quick Start

1. **Clone the repository** (if not already done):
   ```bash
   git clone <repository-url>
   cd forum/backend
   ```

2. **Build and run with Docker Compose**:
   ```bash
   docker-compose up -d
   ```

3. **Check the logs**:
   ```bash
   docker-compose logs -f backend
   ```

4. **Access the application**:
   - HTTP: http://localhost:8000
   - HTTPS: https://localhost:8080 (if certificates are configured)

## Configuration

### Environment Variables

The application can be configured using environment variables. You can set them in:

1. **docker-compose.yml** file (recommended for development)
2. **.env file** (mounted as volume)
3. **Docker run command** (for production)

Key environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://postgres:password@postgres:5432/postgres` |
| `HOST_URL` | Host address to bind to | `0.0.0.0` |
| `JWT_SECRET_KEY` | Secret key for JWT tokens | `change_this_in_production` |
| `JWT_MAXAGE` | JWT token expiration in minutes | `60` |
| `PORT_HTTP` | HTTP port | `8000` |
| `PORT_HTTPS` | HTTPS port | `8080` |
| `UPLOAD_DIR` | Directory for file uploads | `./uploads` |
| `MAX_FILE_SIZE` | Maximum file size in bytes | `5242880` (5MB) |
| `VERIFY_EMAIL` | Enable email verification | `false` |
| `ENABLE_HTTPS` | Enable HTTPS | `false` |

### SMTP Configuration (for email verification)

If `VERIFY_EMAIL=true`, configure SMTP settings:

```yaml
SMTP_SERVER: smtp.your-email-provider.com
SMTP_PORT: 587
SMTP_USERNAME: your_email@example.com
SMTP_PASSWORD: your_email_password
SMTP_FROM_ADDRESS: no-reply@yourdomain.com
```

### OAuth Configuration

For OAuth authentication (Google, Facebook, Discord), configure:

```yaml
GOOGLE_CLIENT_ID: your_client_id
GOOGLE_CLIENT_SECRET: your_client_secret
GOOGLE_REDIRECT_URI: http://localhost:3000/auth/google/callback

FACEBOOK_CLIENT_ID: your_facebook_client_id
FACEBOOK_CLIENT_SECRET: your_facebook_client_secret
FACEBOOK_REDIRECT_URI: http://localhost:3000/auth/facebook/callback
```

## Production Deployment

### 1. Build the Docker image

```bash
docker build -t forum-backend:latest .
```

### 2. Run with custom configuration

```bash
docker run -d \
  --name forum-backend \
  -p 8000:8000 \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://user:password@host:5432/db \
  -e JWT_SECRET_KEY=your_secure_secret_key \
  -v ./uploads:/app/uploads \
  -v ./cert.pem:/app/cert.pem \
  -v ./key.pem:/app/key.pem \
  forum-backend:latest
```

### 3. Using Docker Compose for production

Create a `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: ${DB_NAME}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./schema.sql:/docker-entrypoint-initdb.d/schema.sql
    restart: always

  backend:
    build: .
    ports:
      - "8000:8000"
      - "8080:8080"
    environment:
      DATABASE_URL: postgresql://${DB_USER}:${DB_PASSWORD}@postgres:5432/${DB_NAME}
      JWT_SECRET_KEY: ${JWT_SECRET}
      # Add other environment variables as needed
    volumes:
      - uploads:/app/uploads
      - ./certs:/app/certs
    depends_on:
      - postgres
    restart: always

volumes:
  postgres_data:
  uploads:
```

Create a `.env.prod` file with your production values:

```env
DB_USER=forum_user
DB_PASSWORD=strong_password
DB_NAME=forum_db
JWT_SECRET=very_strong_jwt_secret_key
```

Run with:

```bash
docker-compose -f docker-compose.prod.yml --env-file .env.prod up -d
```

## SSL/TLS Configuration

For HTTPS support:

1. Generate SSL certificates:
   ```bash
   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
   ```

2. Mount certificates as volumes:
   ```yaml
   volumes:
     - ./cert.pem:/app/cert.pem
     - ./key.pem:/app/key.pem
   ```

3. Set `ENABLE_HTTPS=true` in environment variables

## Health Checks

The container includes a health check that runs every 30 seconds:

```bash
# Check container health
docker ps --filter "health=healthy"

# View health status
docker inspect --format='{{.State.Health.Status}}' forum-backend
```

## Monitoring

### View logs
```bash
docker-compose logs -f backend
```

### Check resource usage
```bash
docker stats
```

### Access the container shell
```bash
docker exec -it forum-backend /bin/bash
```

## Troubleshooting

### Database connection issues
1. Check if PostgreSQL is running: `docker-compose ps`
2. Check PostgreSQL logs: `docker-compose logs postgres`
3. Verify connection string in environment variables

### Application won't start
1. Check application logs: `docker-compose logs backend`
2. Verify all required environment variables are set
3. Check file permissions for uploads directory

### File upload issues
1. Ensure uploads directory is mounted as volume
2. Check permissions: `docker exec forum-backend ls -la /app/uploads`
3. Verify `MAX_FILE_SIZE` is set appropriately

## Backup and Restore

### Backup database
```bash
docker exec -t postgres pg_dumpall -c -U postgres > dump_$(date +%Y-%m-%d).sql
```

### Backup uploads
```bash
tar -czf uploads_backup_$(date +%Y-%m-%d).tar.gz ./uploads
```

### Restore database
```bash
cat dump.sql | docker exec -i postgres psql -U postgres
```


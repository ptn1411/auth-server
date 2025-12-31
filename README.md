# Auth Server

A centralized authentication server built with Rust for managing authentication and authorization across multiple applications (Apps). Provides SSO (Single Sign-On), User management, and Role-Based Access Control (RBAC) scoped to individual apps.

## Features

- **User Authentication**: Registration, login, password reset
- **JWT Tokens**: RS256-signed access and refresh tokens
- **Multi-App Support**: Each app has its own roles and permissions
- **RBAC**: Role-Based Access Control scoped to apps
- **Secure Password Storage**: Argon2 password hashing

## Tech Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: MySQL with SQLx
- **Password Hashing**: Argon2
- **JWT**: RS256 algorithm with jsonwebtoken crate

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- MySQL 8.0+
- OpenSSL (for RSA key generation)

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd auth-server
Import-Certificate -FilePath "docker\nginx\ssl\auth.local.crt" -CertStoreLocation Cert:\LocalMachine\Root
```

### 2. Database Setup

Create a MySQL database:

```bash
# Connect to MySQL
mysql -u root -p

# Create database
CREATE DATABASE auth_server;

# Exit
exit
```

### 3. Environment Configuration

Copy the example environment file and configure:

```bash
cp .env.example .env
```

Edit `.env` with your settings:

```env
# Database
DATABASE_URL=mysql://root:password@localhost/auth_server

# Token Expiry (in seconds)
ACCESS_TOKEN_EXPIRY_SECS=900       # 15 minutes
REFRESH_TOKEN_EXPIRY_SECS=604800  # 7 days

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Logging
RUST_LOG=auth_server=debug,tower_http=debug
```

### 4. Generate RSA Keys (Production)

For production, generate proper RSA keys:

```bash
# Create keys directory
mkdir -p keys

# Generate private key
openssl genrsa -out keys/private.pem 2048

# Generate public key
openssl rsa -in keys/private.pem -pubout -out keys/public.pem
```

The server will automatically load keys from `keys/private.pem` and `keys/public.pem`.

Alternatively, set keys via environment variables:
```env
JWT_PRIVATE_KEY=<your-private-key-pem-content>
JWT_PUBLIC_KEY=<your-public-key-pem-content>
```

### 5. Run Migrations

Migrations run automatically on server startup, or run manually:

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features mysql

# Run migrations
sqlx migrate run
```

### 6. Build and Run

```bash
# Development
cargo run

# Production (release build)
cargo build --release
./target/release/auth-server
```

The server will start at `http://localhost:3000`.

## API Endpoints

### Public Endpoints (No Authentication Required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/register` | Register a new user |
| POST | `/auth/login` | Authenticate and get tokens |
| POST | `/auth/refresh` | Refresh access token |
| POST | `/auth/forgot-password` | Initiate password reset |
| POST | `/auth/reset-password` | Complete password reset |

### Protected Endpoints (JWT Required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/apps` | Create a new app |
| POST | `/apps/{app_id}/roles` | Create a role for an app |
| POST | `/apps/{app_id}/permissions` | Create a permission for an app |
| POST | `/apps/{app_id}/users/{user_id}/roles` | Assign a role to a user |

## Usage Examples

### Register a User

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "SecurePassword123!"}'
```

### Login

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "SecurePassword123!"}'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

### Create an App (Protected)

```bash
curl -X POST http://localhost:3000/apps \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"code": "my-app", "name": "My Application"}'
```

### Create a Role

```bash
curl -X POST http://localhost:3000/apps/{app_id}/roles \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"name": "admin"}'
```

### Create a Permission

```bash
curl -X POST http://localhost:3000/apps/{app_id}/permissions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"code": "read:users"}'
```

### Assign Role to User

```bash
curl -X POST http://localhost:3000/apps/{app_id}/users/{user_id}/roles \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"role_id": "<role_uuid>"}'
```

### Refresh Token

```bash
curl -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "<refresh_token>"}'
```

## JWT Token Structure

Access tokens contain the following claims:

```json
{
  "sub": "user-uuid",
  "apps": {
    "my-app": {
      "roles": ["admin", "user"],
      "permissions": ["read:users", "write:users"]
    }
  },
  "exp": 1703865600,
  "iat": 1703864700
}
```

## Database Schema

The server uses the following tables:

- `users` - User accounts
- `apps` - Registered applications
- `roles` - Roles scoped to apps
- `permissions` - Permissions scoped to apps
- `user_app_roles` - User-App-Role associations
- `role_permissions` - Role-Permission associations
- `refresh_tokens` - Refresh token storage
- `password_reset_tokens` - Password reset token storage

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | MySQL connection string | `mysql://root:password@localhost/auth_server` |
| `JWT_PRIVATE_KEY` | RSA private key (PEM format) | Loaded from `keys/private.pem` |
| `JWT_PUBLIC_KEY` | RSA public key (PEM format) | Loaded from `keys/public.pem` |
| `ACCESS_TOKEN_EXPIRY_SECS` | Access token expiry in seconds | `900` (15 minutes) |
| `REFRESH_TOKEN_EXPIRY_SECS` | Refresh token expiry in seconds | `604800` (7 days) |
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `3000` |
| `RUST_LOG` | Log level configuration | `auth_server=debug,tower_http=debug` |

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## API Documentation

Full API documentation is available in OpenAPI format:
- [openapi.yaml](./openapi.yaml)

You can view the documentation using:
- [Swagger Editor](https://editor.swagger.io/) - Paste the openapi.yaml content
- [Swagger UI](https://swagger.io/tools/swagger-ui/) - Host locally

## Security Considerations

1. **Never use default keys in production** - Always generate proper RSA keys
2. **Use HTTPS** - Deploy behind a reverse proxy with TLS
3. **Secure database** - Use strong passwords and restrict network access
4. **Token expiry** - Access tokens expire in 15 minutes by default
5. **Password requirements** - Enforce strong passwords

## License

MIT

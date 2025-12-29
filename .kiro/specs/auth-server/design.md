# Design Document: Auth Server

## Overview

Auth Server là hệ thống xác thực trung tâm được xây dựng bằng Rust, sử dụng axum framework. Server cung cấp:
- Quản lý User (đăng ký, đăng nhập, reset password)
- Quản lý App, Role, Permission theo scope
- JWT authentication với RS256
- RBAC authorization

### Key Design Decisions

1. **RS256 over HS256**: Sử dụng asymmetric keys để app con có thể verify token bằng public key mà không cần shared secret
2. **Scoped RBAC**: Role và Permission luôn gắn với App, không có global role
3. **Argon2 for passwords**: Algorithm được khuyến nghị cho password hashing
4. **15-minute token expiry**: Balance giữa security và UX, kết hợp với refresh token

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Auth Server                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Handlers  │  │ Middleware  │  │      Services           │  │
│  │  (Routes)   │──│  (JWT Auth) │──│  - AuthService          │  │
│  │             │  │             │  │  - UserService          │  │
│  └─────────────┘  └─────────────┘  │  - AppService           │  │
│                                     │  - RoleService          │  │
│                                     │  - PermissionService    │  │
│                                     └───────────┬─────────────┘  │
│                                                 │                │
│  ┌─────────────────────────────────────────────┴───────────────┐│
│  │                    Repository Layer                          ││
│  │  - UserRepository                                            ││
│  │  - AppRepository                                             ││
│  │  - RoleRepository                                            ││
│  │  - PermissionRepository                                      ││
│  └──────────────────────────┬──────────────────────────────────┘│
│                             │                                    │
│  ┌──────────────────────────┴──────────────────────────────────┐│
│  │                    MySQL (sqlx)                              ││
│  └──────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Router
    participant Middleware
    participant Handler
    participant Service
    participant Repository
    participant DB

    Client->>Router: HTTP Request
    Router->>Middleware: Route matching
    Middleware->>Middleware: JWT Verification (if protected)
    Middleware->>Handler: Inject Claims
    Handler->>Service: Business Logic
    Service->>Repository: Data Access
    Repository->>DB: SQL Query
    DB-->>Repository: Result
    Repository-->>Service: Domain Object
    Service-->>Handler: Response Data
    Handler-->>Client: JSON Response
```

## Components and Interfaces

### 1. Domain Models

```rust
// User domain
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

// App domain
pub struct App {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

// Role domain (scoped to App)
pub struct Role {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
}

// Permission domain (scoped to App)
pub struct Permission {
    pub id: Uuid,
    pub app_id: Uuid,
    pub code: String,
}

// User-App-Role association
pub struct UserAppRole {
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub role_id: Uuid,
}

// Role-Permission association
pub struct RolePermission {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}
```

### 2. Service Interfaces

```rust
// AuthService - handles authentication
pub trait AuthService {
    async fn register(&self, email: &str, password: &str) -> Result<User, AuthError>;
    async fn login(&self, email: &str, password: &str) -> Result<TokenPair, AuthError>;
    async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, AuthError>;
    async fn forgot_password(&self, email: &str) -> Result<(), AuthError>;
    async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AuthError>;
}

// AppService - manages apps
pub trait AppService {
    async fn create_app(&self, code: &str, name: &str) -> Result<App, AppError>;
    async fn get_app(&self, id: Uuid) -> Result<Option<App>, AppError>;
    async fn get_app_by_code(&self, code: &str) -> Result<Option<App>, AppError>;
}

// RoleService - manages roles within apps
pub trait RoleService {
    async fn create_role(&self, app_id: Uuid, name: &str) -> Result<Role, RoleError>;
    async fn get_roles_by_app(&self, app_id: Uuid) -> Result<Vec<Role>, RoleError>;
    async fn assign_role_to_user(&self, user_id: Uuid, app_id: Uuid, role_id: Uuid) -> Result<(), RoleError>;
}

// PermissionService - manages permissions within apps
pub trait PermissionService {
    async fn create_permission(&self, app_id: Uuid, code: &str) -> Result<Permission, PermissionError>;
    async fn get_permissions_by_app(&self, app_id: Uuid) -> Result<Vec<Permission>, PermissionError>;
    async fn assign_permission_to_role(&self, role_id: Uuid, permission_id: Uuid) -> Result<(), PermissionError>;
}

// Authorization check
pub fn can(claims: &Claims, app_code: &str, permission: &str) -> bool;
```

### 3. JWT Structure

```rust
// Token pair returned on login/refresh
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,  // "Bearer"
    pub expires_in: i64,     // seconds
}

// JWT Claims structure
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub apps: HashMap<String, AppClaims>,  // app_code -> claims
    pub exp: i64,     // expiration timestamp
    pub iat: i64,     // issued at
}

#[derive(Serialize, Deserialize)]
pub struct AppClaims {
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}
```

### 4. API Handlers

```rust
// Auth routes
POST /auth/register     -> register_handler
POST /auth/login        -> login_handler
POST /auth/refresh      -> refresh_handler
POST /auth/forgot-password -> forgot_password_handler
POST /auth/reset-password  -> reset_password_handler

// App management routes (protected)
POST /apps              -> create_app_handler
POST /apps/{app_id}/roles -> create_role_handler
POST /apps/{app_id}/permissions -> create_permission_handler
POST /apps/{app_id}/users/{user_id}/roles -> assign_role_handler
```

### 5. Middleware

```rust
// JWT Authentication middleware
pub async fn jwt_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // 1. Extract token from Authorization header
    // 2. Verify token signature (RS256)
    // 3. Check expiration
    // 4. Inject claims into request extensions
    // 5. Call next handler
}
```

## Data Models

### Database Schema

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    email_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Apps table
CREATE TABLE apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL
);

-- Roles table (scoped to app)
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    UNIQUE(app_id, name)
);

-- Permissions table (scoped to app)
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    code VARCHAR(100) NOT NULL,
    UNIQUE(app_id, code)
);

-- User-App-Role junction table
CREATE TABLE user_app_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, app_id, role_id)
);

-- Role-Permission junction table
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Refresh tokens table
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Password reset tokens table
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_roles_app_id ON roles(app_id);
CREATE INDEX idx_permissions_app_id ON permissions(app_id);
CREATE INDEX idx_user_app_roles_user_id ON user_app_roles(user_id);
CREATE INDEX idx_user_app_roles_app_id ON user_app_roles(app_id);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
```

### Request/Response DTOs

```rust
// Registration
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
}

// Login
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

// Refresh
#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// Password Reset
#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

// App
#[derive(Deserialize)]
pub struct CreateAppRequest {
    pub code: String,
    pub name: String,
}

// Role
#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
}

// Permission
#[derive(Deserialize)]
pub struct CreatePermissionRequest {
    pub code: String,
}

// Role Assignment
#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Password Storage Security

*For any* registered user, the stored password_hash SHALL NOT equal the original plain-text password.

**Validates: Requirements 1.5**

### Property 2: Email Uniqueness

*For any* two users in the system, their email addresses SHALL be different.

**Validates: Requirements 1.2**

### Property 3: Email Format Validation

*For any* string that does not match valid email format, registration SHALL be rejected.

**Validates: Requirements 1.3**

### Property 4: Valid Registration Creates User

*For any* valid email and password combination, registration SHALL create a user that can be retrieved from the database.

**Validates: Requirements 1.1**

### Property 5: Valid Login Returns Tokens

*For any* registered and active user, login with correct credentials SHALL return both access_token and refresh_token.

**Validates: Requirements 2.1**

### Property 6: Invalid Credentials Rejection

*For any* login attempt with incorrect email or password, the Auth_Server SHALL reject with authentication error.

**Validates: Requirements 2.2**

### Property 7: Inactive User Login Rejection

*For any* user with is_active = false, login SHALL be rejected regardless of correct credentials.

**Validates: Requirements 2.3**

### Property 8: JWT Token Structure Correctness

*For any* generated JWT token, it SHALL:
- Be signed with RS256 algorithm
- Contain "sub" field with user_id
- Contain "apps" object with app codes as keys
- Contain "exp" field with expiration timestamp
- Each app entry SHALL contain "roles" and "permissions" arrays

**Validates: Requirements 10.1, 10.2, 10.3, 2.4**

### Property 9: Token Expiry Duration

*For any* generated access token, the exp claim SHALL be approximately 15 minutes (900 seconds) from the iat claim.

**Validates: Requirements 2.5**

### Property 10: Valid Refresh Token Returns New Access Token

*For any* valid, non-expired refresh token, the refresh endpoint SHALL return a new access token.

**Validates: Requirements 3.1**

### Property 11: Invalid Refresh Token Rejection

*For any* expired, invalid, or malformed refresh token, the refresh endpoint SHALL reject the request.

**Validates: Requirements 3.2**

### Property 12: Refreshed Token Contains Updated Permissions

*For any* user whose roles/permissions have changed, a refreshed token SHALL contain the updated roles and permissions.

**Validates: Requirements 3.3**

### Property 13: Password Reset Round Trip

*For any* registered user, requesting password reset and then using the reset token with a new password SHALL allow login with the new password and reject login with the old password.

**Validates: Requirements 4.1, 4.3**

### Property 14: Password Reset Email Privacy

*For any* email (registered or not), the forgot-password response SHALL be identical, not revealing whether the email exists.

**Validates: Requirements 4.2**

### Property 15: Invalid Reset Token Rejection

*For any* expired, invalid, or already-used reset token, password reset SHALL be rejected.

**Validates: Requirements 4.4**

### Property 16: App Code Uniqueness

*For any* two apps in the system, their codes SHALL be different.

**Validates: Requirements 5.2**

### Property 17: Role Scoped to App

*For any* created role, it SHALL be associated with exactly one app_id.

**Validates: Requirements 6.1**

### Property 18: Role Name Uniqueness Within App

*For any* app, no two roles SHALL have the same name.

**Validates: Requirements 6.2**

### Property 19: Permission Scoped to App

*For any* created permission, it SHALL be associated with exactly one app_id.

**Validates: Requirements 7.1**

### Property 20: Permission Code Uniqueness Within App

*For any* app, no two permissions SHALL have the same code.

**Validates: Requirements 7.2**

### Property 21: User Role Assignment Creates Association

*For any* valid user, app, and role combination, assigning the role SHALL create a retrievable user_app_role record.

**Validates: Requirements 8.1**

### Property 22: Invalid Role Assignment Rejection

*For any* role assignment with non-existent user, app, or role, the assignment SHALL be rejected.

**Validates: Requirements 8.2**

### Property 23: User Multi-App Role Support

*For any* user, they SHALL be able to have different roles in different apps simultaneously.

**Validates: Requirements 8.3**

### Property 24: Role Permission Assignment Creates Association

*For any* valid role and permission within the same app, assigning the permission SHALL create a retrievable role_permission record.

**Validates: Requirements 9.1**

### Property 25: Cross-App Permission Assignment Rejection

*For any* role and permission from different apps, assigning the permission to the role SHALL be rejected.

**Validates: Requirements 9.2**

### Property 26: Valid JWT Passes Middleware

*For any* request with valid, non-expired JWT in Authorization header, the middleware SHALL pass the request and inject claims.

**Validates: Requirements 11.1**

### Property 27: Invalid JWT Rejected by Middleware

*For any* request with expired, invalid, or malformed JWT, the middleware SHALL reject the request.

**Validates: Requirements 11.2, 11.3**

### Property 28: Authorization Check Correctness

*For any* user with permission P in app A, can(claims, "A", "P") SHALL return true. For any user without permission P in app A, can(claims, "A", "P") SHALL return false.

**Validates: Requirements 12.1**

### Property 29: Cross-App Permission Isolation

*For any* user with permission P in app A but not in app B, can(claims, "B", "P") SHALL return false.

**Validates: Requirements 12.2**

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User is inactive")]
    UserInactive,
    
    #[error("Email already exists")]
    EmailAlreadyExists,
    
    #[error("Invalid email format")]
    InvalidEmailFormat,
    
    #[error("Password does not meet requirements")]
    WeakPassword,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("App not found")]
    NotFound,
    
    #[error("App code already exists")]
    CodeAlreadyExists,
    
    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RoleError {
    #[error("Role not found")]
    NotFound,
    
    #[error("Role name already exists in this app")]
    NameAlreadyExists,
    
    #[error("App not found")]
    AppNotFound,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Permission not found")]
    NotFound,
    
    #[error("Permission code already exists in this app")]
    CodeAlreadyExists,
    
    #[error("App not found")]
    AppNotFound,
    
    #[error("Cross-app permission assignment not allowed")]
    CrossAppAssignment,
    
    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}
```

### HTTP Error Responses

```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

// Error mapping to HTTP status codes
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            AuthError::UserInactive => (StatusCode::FORBIDDEN, "user_inactive"),
            AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, "email_exists"),
            AuthError::InvalidEmailFormat => (StatusCode::BAD_REQUEST, "invalid_email"),
            AuthError::WeakPassword => (StatusCode::BAD_REQUEST, "weak_password"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "token_expired"),
            AuthError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        
        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });
        
        (status, body).into_response()
    }
}
```

## Testing Strategy

### Dual Testing Approach

Auth Server sử dụng kết hợp Unit Tests và Property-Based Tests:

1. **Unit Tests**: Kiểm tra các trường hợp cụ thể, edge cases, và error conditions
2. **Property-Based Tests**: Kiểm tra các thuộc tính phải đúng với mọi input hợp lệ

### Property-Based Testing Framework

- **Library**: `proptest` crate cho Rust
- **Minimum iterations**: 100 per property test
- **Tag format**: `// Feature: auth-server, Property N: [property_text]`

### Test Categories

#### 1. Authentication Tests
- Registration with valid/invalid inputs
- Login with correct/incorrect credentials
- Token refresh flow
- Password reset flow

#### 2. Authorization Tests
- Role creation and assignment
- Permission creation and assignment
- Cross-app isolation
- can() function correctness

#### 3. JWT Tests
- Token structure validation
- Signature verification (RS256)
- Expiry handling
- Claims injection

#### 4. Database Constraint Tests
- Email uniqueness
- App code uniqueness
- Role name uniqueness within app
- Permission code uniqueness within app

### Test Configuration

```rust
// proptest configuration
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn test_property_name(input in strategy()) {
        // Property assertion
    }
}
```

### Integration Test Setup

```rust
// Test database setup
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost/auth_test".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}
```

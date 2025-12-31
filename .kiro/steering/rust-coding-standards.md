# Auth Server - Rust Coding Standards

Bộ quy tắc coding cho dự án auth-server. Tất cả contributors phải tuân theo các quy tắc này.

## 1. Cấu Trúc Project

```
src/
├── main.rs           # Entry point, router configuration
├── config.rs         # Configuration và AppState
├── error.rs          # Error types với IntoResponse
├── dto/              # Data Transfer Objects (request/response)
├── models/           # Domain models và database row types
├── repositories/     # Database operations (CRUD)
├── services/         # Business logic
├── handlers/         # HTTP request handlers
├── middleware/       # Axum middleware (auth, etc.)
└── utils/            # Utilities (jwt, password, email, etc.)
```

### Nguyên tắc phân tách:
- **DTO**: Chỉ chứa struct cho request/response, derive `Serialize`/`Deserialize`
- **Model**: Domain entities, có thể có `FromRow` cho database mapping
- **Repository**: Chỉ database operations, không business logic
- **Service**: Business logic, orchestrate repositories
- **Handler**: HTTP layer, parse request → call service → format response

## 2. Error Handling

### 2.1 Định nghĩa Error Types

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}
```

### 2.2 Implement IntoResponse

```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,      // error code (snake_case)
    pub message: String,    // human-readable message
    pub status_code: u16,
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            DomainError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            DomainError::ValidationError(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            DomainError::InternalError(ref e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
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

### 2.3 Quy tắc Error:
- Mỗi domain có error type riêng: `AuthError`, `AppError`, `RoleError`, etc.
- Dùng `thiserror` cho derive `Error`
- Dùng `anyhow::Error` cho internal errors
- KHÔNG expose internal details trong error message cho client
- Log internal errors với `tracing::error!`

## 3. DTO (Data Transfer Objects)

### 3.1 Request DTOs

```rust
use serde::Deserialize;

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    /// Optional app_id to check ban status
    pub app_id: Option<uuid::Uuid>,
}
```

### 3.2 Response DTOs

```rust
use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// User profile response
/// KHÔNG bao gồm sensitive fields như password_hash
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
```

### 3.3 Quy tắc DTO:
- Request: `#[derive(Debug, Deserialize)]`
- Response: `#[derive(Debug, Serialize)]`
- KHÔNG serialize sensitive data (password_hash, secrets, etc.)
- Dùng `Option<T>` cho optional fields
- Document với `///` comments

## 4. Models

### 4.1 Domain Model

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

### 4.2 Database Row Type (cho MySQL)

```rust
use sqlx::FromRow;

/// Row type for MySQL query results
/// MySQL trả về UUID dạng String, cần convert
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,  // MySQL VARCHAR(36)
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            email: row.email,
            password_hash: row.password_hash,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
```

## 5. Repository Pattern

### 5.1 Cấu trúc Repository

```rust
use sqlx::MySqlPool;
use uuid::Uuid;
use crate::error::AuthError;
use crate::models::User;

/// Repository for user database operations
#[derive(Clone)]
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    /// Create a new repository with database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(&self, email: &str, password_hash: &str) -> Result<User, AuthError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(email)
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // Handle duplicate entry error
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("Duplicate entry") {
                    return AuthError::EmailAlreadyExists;
                }
            }
            AuthError::InternalError(e.into())
        })?;

        self.find_by_id(id).await?.ok_or(AuthError::InternalError(
            anyhow::anyhow!("Failed to fetch created user")
        ))
    }
}
```

### 5.2 Quy tắc Repository:
- Mỗi entity có repository riêng
- Repository chỉ làm CRUD, không business logic
- Dùng `sqlx::query_as` cho SELECT với mapping
- Dùng `sqlx::query` cho INSERT/UPDATE/DELETE
- UUID bind dạng `.bind(id.to_string())` cho MySQL
- Handle database errors và convert sang domain errors

## 6. Service Layer

### 6.1 Cấu trúc Service

```rust
use sqlx::MySqlPool;
use crate::error::AuthError;
use crate::models::User;
use crate::repositories::UserRepository;
use crate::utils::jwt::JwtManager;
use crate::utils::password::{hash_password, verify_password};

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    pool: MySqlPool,
    user_repo: UserRepository,
    jwt_manager: JwtManager,
}

impl AuthService {
    pub fn new(pool: MySqlPool, jwt_manager: JwtManager) -> Self {
        let user_repo = UserRepository::new(pool.clone());
        Self {
            pool,
            user_repo,
            jwt_manager,
        }
    }

    /// Register a new user
    pub async fn register(&self, email: &str, password: &str) -> Result<User, AuthError> {
        // Validate email format
        self.validate_email(email)?;
        
        // Validate password strength
        self.validate_password(password)?;
        
        // Hash password
        let password_hash = hash_password(password)?;
        
        // Create user
        let user = self.user_repo.create(email, &password_hash).await?;
        
        Ok(user)
    }

    /// Login user
    pub async fn login(&self, email: &str, password: &str) -> Result<TokenPair, AuthError> {
        // Find user
        let user = self.user_repo
            .find_by_email(email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;
        
        // Verify password
        if !verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Check if active
        if !user.is_active {
            return Err(AuthError::UserInactive);
        }
        
        // Generate tokens
        let tokens = self.jwt_manager.create_token_pair(user.id)?;
        
        Ok(tokens)
    }
}
```

### 6.2 Quy tắc Service:
- Service chứa business logic
- Inject dependencies qua constructor
- Orchestrate multiple repositories nếu cần
- Validation logic nằm trong service
- KHÔNG access database trực tiếp, dùng repository

## 7. Handlers (HTTP Layer)

### 7.1 Cấu trúc Handler

```rust
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{CreateUserRequest, UserResponse, PaginationQuery};
use crate::error::AuthError;
use crate::services::UserService;
use crate::utils::jwt::Claims;

/// POST /users - Create a new user
///
/// # Requirements
/// - 1.1: Validate email format
/// - 1.2: Ensure email uniqueness
pub async fn create_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,  // From auth middleware
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AuthError> {
    let service = UserService::new(state.pool.clone());
    
    let user = service.create(&req.email, &req.password).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: user.id,
            email: user.email,
        }),
    ))
}

/// GET /users/:id - Get user by ID
pub async fn get_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, AuthError> {
    let service = UserService::new(state.pool.clone());
    
    let user = service.get_by_id(user_id).await?;
    
    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
    }))
}

/// GET /users - List users with pagination
pub async fn list_users_handler(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<UserResponse>>, AuthError> {
    let service = UserService::new(state.pool.clone());
    
    let (users, total) = service.list(pagination.page, pagination.limit).await?;
    
    Ok(Json(PaginatedResponse {
        data: users.into_iter().map(UserResponse::from).collect(),
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}
```

### 7.2 Quy tắc Handler:
- Handler chỉ làm: parse request → call service → format response
- Dùng `State(state)` để access AppState
- Dùng `Extension(claims)` để access JWT claims từ middleware
- Dùng `Path(id)` cho path parameters
- Dùng `Query(params)` cho query parameters
- Dùng `Json(body)` cho request body
- Return `Result<(StatusCode, Json<T>), Error>` cho POST/PUT
- Return `Result<Json<T>, Error>` cho GET
- Document với `///` comments và `# Requirements`

## 8. Middleware

### 8.1 JWT Auth Middleware

```rust
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::config::AppState;
use crate::error::AuthError;

pub async fn jwt_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::InvalidToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)?;

    // Verify token
    let claims = state.jwt_manager.verify_token(token)?;

    // Insert claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
```

## 9. Router Configuration

```rust
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn create_router(state: AppState) -> Router {
    // Public routes - no auth required
    let public_routes = Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler));

    // Protected routes - JWT required
    let protected_routes = Router::new()
        .route("/users/me", get(get_profile_handler))
        .route("/users/me", put(update_profile_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // Admin routes - JWT + admin check required
    let admin_routes = Router::new()
        .route("/admin/users", get(list_all_users_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .nest("/admin", admin_routes)
        .with_state(state)
}
```

## 10. Testing

### 10.1 Property-Based Testing với Proptest

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    /// Generate valid email addresses
    fn email_strategy() -> impl Strategy<Value = String> {
        ("[a-z]{3,10}", "[a-z]{3,8}")
            .prop_map(|(local, domain)| format!("test_{}@{}.com", local, domain))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Email uniqueness is enforced
        #[test]
        fn prop_email_uniqueness(
            email in email_strategy(),
            password1 in "[a-zA-Z0-9]{10,20}",
            password2 in "[a-zA-Z0-9]{10,20}"
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = setup_test_db().await;
                let repo = UserRepository::new(pool.clone());

                // First creation should succeed
                let result1 = repo.create(&email, &password1).await;
                prop_assert!(result1.is_ok());

                // Second creation with same email should fail
                let result2 = repo.create(&email, &password2).await;
                prop_assert!(matches!(result2, Err(AuthError::EmailAlreadyExists)));

                cleanup_test_data(&pool, &[email]).await;
                Ok(())
            })?;
        }
    }
}
```

## 11. Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Files | snake_case | `user_management.rs` |
| Structs | PascalCase | `UserRepository` |
| Functions | snake_case | `find_by_email` |
| Constants | SCREAMING_SNAKE_CASE | `MIN_PASSWORD_LENGTH` |
| Handlers | snake_case + `_handler` | `create_user_handler` |
| Error types | PascalCase + `Error` | `AuthError` |
| DTOs | PascalCase + `Request`/`Response` | `LoginRequest` |

## 12. Documentation

### 12.1 Function Documentation

```rust
/// Register a new user with email and password
///
/// # Arguments
/// * `email` - User's email address (must be valid format)
/// * `password` - User's password (min 8 chars, mixed case + digit)
///
/// # Returns
/// * `Ok(User)` - Created user
/// * `Err(AuthError::EmailAlreadyExists)` - Email taken
/// * `Err(AuthError::WeakPassword)` - Password too weak
///
/// # Requirements
/// - 1.1: Validate email format
/// - 1.2: Ensure email uniqueness
/// - 1.3: Hash password with argon2
pub async fn register(&self, email: &str, password: &str) -> Result<User, AuthError> {
    // ...
}
```

### 12.2 Handler Documentation

```rust
/// POST /auth/register - Register a new user
///
/// # Requirements
/// - 14.1: Expose POST /auth/register endpoint
/// - 1.1-1.5: User registration requirements
///
/// # Security
/// - Rate limited: 10 requests per minute per IP
/// - Password hashed with argon2
pub async fn register_handler(...) -> Result<...> {
    // ...
}
```

## 13. Security Best Practices

1. **Password Hashing**: Luôn dùng argon2 với salt
2. **Token Storage**: Hash refresh tokens trước khi lưu DB
3. **Error Messages**: KHÔNG leak thông tin sensitive
4. **Rate Limiting**: Apply cho login, register, password reset
5. **Input Validation**: Validate tất cả input từ client
6. **SQL Injection**: Dùng parameterized queries (sqlx tự handle)
7. **Audit Logging**: Log tất cả security events

## 14. Dependencies Chuẩn

```toml
[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace", "timeout"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "mysql", "uuid", "chrono"] }

# Security
argon2 = "0.5"
jsonwebtoken = "9"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"
anyhow = "1"

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```


## 15. Quy Tắc Nâng Cấp Code (Upgrade/Refactor Guidelines)

### 15.1 Nguyên Tắc Chung

1. **Backward Compatibility**: Giữ API compatibility khi có thể
2. **Incremental Changes**: Thay đổi từng bước nhỏ, không refactor lớn một lần
3. **Test First**: Đảm bảo có tests trước khi refactor
4. **Document Breaking Changes**: Ghi rõ những thay đổi breaking

### 15.2 Thêm Field Mới vào Struct

#### Model/Entity
```rust
// TRƯỚC
pub struct User {
    pub id: Uuid,
    pub email: String,
}

// SAU - thêm field mới với Option hoặc default
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,  // Option cho backward compatibility
    #[serde(default)]
    pub is_verified: bool,      // hoặc default value
}
```

#### Database Migration
```sql
-- migrations/YYYYMMDD_add_phone_to_users.sql
ALTER TABLE users ADD COLUMN phone VARCHAR(20) NULL;
ALTER TABLE users ADD COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE;
```

#### Repository - Cập nhật tất cả SELECT queries
```rust
// Cập nhật TẤT CẢ queries có SELECT để include field mới
pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, phone, is_verified, created_at  -- Thêm fields mới
        FROM users WHERE id = ?
        "#,
    )
    // ...
}
```

### 15.3 Thêm Endpoint Mới

#### Checklist:
1. [ ] Tạo DTO trong `dto/`
2. [ ] Tạo/update Service method trong `services/`
3. [ ] Tạo Handler trong `handlers/`
4. [ ] Đăng ký route trong `main.rs`
5. [ ] Thêm middleware nếu cần (auth, rate limit)
6. [ ] Update OpenAPI spec (`openapi.yaml`)
7. [ ] Update SDK nếu có (`sdk/`)

#### Ví dụ thêm endpoint GET /users/:id/settings
```rust
// 1. dto/user.rs
#[derive(Debug, Serialize)]
pub struct UserSettingsResponse {
    pub user_id: Uuid,
    pub theme: String,
    pub notifications_enabled: bool,
}

// 2. services/user_profile.rs
impl UserProfileService {
    pub async fn get_settings(&self, user_id: Uuid) -> Result<UserSettings, AuthError> {
        // business logic
    }
}

// 3. handlers/user_profile.rs
pub async fn get_user_settings_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserSettingsResponse>, AuthError> {
    // handler logic
}

// 4. main.rs - thêm route
let protected_user_routes = Router::new()
    .route("/users/:user_id/settings", get(get_user_settings_handler))
    // ...
```

### 15.4 Thay Đổi Response Format

#### KHÔNG breaking change (thêm fields)
```rust
// TRƯỚC
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
}

// SAU - OK, thêm fields không break clients
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,      // Thêm optional field - OK
    pub created_at: DateTime<Utc>, // Thêm field mới - OK
}
```

#### Breaking change (đổi tên, xóa field, đổi type)
```rust
// Cần versioning hoặc deprecation period

// Option 1: API Versioning
// GET /v1/users/:id -> UserResponseV1
// GET /v2/users/:id -> UserResponseV2

// Option 2: Deprecation với warning header
pub async fn get_user_handler(...) -> Response {
    let response = Json(user_response);
    let mut response = response.into_response();
    response.headers_mut().insert(
        "X-Deprecation-Warning",
        "Field 'username' deprecated, use 'email' instead".parse().unwrap()
    );
    response
}
```

### 15.5 Refactor Service Layer

#### Tách service lớn thành nhiều services nhỏ
```rust
// TRƯỚC - AuthService quá lớn
impl AuthService {
    pub async fn register(...) { }
    pub async fn login(...) { }
    pub async fn setup_mfa(...) { }
    pub async fn verify_mfa(...) { }
    pub async fn create_session(...) { }
}

// SAU - Tách thành các services chuyên biệt
impl AuthService {
    pub async fn register(...) { }
    pub async fn login(...) { }
}

impl MfaService {
    pub async fn setup(...) { }
    pub async fn verify(...) { }
}

impl SessionService {
    pub async fn create(...) { }
    pub async fn revoke(...) { }
}
```

#### Quy tắc tách service:
- Mỗi service < 500 lines
- Single Responsibility: 1 service = 1 domain concern
- Inject services khác qua constructor nếu cần orchestrate

### 15.6 Thêm Error Type Mới

```rust
// 1. Thêm variant vào enum
#[derive(Debug, Error)]
pub enum AuthError {
    // ... existing variants
    
    #[error("Phone number already exists")]
    PhoneAlreadyExists,  // Thêm variant mới
    
    #[error("Invalid phone format")]
    InvalidPhoneFormat,
}

// 2. Update IntoResponse
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            // ... existing matches
            AuthError::PhoneAlreadyExists => (StatusCode::CONFLICT, "phone_exists"),
            AuthError::InvalidPhoneFormat => (StatusCode::BAD_REQUEST, "invalid_phone"),
        };
        // ...
    }
}
```

### 15.7 Database Schema Changes

#### Quy tắc Migration:
1. **Tên file**: `YYYYMMDDHHMMSS_description.sql`
2. **Không sửa migration cũ**: Tạo migration mới để fix
3. **Rollback plan**: Comment cách rollback trong migration
4. **Test trên staging trước**

```sql
-- migrations/20241230120000_add_user_preferences.sql

-- Forward migration
CREATE TABLE user_preferences (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    theme VARCHAR(20) DEFAULT 'light',
    language VARCHAR(10) DEFAULT 'en',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);

-- Rollback (comment for reference):
-- DROP TABLE IF EXISTS user_preferences;
```

#### Dangerous Operations - Cần review kỹ:
```sql
-- ⚠️ DROP COLUMN - data loss
ALTER TABLE users DROP COLUMN old_field;

-- ⚠️ RENAME COLUMN - có thể break queries
ALTER TABLE users RENAME COLUMN old_name TO new_name;

-- ⚠️ CHANGE TYPE - có thể fail với data hiện tại
ALTER TABLE users MODIFY COLUMN status ENUM('active', 'inactive', 'banned');

-- ✅ Safe: ADD COLUMN với NULL hoặc DEFAULT
ALTER TABLE users ADD COLUMN new_field VARCHAR(100) NULL;
```

### 15.8 Dependency Upgrade

#### Checklist khi upgrade dependency:
1. [ ] Đọc CHANGELOG của dependency
2. [ ] Check breaking changes
3. [ ] Update code nếu API thay đổi
4. [ ] Run full test suite
5. [ ] Test manually các critical flows

#### Ví dụ upgrade axum 0.6 → 0.7
```rust
// TRƯỚC (axum 0.6)
use axum::extract::TypedHeader;
use headers::Authorization;

pub async fn handler(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) { }

// SAU (axum 0.7) - TypedHeader moved to axum-extra
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};

pub async fn handler(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) { }
```

### 15.9 Performance Optimization

#### Thêm Index
```sql
-- Analyze slow queries trước
EXPLAIN SELECT * FROM users WHERE email = 'test@example.com';

-- Thêm index
CREATE INDEX idx_users_email ON users(email);

-- Composite index cho queries với multiple conditions
CREATE INDEX idx_user_apps_user_status ON user_apps(user_id, status);
```

#### Optimize N+1 Queries
```rust
// TRƯỚC - N+1 problem
for user in users {
    let roles = repo.get_roles_for_user(user.id).await?;  // N queries
}

// SAU - Single query với JOIN
let users_with_roles = sqlx::query_as::<_, UserWithRoles>(
    r#"
    SELECT u.*, GROUP_CONCAT(r.name) as roles
    FROM users u
    LEFT JOIN user_roles ur ON u.id = ur.user_id
    LEFT JOIN roles r ON ur.role_id = r.id
    GROUP BY u.id
    "#
).fetch_all(&pool).await?;
```

### 15.10 Deprecation Process

#### Bước 1: Mark as deprecated
```rust
/// Get user by username
/// 
/// # Deprecated
/// Use `find_by_email` instead. Will be removed in v2.0.
#[deprecated(since = "1.5.0", note = "Use find_by_email instead")]
pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
    // ...
}
```

#### Bước 2: Log warning khi sử dụng
```rust
pub async fn old_endpoint_handler(...) -> Result<Json<Response>, Error> {
    tracing::warn!("Deprecated endpoint called: GET /api/v1/old-endpoint");
    // ...
}
```

#### Bước 3: Thêm deprecation header
```rust
let mut response = Json(data).into_response();
response.headers_mut().insert(
    "Deprecation",
    "true".parse().unwrap()
);
response.headers_mut().insert(
    "Sunset", 
    "Sat, 01 Jun 2025 00:00:00 GMT".parse().unwrap()
);
```

#### Bước 4: Remove sau deprecation period (thường 2-3 releases)

### 15.11 Checklist Trước Khi Merge

- [ ] Code follows coding standards
- [ ] All tests pass (`cargo test`)
- [ ] No new warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Migration tested on staging DB
- [ ] OpenAPI spec updated (nếu có API changes)
- [ ] SDK updated (nếu có API changes)
- [ ] CHANGELOG updated
- [ ] Breaking changes documented
- [ ] Reviewed by at least 1 team member

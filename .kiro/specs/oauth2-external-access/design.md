# Design Document: OAuth2 External Access

## Overview

Thiết kế hệ thống OAuth2/OpenID Connect cho phép app bên ngoài (third-party) truy cập dữ liệu người dùng một cách an toàn. Hệ thống mở rộng từ Auth Server hiện có, thêm các thành phần:

- OAuth Client management (đăng ký app bên ngoài)
- Scope-based authorization
- User consent flow
- Authorization Code Flow với PKCE (cho External Apps)
- Client Credentials Flow (cho Internal Apps)
- Token management với refresh rotation
- Revocation mechanism

## Architecture

```
┌─────────────────┐     ┌──────────────────────────────────────────┐
│   External App  │     │           Authorization Server           │
│    (Client)     │     │                                          │
└────────┬────────┘     │  ┌─────────────┐  ┌──────────────────┐  │
         │              │  │   OAuth     │  │    Consent       │  │
         │ 1. Authorize │  │  Endpoints  │  │    Service       │  │
         ├─────────────►│  │             │  │                  │  │
         │              │  │ /authorize  │  │ - Show consent   │  │
         │              │  │ /token      │  │ - Store consent  │  │
         │              │  │ /revoke     │  │ - Check consent  │  │
         │              │  │ /userinfo   │  │                  │  │
         │              │  └──────┬──────┘  └────────┬─────────┘  │
         │              │         │                   │            │
         │              │  ┌──────▼───────────────────▼─────────┐  │
         │              │  │         OAuth Service              │  │
         │              │  │                                    │  │
         │              │  │  - Validate authorization request  │  │
         │              │  │  - Generate authorization code     │  │
         │              │  │  - Exchange code for tokens        │  │
         │              │  │  - Validate PKCE                   │  │
         │              │  │  - Issue scoped tokens             │  │
         │              │  └──────┬─────────────────────────────┘  │
         │              │         │                                │
         │              │  ┌──────▼─────────────────────────────┐  │
         │              │  │         OAuth Repository           │  │
         │              │  │                                    │  │
         │              │  │  - oauth_clients                   │  │
         │              │  │  - oauth_scopes                    │  │
         │              │  │  - user_consents                   │  │
         │              │  │  - oauth_authorization_codes       │  │
         │              │  │  - oauth_tokens                    │  │
         │              │  └────────────────────────────────────┘  │
         │              │                                          │
         │              └──────────────────────────────────────────┘
         │
         │ 5. Access API
         ▼
┌─────────────────┐
│ Resource Server │
│                 │
│ - Validate token│
│ - Check scope   │
│ - Return data   │
└─────────────────┘
```

### OAuth2 Authorization Code Flow with PKCE

```
┌──────┐          ┌──────────────┐          ┌─────────────────────┐
│ User │          │ External App │          │ Authorization Server│
└──┬───┘          └──────┬───────┘          └──────────┬──────────┘
   │                     │                             │
   │  1. Click Login     │                             │
   │────────────────────►│                             │
   │                     │                             │
   │                     │ 2. Generate code_verifier   │
   │                     │    & code_challenge         │
   │                     │                             │
   │  3. Redirect to /oauth/authorize                  │
   │◄────────────────────┤                             │
   │                     │                             │
   │  4. GET /oauth/authorize?client_id=...            │
   │       &redirect_uri=...&scope=...                 │
   │       &code_challenge=...&code_challenge_method=S256
   │───────────────────────────────────────────────────►
   │                     │                             │
   │  5. Login (if not authenticated)                  │
   │◄──────────────────────────────────────────────────┤
   │                     │                             │
   │  6. Consent Screen (show requested scopes)        │
   │◄──────────────────────────────────────────────────┤
   │                     │                             │
   │  7. User approves   │                             │
   │───────────────────────────────────────────────────►
   │                     │                             │
   │  8. Redirect to redirect_uri?code=AUTH_CODE       │
   │◄──────────────────────────────────────────────────┤
   │                     │                             │
   │  9. Follow redirect │                             │
   │────────────────────►│                             │
   │                     │                             │
   │                     │ 10. POST /oauth/token       │
   │                     │     code=AUTH_CODE          │
   │                     │     code_verifier=...       │
   │                     │─────────────────────────────►
   │                     │                             │
   │                     │ 11. Validate code_verifier  │
   │                     │     against code_challenge  │
   │                     │                             │
   │                     │ 12. Return tokens           │
   │                     │◄─────────────────────────────
   │                     │                             │
   │  13. Access granted │                             │
   │◄────────────────────┤                             │
```

## Components and Interfaces

### 1. OAuth Client Model

```rust
/// OAuth Client - represents an external or internal application
pub struct OAuthClient {
    pub id: Uuid,
    pub client_id: String,           // Public identifier
    pub client_secret_hash: String,  // Hashed secret
    pub name: String,
    pub redirect_uris: Vec<String>,  // Allowed redirect URIs
    pub is_internal: bool,           // Internal vs External app
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
```

### 2. OAuth Scope Model

```rust
/// OAuth Scope - defines a permission scope
pub struct OAuthScope {
    pub id: Uuid,
    pub code: String,        // e.g., "profile.read", "email.read"
    pub description: String, // Human-readable description
    pub is_active: bool,
}
```

### 3. User Consent Model

```rust
/// User Consent - records user's consent for a client
pub struct UserConsent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scopes: Vec<String>,
    pub granted_at: DateTime<Utc>,
}
```

### 4. Authorization Code Model

```rust
/// Authorization Code - temporary code for token exchange
pub struct AuthorizationCode {
    pub id: Uuid,
    pub code_hash: String,
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub code_challenge: String,
    pub code_challenge_method: String, // "S256"
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}
```

### 5. OAuth Token Model

```rust
/// OAuth Token - stores issued tokens
pub struct OAuthToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}
```

### 6. OAuth Service Interface

```rust
pub trait OAuthService {
    /// Validate authorization request parameters
    async fn validate_authorization_request(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scopes: &[String],
        code_challenge: Option<&str>,
    ) -> Result<OAuthClient, OAuthError>;

    /// Generate authorization code after user consent
    async fn create_authorization_code(
        &self,
        client_id: Uuid,
        user_id: Uuid,
        redirect_uri: &str,
        scopes: &[String],
        code_challenge: &str,
    ) -> Result<String, OAuthError>;

    /// Exchange authorization code for tokens
    async fn exchange_code_for_tokens(
        &self,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Result<OAuthTokenResponse, OAuthError>;

    /// Client credentials flow (internal apps)
    async fn client_credentials_grant(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> Result<OAuthTokenResponse, OAuthError>;

    /// Refresh access token
    async fn refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<OAuthTokenResponse, OAuthError>;

    /// Revoke token
    async fn revoke_token(
        &self,
        token: &str,
        client_id: &str,
    ) -> Result<(), OAuthError>;
}
```

### 7. Consent Service Interface

```rust
pub trait ConsentService {
    /// Check if user has already consented to all requested scopes
    async fn has_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<bool, OAuthError>;

    /// Store user consent
    async fn grant_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<(), OAuthError>;

    /// Revoke user consent for a client
    async fn revoke_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<(), OAuthError>;

    /// List all consents for a user
    async fn list_user_consents(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ConsentInfo>, OAuthError>;
}
```

### 8. OAuth Endpoints

```rust
// Authorization endpoint
GET /oauth/authorize
    ?client_id=xxx
    &response_type=code
    &redirect_uri=https://app/callback
    &scope=profile.read email.read
    &code_challenge=xxx
    &code_challenge_method=S256
    &state=xxx

// Token endpoint
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=xxx
&redirect_uri=https://app/callback
&client_id=xxx
&code_verifier=xxx

// Or for client credentials
grant_type=client_credentials
&client_id=xxx
&client_secret=xxx
&scope=service.read

// Refresh token
grant_type=refresh_token
&refresh_token=xxx
&client_id=xxx

// Revoke endpoint
POST /oauth/revoke
Content-Type: application/x-www-form-urlencoded

token=xxx
&client_id=xxx

// UserInfo endpoint
GET /oauth/userinfo
Authorization: Bearer ACCESS_TOKEN

// Discovery endpoint
GET /.well-known/openid-configuration
```

## Data Models

### Database Schema

```sql
-- OAuth Clients table
CREATE TABLE oauth_clients (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    client_id VARCHAR(64) UNIQUE NOT NULL,
    client_secret_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    redirect_uris JSON NOT NULL,
    is_internal BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- OAuth Scopes table
CREATE TABLE oauth_scopes (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    code VARCHAR(100) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true
);

-- User Consents table
CREATE TABLE user_consents (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    client_id CHAR(36) NOT NULL,
    scopes JSON NOT NULL,
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_user_client (user_id, client_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE
);

-- Authorization Codes table
CREATE TABLE oauth_authorization_codes (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    code_hash VARCHAR(255) NOT NULL,
    client_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    redirect_uri VARCHAR(2048) NOT NULL,
    scopes JSON NOT NULL,
    code_challenge VARCHAR(128) NOT NULL,
    code_challenge_method VARCHAR(10) NOT NULL DEFAULT 'S256',
    expires_at TIMESTAMP NOT NULL,
    used BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- OAuth Tokens table
CREATE TABLE oauth_tokens (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36),
    client_id CHAR(36) NOT NULL,
    access_token_hash VARCHAR(255) NOT NULL,
    refresh_token_hash VARCHAR(255),
    scopes JSON NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE
);

-- OAuth Audit Log table
CREATE TABLE oauth_audit_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    event_type VARCHAR(50) NOT NULL,
    client_id CHAR(36),
    user_id CHAR(36),
    ip_address VARCHAR(45),
    details JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_oauth_clients_client_id ON oauth_clients(client_id);
CREATE INDEX idx_user_consents_user_id ON user_consents(user_id);
CREATE INDEX idx_oauth_codes_code_hash ON oauth_authorization_codes(code_hash);
CREATE INDEX idx_oauth_tokens_access_hash ON oauth_tokens(access_token_hash);
CREATE INDEX idx_oauth_tokens_refresh_hash ON oauth_tokens(refresh_token_hash);
CREATE INDEX idx_oauth_audit_user_id ON oauth_audit_logs(user_id);
CREATE INDEX idx_oauth_audit_client_id ON oauth_audit_logs(client_id);
```

### JWT Access Token Claims (OAuth2)

```rust
/// OAuth2 Access Token Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Claims {
    /// Subject - user_id
    pub sub: String,
    /// Audience - client_id
    pub aud: String,
    /// Scopes granted
    pub scope: Vec<String>,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// Token type
    pub token_type: String, // "oauth2"
}
```

## PKCE Implementation

```rust
/// PKCE utilities
pub mod pkce {
    use sha2::{Sha256, Digest};
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    /// Verify code_verifier against code_challenge
    pub fn verify_pkce(
        code_verifier: &str,
        code_challenge: &str,
        method: &str,
    ) -> bool {
        match method {
            "S256" => {
                let mut hasher = Sha256::new();
                hasher.update(code_verifier.as_bytes());
                let hash = hasher.finalize();
                let computed_challenge = URL_SAFE_NO_PAD.encode(hash);
                computed_challenge == code_challenge
            }
            "plain" => code_verifier == code_challenge,
            _ => false,
        }
    }

    /// Validate code_verifier format (43-128 chars, URL-safe)
    pub fn validate_code_verifier(verifier: &str) -> bool {
        let len = verifier.len();
        len >= 43 && len <= 128 && 
        verifier.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~')
    }
}
```

## Token Response Format

```rust
/// OAuth2 Token Response
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub token_type: String,  // "Bearer"
    pub expires_in: i64,     // seconds
    pub scope: String,       // space-separated scopes
}
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property 1: Client Registration Data Integrity
*For any* valid OAuth client registration request, storing and then retrieving the client SHALL return all original fields (client_id, name, redirect_uris, is_internal) unchanged.
**Validates: Requirements 1.1**

### Property 2: Client ID Uniqueness
*For any* set of registered OAuth clients, all client_ids SHALL be unique.
**Validates: Requirements 1.2**

### Property 3: Client Secret Hashing
*For any* OAuth client with a secret, the stored secret_hash SHALL NOT equal the original plaintext secret, AND verifying the original secret against the hash SHALL succeed.
**Validates: Requirements 1.3**

### Property 4: External App HTTPS Redirect URI Validation
*For any* External_App registration, all redirect_uris SHALL use HTTPS protocol. HTTP URIs SHALL be rejected.
**Validates: Requirements 1.4**

### Property 5: Scope Code Uniqueness
*For any* set of OAuth scopes, attempting to create a scope with a duplicate code SHALL be rejected.
**Validates: Requirements 2.2**

### Property 6: Invalid Scope Rejection
*For any* token request with scopes, if any requested scope does not exist in the system, the request SHALL be rejected.
**Validates: Requirements 2.4**

### Property 7: Authorization Request Parameter Validation
*For any* External_App authorization request, if any required parameter (response_type, client_id, redirect_uri, scope, code_challenge) is missing, the request SHALL be rejected.
**Validates: Requirements 3.1, 3.2, 10.2**

### Property 8: Redirect URI Exact Match
*For any* authorization request, the redirect_uri SHALL exactly match one of the registered redirect_uris. Partial matches or substrings SHALL be rejected.
**Validates: Requirements 3.3, 10.5**

### Property 9: Authorization Code Expiration
*For any* generated authorization code, the expiration time SHALL be at most 10 minutes from creation time.
**Validates: Requirements 3.4**

### Property 10: PKCE Verification Round-Trip
*For any* valid code_verifier, computing SHA256 and base64url encoding SHALL produce the original code_challenge. Verification with incorrect code_verifier SHALL fail.
**Validates: Requirements 3.5**

### Property 11: New Scope Consent Requirement
*For any* External_App authorization request with scopes not previously consented, the system SHALL require explicit user consent before proceeding.
**Validates: Requirements 4.2**

### Property 12: Consent Record Integrity
*For any* granted consent, the stored record SHALL contain the correct user_id, client_id, all granted scopes, and a valid timestamp.
**Validates: Requirements 4.3**

### Property 13: Existing Consent Skip
*For any* External_App authorization request where user has previously consented to all requested scopes, the consent check SHALL return true (allowing skip).
**Validates: Requirements 4.5**

### Property 14: Internal App Consent Skip
*For any* Internal_App, the system SHALL NOT require user consent regardless of requested scopes.
**Validates: Requirements 4.6, 6.3**

### Property 15: Token Exchange Returns Both Tokens
*For any* successful authorization code exchange, the response SHALL contain both access_token and refresh_token.
**Validates: Requirements 5.1**

### Property 16: Access Token Short Expiration
*For any* issued access token, the expiration time SHALL be at most 15 minutes (900 seconds) from issuance.
**Validates: Requirements 5.2, 10.3**

### Property 17: Token Response Scope Inclusion
*For any* token response, the scope field SHALL contain all scopes that were granted during authorization.
**Validates: Requirements 5.3**

### Property 18: JWT Required Claims
*For any* issued JWT access token, decoding SHALL reveal sub (user_id), aud (client_id), scope (array), and exp (expiration) claims.
**Validates: Requirements 5.4**

### Property 19: JWT RS256 Algorithm
*For any* issued JWT token, the header SHALL specify alg: "RS256".
**Validates: Requirements 5.5**

### Property 20: Token Storage Hashing
*For any* stored token, the access_token_hash and refresh_token_hash SHALL NOT equal the original plaintext tokens.
**Validates: Requirements 5.6**

### Property 21: Client Credentials Flow for Internal Apps
*For any* Internal_App with valid client_id and client_secret, the client_credentials grant SHALL succeed without PKCE or user consent.
**Validates: Requirements 6.1, 6.2**

### Property 22: Refresh Token Returns New Access Token
*For any* valid refresh_token, the refresh request SHALL return a new access_token.
**Validates: Requirements 7.1**

### Property 23: Refresh Token Rotation
*For any* successful refresh request, a new refresh_token SHALL be issued alongside the new access_token.
**Validates: Requirements 7.2, 10.4**

### Property 24: Old Refresh Token Invalidation
*For any* refresh token rotation, the old refresh_token SHALL be invalidated and subsequent use SHALL fail.
**Validates: Requirements 7.4**

### Property 25: Revoked Token Cascade
*For any* attempt to use a revoked refresh_token, the system SHALL revoke all tokens for that client-user pair.
**Validates: Requirements 7.5**

### Property 26: Token Signature and Expiration Validation
*For any* access token, validation SHALL verify the RS256 signature and check expiration. Invalid signature or expired tokens SHALL be rejected.
**Validates: Requirements 8.1**

### Property 27: Scope Enforcement
*For any* API request with a valid token, if the token lacks the required scope for the endpoint, the request SHALL return 403 Forbidden.
**Validates: Requirements 8.3**

### Property 28: Token Claims Extraction
*For any* validated token, extracting user_id and scopes SHALL return the values that were encoded during token creation.
**Validates: Requirements 8.4**

### Property 29: Connected Apps List Accuracy
*For any* user with granted consents, the connected apps list SHALL return all clients with their granted scopes and consent timestamps.
**Validates: Requirements 9.1**

### Property 30: Revocation Invalidates All Tokens
*For any* user revocation of a client, all access_tokens and refresh_tokens for that client-user pair SHALL be invalidated.
**Validates: Requirements 9.2**

### Property 31: Revocation Deletes Consent
*For any* user revocation of a client, the consent record for that client-user pair SHALL be deleted.
**Validates: Requirements 9.3**

### Property 32: Token Revocation Endpoint
*For any* valid token submitted to the revoke endpoint, that specific token SHALL be invalidated.
**Validates: Requirements 9.4**

### Property 33: Audit Log Creation
*For any* authorization, token issuance, or revocation event, an audit log entry SHALL be created with event type, client_id, user_id, and timestamp.
**Validates: Requirements 9.5, 10.6**

## Error Handling

### OAuth Error Responses

All error responses follow RFC 6749 format:

```rust
#[derive(Debug, Serialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}
```

### Error Codes

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `invalid_request` | 400 | Missing or invalid parameter |
| `invalid_client` | 401 | Client authentication failed |
| `invalid_grant` | 400 | Invalid authorization code or refresh token |
| `unauthorized_client` | 401 | Client not authorized for this grant type |
| `unsupported_grant_type` | 400 | Grant type not supported |
| `invalid_scope` | 400 | Invalid or unknown scope |
| `access_denied` | 403 | User denied consent |
| `server_error` | 500 | Internal server error |

### Error Handling Strategy

```rust
pub enum OAuthError {
    InvalidRequest(String),
    InvalidClient,
    InvalidGrant(String),
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope(String),
    AccessDenied,
    ServerError(String),
}

impl OAuthError {
    pub fn to_response(&self) -> (StatusCode, OAuthErrorResponse) {
        match self {
            OAuthError::InvalidRequest(desc) => (
                StatusCode::BAD_REQUEST,
                OAuthErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some(desc.clone()),
                    error_uri: None,
                },
            ),
            // ... other variants
        }
    }
}
```

## Testing Strategy

### Dual Testing Approach

This feature requires both unit tests and property-based tests:

1. **Unit Tests**: Verify specific examples, edge cases, and error conditions
2. **Property-Based Tests**: Verify universal properties across all valid inputs

### Property-Based Testing Configuration

- **Library**: `proptest` (already in dev-dependencies)
- **Minimum iterations**: 100 per property test
- **Tag format**: `Feature: oauth2-external-access, Property {number}: {property_text}`

### Test Categories

#### 1. PKCE Tests
- Property 10: PKCE verification round-trip
- Edge cases: invalid verifier length, invalid characters

#### 2. Token Tests
- Property 16: Access token expiration
- Property 18: JWT required claims
- Property 19: JWT RS256 algorithm
- Property 20: Token storage hashing
- Property 26: Token validation

#### 3. Consent Tests
- Property 11: New scope consent requirement
- Property 12: Consent record integrity
- Property 13: Existing consent skip
- Property 14: Internal app consent skip

#### 4. Client Tests
- Property 1: Client registration data integrity
- Property 2: Client ID uniqueness
- Property 3: Client secret hashing
- Property 4: External app HTTPS redirect URI validation

#### 5. Authorization Flow Tests
- Property 7: Authorization request parameter validation
- Property 8: Redirect URI exact match
- Property 9: Authorization code expiration
- Property 15: Token exchange returns both tokens

#### 6. Refresh Token Tests
- Property 22: Refresh token returns new access token
- Property 23: Refresh token rotation
- Property 24: Old refresh token invalidation
- Property 25: Revoked token cascade

#### 7. Revocation Tests
- Property 30: Revocation invalidates all tokens
- Property 31: Revocation deletes consent
- Property 32: Token revocation endpoint

### Integration Tests

- Full authorization code flow with PKCE
- Client credentials flow for internal apps
- Token refresh with rotation
- User consent flow
- Revocation flow

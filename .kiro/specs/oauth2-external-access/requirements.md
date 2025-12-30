# Requirements Document

## Introduction

Xây dựng Authorization System theo chuẩn OAuth2/OpenID Connect cho phép app bên ngoài (third-party) truy cập dữ liệu người dùng một cách an toàn. Hệ thống đảm bảo:
- App bên ngoài không biết mật khẩu user
- User chủ động đồng ý (consent) trước khi cấp quyền
- App chỉ được truy cập dữ liệu theo scope được cấp
- User có thể thu hồi quyền bất kỳ lúc nào

Hệ thống mở rộng từ Auth Server trung tâm hiện có.

## Glossary

- **Authorization_Server**: Auth Server trung tâm xử lý xác thực và cấp token
- **Resource_Server**: API server chứa dữ liệu user cần bảo vệ
- **OAuth_Client**: App bên ngoài đăng ký để truy cập dữ liệu user
- **Internal_App**: App nội bộ do hệ thống sở hữu, chạy trong hạ tầng nội bộ
- **External_App**: App bên thứ ba, không kiểm soát hạ tầng
- **Scope**: Phạm vi quyền truy cập cụ thể (vd: profile.read, email.read)
- **Consent**: Sự đồng ý của user cho phép app truy cập dữ liệu
- **Authorization_Code**: Mã tạm thời dùng để đổi lấy access token
- **PKCE**: Proof Key for Code Exchange - cơ chế bảo mật cho Authorization Code Flow
- **Access_Token**: Token ngắn hạn để truy cập API
- **Refresh_Token**: Token dài hạn để lấy access token mới

## Requirements

### Requirement 1: OAuth Client Registration

**User Story:** As a developer, I want to register my external application as an OAuth client, so that my app can request access to user data through the authorization system.

#### Acceptance Criteria

1. WHEN a developer registers a new OAuth client, THE Authorization_Server SHALL store client_id, client_secret, redirect_uris, and is_internal flag
2. THE Authorization_Server SHALL generate a unique client_id for each registered OAuth_Client
3. THE Authorization_Server SHALL securely hash the client_secret before storing
4. WHEN registering redirect_uris, THE Authorization_Server SHALL validate that each URI uses HTTPS protocol for External_App
5. THE Authorization_Server SHALL distinguish between Internal_App (is_internal=true) and External_App (is_internal=false)

### Requirement 2: OAuth Scope Management

**User Story:** As a system administrator, I want to define granular scopes, so that access permissions can be precisely controlled.

#### Acceptance Criteria

1. THE Authorization_Server SHALL support defining scopes with unique code and description
2. WHEN a scope is created, THE Authorization_Server SHALL enforce unique scope codes (e.g., profile.read, email.read, drive.read)
3. THE Authorization_Server SHALL reject scope codes that are overly broad or ambiguous
4. WHEN validating token requests, THE Authorization_Server SHALL verify all requested scopes exist and are valid

### Requirement 3: Authorization Code Flow with PKCE (External Apps)

**User Story:** As a user, I want to authorize external apps securely without sharing my password, so that I maintain control over my credentials.

#### Acceptance Criteria

1. WHEN an External_App initiates authorization, THE Authorization_Server SHALL require response_type=code, client_id, redirect_uri, scope, and code_challenge parameters
2. WHEN code_challenge is missing for External_App, THE Authorization_Server SHALL reject the request with an error
3. WHEN redirect_uri does not match registered URIs, THE Authorization_Server SHALL reject the request
4. WHEN authorization is successful, THE Authorization_Server SHALL generate a short-lived authorization code (max 10 minutes)
5. WHEN exchanging code for token, THE Authorization_Server SHALL verify code_verifier matches the original code_challenge
6. IF code_verifier validation fails, THEN THE Authorization_Server SHALL reject the token request

### Requirement 4: User Consent

**User Story:** As a user, I want to see exactly what permissions an app is requesting before I grant access, so that I can make informed decisions about my data.

#### Acceptance Criteria

1. WHEN an External_App requests authorization, THE Authorization_Server SHALL display a consent screen showing all requested scopes with descriptions
2. WHEN user has not previously consented to the requested scopes, THE Authorization_Server SHALL require explicit user approval
3. WHEN user grants consent, THE Authorization_Server SHALL store the consent record with user_id, client_id, scopes, and timestamp
4. WHEN user denies consent, THE Authorization_Server SHALL redirect back to the app with an access_denied error
5. WHEN user has previously consented to all requested scopes, THE Authorization_Server SHALL skip the consent screen and proceed
6. THE Authorization_Server SHALL NOT require consent for Internal_App

### Requirement 5: Token Issuance

**User Story:** As an authorized app, I want to receive access and refresh tokens, so that I can access user data within the granted scope.

#### Acceptance Criteria

1. WHEN authorization code is exchanged successfully, THE Authorization_Server SHALL issue an access_token and refresh_token
2. THE Authorization_Server SHALL set access_token expiration to a short duration (max 15 minutes)
3. THE Authorization_Server SHALL include granted scopes in the token response
4. WHEN issuing JWT access tokens, THE Authorization_Server SHALL include sub (user_id), aud (client_id), scope, and exp claims
5. THE Authorization_Server SHALL sign JWT tokens using RS256 algorithm
6. THE Authorization_Server SHALL hash tokens before storing in the database

### Requirement 6: Client Credentials Flow (Internal Apps)

**User Story:** As an internal service, I want to authenticate using client credentials, so that I can access resources without user interaction.

#### Acceptance Criteria

1. WHEN an Internal_App requests token with grant_type=client_credentials, THE Authorization_Server SHALL authenticate using client_id and client_secret
2. THE Authorization_Server SHALL NOT require PKCE for Internal_App using client credentials flow
3. THE Authorization_Server SHALL NOT require user consent for Internal_App
4. WHEN client credentials are invalid, THE Authorization_Server SHALL reject the request with invalid_client error
5. THE Authorization_Server SHALL issue service-scoped tokens for Internal_App

### Requirement 7: Token Refresh

**User Story:** As an app, I want to refresh my access token without user interaction, so that I can maintain continuous access.

#### Acceptance Criteria

1. WHEN a valid refresh_token is provided, THE Authorization_Server SHALL issue a new access_token
2. THE Authorization_Server SHALL implement refresh token rotation (issue new refresh_token on each refresh)
3. WHEN refresh_token is expired or invalid, THE Authorization_Server SHALL reject with invalid_grant error
4. THE Authorization_Server SHALL invalidate the old refresh_token after rotation
5. IF a revoked refresh_token is used, THEN THE Authorization_Server SHALL revoke all tokens for that client-user pair

### Requirement 8: Resource Server Token Validation

**User Story:** As a resource server, I want to validate access tokens and enforce scopes, so that I only allow authorized access to protected resources.

#### Acceptance Criteria

1. WHEN a request includes an access_token, THE Resource_Server SHALL verify the token signature and expiration
2. WHEN token is expired or invalid, THE Resource_Server SHALL return 401 Unauthorized
3. WHEN token lacks required scope for the endpoint, THE Resource_Server SHALL return 403 Forbidden
4. THE Resource_Server SHALL extract user_id and scopes from validated token for authorization decisions

### Requirement 9: Token and Consent Revocation

**User Story:** As a user, I want to revoke access for any connected app at any time, so that I maintain control over my data.

#### Acceptance Criteria

1. WHEN user requests to view connected apps, THE Authorization_Server SHALL return list of apps with granted scopes and consent timestamps
2. WHEN user revokes access for an app, THE Authorization_Server SHALL invalidate all tokens for that client-user pair
3. WHEN user revokes access, THE Authorization_Server SHALL delete the consent record
4. WHEN an app calls the revoke endpoint with a token, THE Authorization_Server SHALL invalidate that specific token
5. THE Authorization_Server SHALL log all revocation events for audit purposes

### Requirement 10: Security Requirements

**User Story:** As a security administrator, I want the OAuth system to follow security best practices, so that user data is protected from unauthorized access.

#### Acceptance Criteria

1. THE Authorization_Server SHALL require HTTPS for all OAuth endpoints
2. THE Authorization_Server SHALL require PKCE for all External_App authorization requests
3. THE Authorization_Server SHALL enforce short-lived access tokens (max 15 minutes)
4. THE Authorization_Server SHALL implement refresh token rotation
5. THE Authorization_Server SHALL validate redirect_uri exactly matches registered URIs (no partial matching)
6. THE Authorization_Server SHALL log all authorization events for audit
7. THE Authorization_Server SHALL rate-limit token endpoints to prevent brute force attacks

### Requirement 11: OAuth Endpoints

**User Story:** As a developer, I want standard OAuth2 endpoints, so that I can integrate using standard OAuth libraries.

#### Acceptance Criteria

1. THE Authorization_Server SHALL expose GET /oauth/authorize for authorization requests
2. THE Authorization_Server SHALL expose POST /oauth/token for token requests
3. THE Authorization_Server SHALL expose POST /oauth/revoke for token revocation
4. THE Authorization_Server SHALL expose GET /oauth/userinfo for retrieving user profile (requires valid token with profile scope)
5. THE Authorization_Server SHALL expose GET /.well-known/openid-configuration for discovery metadata

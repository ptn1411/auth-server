# Implementation Plan: OAuth2 External Access

## Overview

Triển khai hệ thống OAuth2/OpenID Connect cho phép app bên ngoài truy cập dữ liệu người dùng. Implementation sẽ mở rộng Auth Server hiện có với các thành phần OAuth2 mới.

## Tasks

- [x] 1. Database Schema và Models
  - [x] 1.1 Tạo migration cho OAuth2 tables
    - Tạo file migration với 6 tables: oauth_clients, oauth_scopes, user_consents, oauth_authorization_codes, oauth_tokens, oauth_audit_logs
    - Thêm indexes cho performance
    - _Requirements: 1.1, 2.1, 4.3, 5.6_

  - [x] 1.2 Implement OAuth Client model
    - Tạo `src/models/oauth_client.rs` với OAuthClient struct
    - Implement FromRow cho MySQL
    - _Requirements: 1.1, 1.5_

  - [x] 1.3 Implement OAuth Scope model
    - Tạo `src/models/oauth_scope.rs` với OAuthScope struct
    - _Requirements: 2.1, 2.2_

  - [x] 1.4 Implement User Consent model
    - Tạo `src/models/user_consent.rs` với UserConsent struct
    - _Requirements: 4.3_

  - [x] 1.5 Implement Authorization Code model
    - Tạo `src/models/authorization_code.rs` với AuthorizationCode struct
    - _Requirements: 3.4_

  - [x] 1.6 Implement OAuth Token model
    - Tạo `src/models/oauth_token.rs` với OAuthToken struct
    - _Requirements: 5.1, 5.6_

  - [x] 1.7 Implement OAuth Audit Log model
    - Tạo `src/models/oauth_audit_log.rs` với OAuthAuditLog struct
    - _Requirements: 9.5, 10.6_

- [x] 2. PKCE và Crypto Utilities
  - [x] 2.1 Implement PKCE utilities
    - Tạo `src/utils/pkce.rs` với verify_pkce và validate_code_verifier functions
    - Support S256 method
    - _Requirements: 3.5_

  - [ ]* 2.2 Write property test cho PKCE verification
    - **Property 10: PKCE Verification Round-Trip**
    - **Validates: Requirements 3.5**

  - [x] 2.3 Implement OAuth token hashing utilities
    - Extend `src/utils/secret.rs` hoặc tạo mới cho token hashing
    - _Requirements: 5.6_

  - [ ]* 2.4 Write property test cho token hashing
    - **Property 20: Token Storage Hashing**
    - **Validates: Requirements 5.6**

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. OAuth Repositories
  - [x] 4.1 Implement OAuthClientRepository
    - Tạo `src/repositories/oauth_client.rs`
    - CRUD operations cho oauth_clients
    - _Requirements: 1.1, 1.2_

  - [ ]* 4.2 Write property test cho client registration
    - **Property 1: Client Registration Data Integrity**
    - **Property 2: Client ID Uniqueness**
    - **Validates: Requirements 1.1, 1.2**

  - [x] 4.3 Implement OAuthScopeRepository
    - Tạo `src/repositories/oauth_scope.rs`
    - CRUD operations cho oauth_scopes
    - _Requirements: 2.1, 2.2_

  - [ ]* 4.4 Write property test cho scope uniqueness
    - **Property 5: Scope Code Uniqueness**
    - **Validates: Requirements 2.2**

  - [x] 4.5 Implement UserConsentRepository
    - Tạo `src/repositories/user_consent.rs`
    - CRUD operations cho user_consents
    - _Requirements: 4.3, 9.3_

  - [x] 4.6 Implement AuthorizationCodeRepository
    - Tạo `src/repositories/authorization_code.rs`
    - Create, find, mark as used operations
    - _Requirements: 3.4_

  - [x] 4.7 Implement OAuthTokenRepository
    - Tạo `src/repositories/oauth_token.rs`
    - CRUD operations, revocation support
    - _Requirements: 5.1, 5.6, 7.4, 9.2_

  - [x] 4.8 Implement OAuthAuditLogRepository
    - Tạo `src/repositories/oauth_audit_log.rs`
    - Insert và query operations
    - _Requirements: 9.5, 10.6_

- [ ] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. OAuth2 JWT Claims
  - [x] 6.1 Implement OAuth2Claims struct
    - Extend `src/utils/jwt.rs` với OAuth2Claims
    - Include sub, aud, scope, exp, iat, token_type
    - _Requirements: 5.4_

  - [x] 6.2 Implement create_oauth2_token và verify_oauth2_token
    - Methods trong JwtManager cho OAuth2 tokens
    - _Requirements: 5.4, 5.5, 8.1_

  - [ ]* 6.3 Write property tests cho OAuth2 JWT
    - **Property 18: JWT Required Claims**
    - **Property 19: JWT RS256 Algorithm**
    - **Property 16: Access Token Short Expiration**
    - **Validates: Requirements 5.2, 5.4, 5.5, 10.3**

- [x] 7. Consent Service
  - [x] 7.1 Implement ConsentService
    - Tạo `src/services/consent.rs`
    - has_consent, grant_consent, revoke_consent, list_user_consents
    - _Requirements: 4.2, 4.3, 4.5, 9.1, 9.3_

  - [ ]* 7.2 Write property tests cho consent
    - **Property 11: New Scope Consent Requirement**
    - **Property 12: Consent Record Integrity**
    - **Property 13: Existing Consent Skip**
    - **Validates: Requirements 4.2, 4.3, 4.5**

- [x] 8. OAuth Service - Core Logic
  - [x] 8.1 Implement OAuthService struct và validation methods
    - Tạo `src/services/oauth.rs`
    - validate_authorization_request, validate_redirect_uri
    - _Requirements: 3.1, 3.3, 10.5_

  - [ ]* 8.2 Write property tests cho authorization validation
    - **Property 7: Authorization Request Parameter Validation**
    - **Property 8: Redirect URI Exact Match**
    - **Validates: Requirements 3.1, 3.2, 3.3, 10.2, 10.5**

  - [x] 8.3 Implement create_authorization_code
    - Generate và store authorization code với PKCE
    - _Requirements: 3.4_

  - [ ]* 8.4 Write property test cho authorization code
    - **Property 9: Authorization Code Expiration**
    - **Validates: Requirements 3.4**

  - [x] 8.5 Implement exchange_code_for_tokens
    - Validate code, verify PKCE, issue tokens
    - _Requirements: 3.5, 5.1, 5.3_

  - [ ]* 8.6 Write property tests cho token exchange
    - **Property 15: Token Exchange Returns Both Tokens**
    - **Property 17: Token Response Scope Inclusion**
    - **Validates: Requirements 5.1, 5.3**

  - [x] 8.7 Implement client_credentials_grant
    - Authenticate client, issue service token
    - _Requirements: 6.1, 6.2, 6.5_

  - [ ]* 8.8 Write property test cho client credentials
    - **Property 21: Client Credentials Flow for Internal Apps**
    - **Property 14: Internal App Consent Skip**
    - **Validates: Requirements 6.1, 6.2, 4.6, 6.3**

  - [x] 8.9 Implement refresh_token method
    - Validate refresh token, rotate, issue new tokens
    - _Requirements: 7.1, 7.2, 7.4_

  - [ ]* 8.10 Write property tests cho refresh token
    - **Property 22: Refresh Token Returns New Access Token**
    - **Property 23: Refresh Token Rotation**
    - **Property 24: Old Refresh Token Invalidation**
    - **Validates: Requirements 7.1, 7.2, 7.4, 10.4**

  - [x] 8.11 Implement revoke_token method
    - Revoke specific token hoặc all tokens for client-user
    - _Requirements: 9.2, 9.4_

  - [ ]* 8.12 Write property tests cho revocation
    - **Property 30: Revocation Invalidates All Tokens**
    - **Property 32: Token Revocation Endpoint**
    - **Validates: Requirements 9.2, 9.4**

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. OAuth DTOs
  - [x] 10.1 Implement OAuth request/response DTOs
    - Tạo `src/dto/oauth.rs`
    - AuthorizationRequest, TokenRequest, TokenResponse, RevokeRequest
    - _Requirements: 11.1, 11.2, 11.3_

  - [x] 10.2 Implement OAuth error response DTO
    - OAuthErrorResponse theo RFC 6749
    - _Requirements: 3.6, 6.4, 7.3_

- [x] 11. OAuth Handlers
  - [x] 11.1 Implement authorize_handler
    - GET /oauth/authorize endpoint
    - Validate request, check consent, redirect
    - _Requirements: 3.1, 4.1, 11.1_

  - [x] 11.2 Implement token_handler
    - POST /oauth/token endpoint
    - Support authorization_code, client_credentials, refresh_token grants
    - _Requirements: 5.1, 6.1, 7.1, 11.2_

  - [x] 11.3 Implement revoke_handler
    - POST /oauth/revoke endpoint
    - _Requirements: 9.4, 11.3_

  - [x] 11.4 Implement userinfo_handler
    - GET /oauth/userinfo endpoint
    - Return user profile based on token scopes
    - _Requirements: 11.4_

  - [x] 11.5 Implement openid_configuration_handler
    - GET /.well-known/openid-configuration endpoint
    - Return discovery metadata
    - _Requirements: 11.5_

- [x] 12. OAuth Middleware
  - [x] 12.1 Implement oauth_token_middleware
    - Validate OAuth2 access tokens
    - Extract user_id và scopes
    - _Requirements: 8.1, 8.2, 8.4_

  - [ ]* 12.2 Write property tests cho token validation
    - **Property 26: Token Signature and Expiration Validation**
    - **Property 28: Token Claims Extraction**
    - **Validates: Requirements 8.1, 8.4**

  - [x] 12.3 Implement scope_guard middleware
    - Check required scopes for endpoints
    - Return 403 if scope missing
    - _Requirements: 8.3_

  - [ ]* 12.4 Write property test cho scope enforcement
    - **Property 27: Scope Enforcement**
    - **Validates: Requirements 8.3**

- [x] 13. Client Registration và Management
  - [x] 13.1 Implement client registration handler
    - POST /oauth/clients endpoint
    - Validate redirect URIs (HTTPS for external)
    - _Requirements: 1.1, 1.4_

  - [ ]* 13.2 Write property tests cho client validation
    - **Property 3: Client Secret Hashing**
    - **Property 4: External App HTTPS Redirect URI Validation**
    - **Validates: Requirements 1.3, 1.4**

  - [x] 13.3 Implement connected apps handler
    - GET /account/connected-apps endpoint
    - List apps with consents
    - _Requirements: 9.1_

  - [ ]* 13.4 Write property test cho connected apps
    - **Property 29: Connected Apps List Accuracy**
    - **Validates: Requirements 9.1**

  - [x] 13.5 Implement revoke consent handler
    - DELETE /account/connected-apps/{client_id} endpoint
    - Revoke consent và tokens
    - _Requirements: 9.2, 9.3_

  - [ ]* 13.6 Write property test cho consent revocation
    - **Property 31: Revocation Deletes Consent**
    - **Validates: Requirements 9.3**

- [x] 14. Audit Logging
  - [x] 14.1 Implement audit logging trong OAuth service
    - Log authorization, token issuance, revocation events
    - _Requirements: 9.5, 10.6_

  - [ ]* 14.2 Write property test cho audit logging
    - **Property 33: Audit Log Creation**
    - **Validates: Requirements 9.5, 10.6**

- [x] 15. Scope Validation
  - [x] 15.1 Implement scope validation trong OAuth service
    - Validate requested scopes exist
    - _Requirements: 2.4_

  - [ ]* 15.2 Write property test cho scope validation
    - **Property 6: Invalid Scope Rejection**
    - **Validates: Requirements 2.4**

- [x] 16. Revoked Token Cascade
  - [x] 16.1 Implement revoked token detection và cascade
    - Detect use of revoked refresh token
    - Revoke all tokens for client-user pair
    - _Requirements: 7.5_

  - [ ]* 16.2 Write property test cho cascade revocation
    - **Property 25: Revoked Token Cascade**
    - **Validates: Requirements 7.5**

- [x] 17. Router Integration
  - [x] 17.1 Add OAuth routes to main router
    - Wire up all OAuth endpoints
    - Apply appropriate middleware
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

  - [x] 17.2 Update mod.rs files
    - Export new modules
    - _Requirements: All_

- [ ] 18. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases

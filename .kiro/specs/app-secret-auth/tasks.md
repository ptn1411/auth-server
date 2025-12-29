# Implementation Plan: App Secret Authentication

## Overview

Triển khai tính năng App Secret Authentication cho phép các ứng dụng xác thực bằng App ID và Secret để quản lý roles/permissions. Bao gồm cả endpoint GET /users/me để user lấy thông tin profile.

## Tasks

- [x] 1. Database Migration - Thêm secret_hash vào apps table
  - Tạo migration file thêm cột `secret_hash VARCHAR(255)` vào bảng `apps`
  - _Requirements: 1.3_

- [x] 2. Update App Model và Repository
  - [x] 2.1 Update App model với field secret_hash
    - Thêm `secret_hash: Option<String>` vào struct App và AppRow
    - Update FromRow implementation
    - _Requirements: 1.3_
  
  - [x] 2.2 Implement secret generation utility
    - Tạo function `generate_secret()` sinh chuỗi 32+ ký tự
    - Sử dụng `rand` crate với alphanumeric + special chars
    - _Requirements: 1.1, 1.4_
  
  - [ ]* 2.3 Write property test for secret generation
    - **Property 1: Secret Generation Quality**
    - **Validates: Requirements 1.1, 1.4**
  
  - [x] 2.4 Update AppRepository với secret operations
    - `create_with_secret()` - tạo app với secret hash
    - `update_secret_hash()` - update secret hash
    - `get_secret_hash()` - lấy secret hash để verify
    - _Requirements: 1.3, 2.1, 2.2_

- [ ] 3. Checkpoint - Ensure model and repository compile
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement App Authentication DTOs
  - [x] 4.1 Create AppAuthRequest và AppAuthResponse DTOs
    - `AppAuthRequest { app_id: Uuid, secret: String }`
    - `AppAuthResponse { access_token, token_type, expires_in }`
    - _Requirements: 3.1, 3.2_
  
  - [x] 4.2 Create CreateAppWithSecretResponse DTO
    - Include `secret` field for one-time return
    - _Requirements: 1.2_
  
  - [x] 4.3 Create UserProfileResponse DTO
    - Include id, email, is_active, email_verified, created_at
    - Exclude password_hash
    - _Requirements: 8.3, 8.4_

- [x] 5. Implement App Token Claims
  - [x] 5.1 Create AppClaims struct
    - Fields: sub, app_id, token_type ("app"), exp, iat
    - _Requirements: 3.2, 7.4_
  
  - [x] 5.2 Update JwtManager for app tokens
    - `create_app_token(app_id)` - tạo token cho app
    - `verify_app_token(token)` - verify và trả về AppClaims
    - _Requirements: 3.1, 3.2_
  
  - [ ]* 5.3 Write property test for app token claims
    - **Property 7: App Token Contains App Context**
    - **Validates: Requirements 3.2, 7.4**

- [x] 6. Implement AppService Methods
  - [x] 6.1 Implement create_app_with_secret
    - Generate secret, hash with bcrypt, store hash, return plain secret
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [ ]* 6.2 Write property test for secret storage
    - **Property 2: Secret Storage Security**
    - **Validates: Requirements 1.3, 9.2**
  
  - [x] 6.3 Implement authenticate_app
    - Verify app exists, verify secret with bcrypt, return token
    - Use generic error for both invalid app_id and secret
    - _Requirements: 3.1, 3.3, 3.4, 9.3_
  
  - [ ]* 6.4 Write property test for authentication
    - **Property 5: App Authentication Round-Trip**
    - **Property 6: Invalid Credentials Rejection**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 9.3**
  
  - [x] 6.5 Implement regenerate_secret
    - Verify requester is owner, generate new secret, update hash
    - _Requirements: 2.1, 2.2, 2.4_
  
  - [ ]* 6.6 Write property test for secret regeneration
    - **Property 3: Secret Regeneration Invalidates Previous**
    - **Property 4: Owner-Only Secret Regeneration**
    - **Validates: Requirements 2.1, 2.2, 2.4**

- [ ] 7. Checkpoint - Ensure service layer works
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Implement Error Types
  - [x] 8.1 Add AppAuthError enum
    - InvalidCredentials, NotAppOwner, CrossAppAccess, UserInactive
    - Implement IntoResponse
    - _Requirements: 3.3, 3.4, 2.4, 9.3_

- [x] 9. Implement App Auth Middleware
  - [x] 9.1 Create app_auth_middleware
    - Extract Bearer token, verify as app token
    - Inject app_id into request extensions
    - _Requirements: 7.3_
  
  - [x] 9.2 Create AppContext extractor
    - Extract app_id from request extensions
    - _Requirements: 4.1, 5.1_

- [x] 10. Implement Handlers
  - [x] 10.1 Update create_app_handler
    - Return CreateAppWithSecretResponse với secret
    - _Requirements: 1.1, 1.2_
  
  - [x] 10.2 Implement app_auth_handler
    - POST /apps/auth - xác thực và trả về token
    - _Requirements: 3.1, 3.2, 7.1_
  
  - [x] 10.3 Implement regenerate_secret_handler
    - POST /apps/{id}/secret/regenerate
    - Verify owner, regenerate, return new secret
    - _Requirements: 2.1, 2.3, 7.2_
  
  - [x] 10.4 Implement get_user_profile_handler
    - GET /users/me - return user profile from token
    - Check user is active
    - _Requirements: 8.1, 8.2, 8.5, 8.6_
  
  - [ ]* 10.5 Write property test for user profile
    - **Property 11: User Profile Response Format**
    - **Property 12: User Profile Authentication**
    - **Validates: Requirements 8.2, 8.3, 8.4, 8.5, 8.6**

- [x] 11. Update Role/Permission Handlers for App Auth
  - [x] 11.1 Add app-authenticated role endpoints
    - POST /apps/{id}/roles - create role (app auth)
    - GET /apps/{id}/roles - list roles (app auth)
    - Verify app_id matches token
    - _Requirements: 4.1, 4.2, 4.5_
  
  - [ ]* 11.2 Write property test for role scope isolation
    - **Property 8: Role Scope Isolation**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**
  
  - [x] 11.3 Add app-authenticated permission endpoints
    - POST /apps/{id}/permissions - create permission (app auth)
    - GET /apps/{id}/permissions - list permissions (app auth)
    - Verify app_id matches token
    - _Requirements: 5.1, 5.2, 5.5_
  
  - [ ]* 11.4 Write property test for permission scope isolation
    - **Property 9: Permission Scope Isolation**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**
  
  - [x] 11.5 Add role-permission assignment endpoint
    - POST /apps/{id}/roles/{role_id}/permissions
    - Verify role and permission belong to same app
    - _Requirements: 6.1, 6.2, 6.3_
  
  - [ ]* 11.6 Write property test for role-permission assignment
    - **Property 10: Role-Permission Assignment Validation**
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4**

- [x] 12. Wire Routes
  - [x] 12.1 Add new routes to main.rs
    - POST /apps/auth (public)
    - POST /apps/{id}/secret/regenerate (user auth, owner only)
    - GET /users/me (user auth)
    - App-authenticated routes with app_auth_middleware
    - _Requirements: 7.1, 7.2, 8.1_

- [ ] 13. Final Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional property-based tests
- Secret chỉ được trả về 1 lần khi tạo hoặc regenerate
- App token có `token_type: "app"` để phân biệt với user token
- Sử dụng bcrypt với cost factor >= 10 cho secret hashing

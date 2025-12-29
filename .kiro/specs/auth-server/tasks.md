# Implementation Plan: Auth Server

## Overview

Implementation plan cho Auth Server sử dụng Rust với axum framework. Tasks được chia thành các phase: setup, core domain, authentication, authorization, và integration.

## Tasks

- [x] 1. Project Setup và Core Infrastructure
  - [x] 1.1 Initialize Rust project với Cargo.toml và dependencies
    - Thêm dependencies: axum, tokio, sqlx, argon2, jsonwebtoken, uuid, serde, thiserror, anyhow, chrono
    - Configure sqlx với PostgreSQL
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7_
  - [x] 1.2 Create database migrations
    - Tạo migration files cho tất cả tables: users, apps, roles, permissions, user_app_roles, role_permissions, refresh_tokens, password_reset_tokens
    - Tạo indexes
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7_
  - [-] 1.3 Setup application configuration và state
    - Config struct cho database URL, JWT keys, token expiry
    - AppState struct với database pool và config
    - _Requirements: 15.1, 15.2, 15.3_

- [x] 2. Domain Models và Error Types
  - [x] 2.1 Implement domain models
    - User, App, Role, Permission structs
    - UserAppRole, RolePermission association structs
    - _Requirements: 13.2, 13.3, 13.4, 13.5, 13.6, 13.7_
  - [x] 2.2 Implement error types
    - AuthError, AppError, RoleError, PermissionError enums
    - IntoResponse implementations cho HTTP error responses
    - _Requirements: 2.2, 5.2, 6.2, 7.2, 8.2, 9.2_
  - [x] 2.3 Implement request/response DTOs
    - RegisterRequest, LoginRequest, RefreshRequest
    - CreateAppRequest, CreateRoleRequest, CreatePermissionRequest
    - Response structs với Serialize
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5, 14.6, 14.7, 14.8, 14.9_

- [ ] 3. Repository Layer
  - [x] 3.1 Implement UserRepository
    - create_user, find_by_email, find_by_id, update_password, set_active
    - _Requirements: 1.1, 1.2, 2.1, 4.3_
  - [x] 3.2 Write property tests for UserRepository

    - **Property 2: Email Uniqueness**
    - **Property 4: Valid Registration Creates User**
    - **Validates: Requirements 1.1, 1.2**
  - [x] 3.3 Implement AppRepository
    - create_app, find_by_id, find_by_code
    - _Requirements: 5.1, 5.2_
  - [x] 3.4 Write property tests for AppRepository

    - **Property 16: App Code Uniqueness**
    - **Validates: Requirements 5.2**
  - [x] 3.5 Implement RoleRepository
    - create_role, find_by_id, find_by_app_id, find_by_app_and_name
    - _Requirements: 6.1, 6.2_
  - [ ]* 3.6 Write property tests for RoleRepository
    - **Property 17: Role Scoped to App**
    - **Property 18: Role Name Uniqueness Within App**
    - **Validates: Requirements 6.1, 6.2**
  - [x] 3.7 Implement PermissionRepository
    - create_permission, find_by_id, find_by_app_id, find_by_app_and_code
    - _Requirements: 7.1, 7.2_
  - [ ]* 3.8 Write property tests for PermissionRepository
    - **Property 19: Permission Scoped to App**
    - **Property 20: Permission Code Uniqueness Within App**
    - **Validates: Requirements 7.1, 7.2**
  - [x] 3.9 Implement UserAppRoleRepository
    - assign_role, remove_role, find_by_user, find_by_user_and_app
    - _Requirements: 8.1, 8.3_
  - [ ]* 3.10 Write property tests for UserAppRoleRepository
    - **Property 21: User Role Assignment Creates Association**
    - **Property 23: User Multi-App Role Support**
    - **Validates: Requirements 8.1, 8.3**
  - [x] 3.11 Implement RolePermissionRepository
    - assign_permission, remove_permission, find_by_role
    - _Requirements: 9.1_
  - [ ]* 3.12 Write property tests for RolePermissionRepository
    - **Property 24: Role Permission Assignment Creates Association**
    - **Validates: Requirements 9.1**

- [ ] 4. Checkpoint - Repository Layer Complete
  - Ensure all repository tests pass, ask the user if questions arise.

- [x] 5. Password và Token Utilities
  - [x] 5.1 Implement password hashing với argon2
    - hash_password, verify_password functions
    - _Requirements: 1.1, 1.5, 2.1_
  - [ ]* 5.2 Write property tests for password hashing
    - **Property 1: Password Storage Security**
    - **Validates: Requirements 1.5**
  - [x] 5.3 Implement email validation
    - validate_email function với regex
    - _Requirements: 1.3_
  - [ ]* 5.4 Write property tests for email validation
    - **Property 3: Email Format Validation**
    - **Validates: Requirements 1.3**
  - [x] 5.5 Implement JWT token generation và verification
    - Generate RS256 key pair
    - create_access_token, create_refresh_token, verify_token functions
    - Claims struct với proper structure
    - _Requirements: 2.4, 2.5, 10.1, 10.2, 10.3_
  - [ ]* 5.6 Write property tests for JWT
    - **Property 8: JWT Token Structure Correctness**
    - **Property 9: Token Expiry Duration**
    - **Validates: Requirements 2.4, 2.5, 10.1, 10.2, 10.3**

- [ ] 6. Checkpoint - Utilities Complete
  - Ensure all utility tests pass, ask the user if questions arise.

- [x] 7. Authentication Service
  - [x] 7.1 Implement AuthService::register
    - Validate email format, check uniqueness, hash password, create user
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - [x] 7.2 Implement AuthService::login
    - Verify credentials, check is_active, generate tokens with roles/permissions
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - [ ]* 7.3 Write property tests for login
    - **Property 5: Valid Login Returns Tokens**
    - **Property 6: Invalid Credentials Rejection**
    - **Property 7: Inactive User Login Rejection**
    - **Validates: Requirements 2.1, 2.2, 2.3**
  - [x] 7.4 Implement AuthService::refresh
    - Verify refresh token, generate new access token with updated permissions
    - _Requirements: 3.1, 3.2, 3.3_
  - [ ]* 7.5 Write property tests for refresh
    - **Property 10: Valid Refresh Token Returns New Access Token**
    - **Property 11: Invalid Refresh Token Rejection**
    - **Property 12: Refreshed Token Contains Updated Permissions**
    - **Validates: Requirements 3.1, 3.2, 3.3**
  - [x] 7.6 Implement AuthService::forgot_password
    - Generate reset token, store hashed token
    - Return same response regardless of email existence
    - _Requirements: 4.1, 4.2_
  - [x] 7.7 Implement AuthService::reset_password
    - Verify reset token, update password hash
    - _Requirements: 4.3, 4.4_
  - [ ]* 7.8 Write property tests for password reset
    - **Property 13: Password Reset Round Trip**
    - **Property 14: Password Reset Email Privacy**
    - **Property 15: Invalid Reset Token Rejection**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4**

- [ ] 8. Checkpoint - Authentication Service Complete
  - Ensure all authentication tests pass, ask the user if questions arise.

- [x] 9. App, Role, Permission Services
  - [x] 9.1 Implement AppService
    - create_app với code uniqueness check
    - _Requirements: 5.1, 5.2_
  - [x] 9.2 Implement RoleService
    - create_role với app scope và name uniqueness
    - assign_role_to_user với validation
    - _Requirements: 6.1, 6.2, 8.1, 8.2_
  - [ ]* 9.3 Write property tests for role assignment
    - **Property 22: Invalid Role Assignment Rejection**
    - **Validates: Requirements 8.2**
  - [x] 9.4 Implement PermissionService
    - create_permission với app scope và code uniqueness
    - assign_permission_to_role với cross-app validation
    - _Requirements: 7.1, 7.2, 9.1, 9.2_
  - [ ]* 9.5 Write property tests for permission assignment
    - **Property 25: Cross-App Permission Assignment Rejection**
    - **Validates: Requirements 9.2**

- [x] 10. Authorization (RBAC)
  - [x] 10.1 Implement can() authorization function
    - Check if user has permission in specific app scope
    - _Requirements: 12.1, 12.2_
  - [ ]* 10.2 Write property tests for authorization
    - **Property 28: Authorization Check Correctness**
    - **Property 29: Cross-App Permission Isolation**
    - **Validates: Requirements 12.1, 12.2**

- [ ] 11. Checkpoint - Services Complete
  - Ensure all service tests pass, ask the user if questions arise.

- [x] 12. JWT Middleware
  - [x] 12.1 Implement jwt_auth_middleware
    - Extract token from Authorization header
    - Verify signature và expiry
    - Inject claims vào request extensions
    - _Requirements: 11.1, 11.2, 11.3, 11.4_
  - [ ]* 12.2 Write property tests for middleware
    - **Property 26: Valid JWT Passes Middleware**
    - **Property 27: Invalid JWT Rejected by Middleware**
    - **Validates: Requirements 11.1, 11.2, 11.3**

- [x] 13. API Handlers
  - [x] 13.1 Implement auth handlers
    - register_handler, login_handler, refresh_handler
    - forgot_password_handler, reset_password_handler
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_
  - [x] 13.2 Implement app management handlers
    - create_app_handler
    - _Requirements: 14.6_
  - [x] 13.3 Implement role handlers
    - create_role_handler, assign_role_handler
    - _Requirements: 14.7, 14.9_
  - [x] 13.4 Implement permission handlers
    - create_permission_handler
    - _Requirements: 14.8_
  - [ ]* 13.5 Write integration tests for API endpoints
    - Test full request/response cycle
    - _Requirements: 14.1-14.10_

- [x] 14. Router và Application Setup
  - [x] 14.1 Setup axum router
    - Configure routes với handlers
    - Apply middleware to protected routes
    - _Requirements: 14.1-14.10_
  - [x] 14.2 Implement main.rs
    - Load config, setup database pool
    - Start server
    - _Requirements: 15.1, 15.2, 15.3_

- [ ] 15. Final Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 16. Documentation và Deliverables
  - [x] 16.1 Create OpenAPI/Swagger specification
    - Document all endpoints với request/response schemas
  - [x] 16.2 Create README với setup instructions
    - Database setup, environment variables, run commands

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties using `proptest` crate
- Unit tests validate specific examples and edge cases

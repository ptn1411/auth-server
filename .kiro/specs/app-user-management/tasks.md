# Implementation Plan: App User Management

## Overview

Implementation plan cho tính năng App User Management, mở rộng Auth Server hiện có. Tasks được chia thành: database migration, repository layer, service layer, handlers, và integration.

## Tasks

- [x] 1. Database Migration
  - [x] 1.1 Create migration file for schema updates
    - Add owner_id column to apps table (FK to users, nullable)
    - Add is_system_admin column to users table (boolean, default false)
    - Create user_apps table với user_id, app_id, status, banned_at, banned_reason, created_at
    - Add indexes cho user_apps table
    - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 2. Domain Models và DTOs
  - [x] 2.1 Update App model
    - Add owner_id field to App struct
    - Update sqlx FromRow derive
    - _Requirements: 1.2_
  - [x] 2.2 Update User model
    - Add is_system_admin field to User struct
    - _Requirements: 7.1_
  - [x] 2.3 Create UserApp model và UserAppStatus enum
    - UserApp struct với all fields
    - UserAppStatus enum (Active, Banned)
    - _Requirements: 2.4_
  - [x] 2.4 Create request/response DTOs
    - RegisterToAppRequest, BanUserRequest
    - AppUserInfo, PaginatedResponse
    - _Requirements: 8.1-8.8_

- [x] 3. Repository Layer
  - [x] 3.1 Update AppRepository
    - Add create_with_owner method
    - Add is_owner method
    - Add get_owner method
    - _Requirements: 1.1, 1.3_
  - [ ]* 3.2 Write property test for App Ownership
    - **Property 1: App Creation Assigns Owner**
    - **Validates: Requirements 1.1, 1.3, 1.4**
  - [x] 3.3 Implement UserAppRepository
    - create, find, update_status, delete methods
    - list_by_app với pagination
    - is_banned method
    - _Requirements: 2.1, 2.4, 3.1, 4.1, 5.1_
  - [ ]* 3.4 Write property tests for UserAppRepository
    - **Property 2: User App Registration Creates Active Association**
    - **Property 8: Ban/Unban Round Trip**
    - **Validates: Requirements 2.1, 4.1**
  - [x] 3.5 Update UserRepository
    - Add is_system_admin method
    - Add set_system_admin method
    - Add deactivate method
    - Add list_all method với pagination
    - _Requirements: 7.1, 7.5_

- [ ] 4. Checkpoint - Repository Layer Complete
  - Ensure all repository tests pass, ask the user if questions arise.

- [x] 5. Service Layer
  - [x] 5.1 Implement UserManagementService::register_to_app
    - Check if user is banned
    - Check if already registered
    - Create user_app association
    - _Requirements: 2.1, 2.2, 2.3_
  - [ ]* 5.2 Write property tests for registration
    - **Property 3: Banned User Registration Rejection**
    - **Property 4: Duplicate Registration Rejection**
    - **Validates: Requirements 2.2, 2.3**
  - [x] 5.3 Implement UserManagementService::ban_user
    - Check permission (owner or admin)
    - Update status to banned
    - Create banned record if user not registered
    - _Requirements: 3.1, 3.2, 3.3, 3.5_
  - [x] 5.4 Implement UserManagementService::unban_user
    - Check permission (owner or admin)
    - Update status to active, clear banned_at
    - Handle idempotent unban
    - _Requirements: 4.1, 4.2, 4.3_
  - [ ]* 5.5 Write property tests for ban/unban
    - **Property 5: Ban Operation Updates Status**
    - **Property 9: Unban Idempotence**
    - **Validates: Requirements 3.1, 3.2, 4.3**
  - [x] 5.6 Implement UserManagementService::remove_user
    - Check permission (owner or admin)
    - Delete user_app association
    - Delete user_app_roles for user in app
    - Handle idempotent remove
    - _Requirements: 5.1, 5.2, 5.3_
  - [ ]* 5.7 Write property tests for remove
    - **Property 10: Remove User Deletes Associations**
    - **Property 11: Remove Idempotence**
    - **Validates: Requirements 5.1, 5.3**
  - [x] 5.8 Implement UserManagementService::list_app_users
    - Check permission (owner or admin)
    - Return paginated list with user info
    - _Requirements: 6.1, 6.2, 6.3_
  - [ ]* 5.9 Write property test for list
    - **Property 12: List App Users Returns All Registered Users**
    - **Validates: Requirements 6.1, 6.2**
  - [x] 5.10 Implement check_permission helper
    - Check if actor is owner OR system admin
    - _Requirements: 3.3, 4.2, 5.2, 6.3, 7.2_
  - [ ]* 5.11 Write property test for authorization
    - **Property 6: Non-Owner Authorization Rejection**
    - **Validates: Requirements 3.3, 4.2, 5.2, 6.3**

- [ ] 6. Checkpoint - User Management Service Complete
  - Ensure all service tests pass, ask the user if questions arise.

- [x] 7. Admin Service
  - [x] 7.1 Implement AdminService
    - list_all_users với pagination
    - list_all_apps với pagination
    - deactivate_user
    - verify_admin helper
    - _Requirements: 7.2, 7.3, 7.4, 7.5_
  - [ ]* 7.2 Write property tests for admin
    - **Property 13: System Admin Override**
    - **Property 14: System Admin Global Deactivation**
    - **Validates: Requirements 7.2, 7.3, 7.4, 7.5**

- [x] 8. Update Auth Service
  - [x] 8.1 Update login to check user_app ban status
    - Check if user is banned from app before allowing login
    - Return appropriate error for banned users
    - _Requirements: 3.4_
  - [ ]* 8.2 Write property test for banned login
    - **Property 7: Banned User Login Rejection**
    - **Validates: Requirements 3.4**

- [ ] 9. Checkpoint - Services Complete
  - Ensure all service tests pass, ask the user if questions arise.

- [x] 10. API Handlers
  - [x] 10.1 Implement app user management handlers
    - register_to_app_handler (POST /apps/{app_id}/register)
    - ban_user_handler (POST /apps/{app_id}/users/{user_id}/ban)
    - unban_user_handler (POST /apps/{app_id}/users/{user_id}/unban)
    - remove_user_handler (DELETE /apps/{app_id}/users/{user_id})
    - list_app_users_handler (GET /apps/{app_id}/users)
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  - [x] 10.2 Implement admin handlers
    - list_all_users_handler (GET /admin/users)
    - list_all_apps_handler (GET /admin/apps)
    - deactivate_user_handler (POST /admin/users/{user_id}/deactivate)
    - _Requirements: 8.6, 8.7, 8.8_
  - [x] 10.3 Update create_app_handler
    - Set owner_id to current user when creating app
    - _Requirements: 1.1_

- [x] 11. Router Updates
  - [x] 11.1 Add new routes to router
    - App user management routes
    - Admin routes với admin middleware
    - _Requirements: 8.1-8.8_

- [ ] 12. Checkpoint - Handlers Complete
  - Ensure all handler tests pass, ask the user if questions arise.

- [ ]* 13. Integration Tests
  - [ ]* 13.1 Write integration tests for app user management
    - Test full ban/unban/remove flow
    - Test authorization checks
    - _Requirements: 3.1-5.4_
  - [ ]* 13.2 Write integration tests for admin operations
    - Test admin override
    - Test global deactivation
    - _Requirements: 7.2-7.5_

- [ ] 14. Final Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties using `proptest` crate
- Implementation builds on existing Auth Server codebase

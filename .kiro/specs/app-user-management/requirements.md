# Requirements Document

## Introduction

Tính năng quản lý user trong App cho phép App Owner (người tạo app) có quyền quản lý user trong phạm vi app của mình. App Owner có thể ban (cấm) hoặc xóa user khỏi app, ngăn không cho user đó đăng ký hoặc truy cập app.

## Glossary

- **Auth_Server**: Hệ thống xác thực trung tâm
- **App_Owner**: User đã tạo app, có toàn quyền quản lý app đó
- **App_User**: User đã đăng ký sử dụng một app cụ thể
- **Banned_User**: User bị cấm truy cập một app cụ thể
- **User_App_Status**: Trạng thái của user trong một app (active, banned, removed)
- **System_Admin**: Admin toàn hệ thống, có quyền quản lý tất cả user và app

## Requirements

### Requirement 1: App Ownership

**User Story:** As a user, I want to create an app and become its owner, so that I can manage users within my app.

#### Acceptance Criteria

1. WHEN a user creates a new app, THE Auth_Server SHALL assign that user as the App_Owner of the created app
2. THE Auth_Server SHALL store owner_id (FK to users) in the apps table
3. WHEN an App_Owner is queried, THE Auth_Server SHALL return the user who created the app
4. THE Auth_Server SHALL allow only one owner per app

### Requirement 2: User App Registration

**User Story:** As a user, I want to register to use a specific app, so that I can access its features.

#### Acceptance Criteria

1. WHEN a user registers to an app, THE Auth_Server SHALL create a user_app association with status "active"
2. WHEN a banned user attempts to register to an app, THE Auth_Server SHALL reject the registration and return an error indicating user is banned
3. WHEN a user is already registered to an app, THE Auth_Server SHALL reject duplicate registration
4. THE Auth_Server SHALL store user_app associations with: user_id, app_id, status, banned_at, banned_reason

### Requirement 3: Ban User from App

**User Story:** As an App_Owner, I want to ban a user from my app, so that they cannot access or register to my app.

#### Acceptance Criteria

1. WHEN an App_Owner bans a user from their app, THE Auth_Server SHALL update the user_app status to "banned" and record banned_at timestamp
2. WHEN an App_Owner bans a user, THE Auth_Server SHALL optionally store a ban reason
3. WHEN a non-owner attempts to ban a user, THE Auth_Server SHALL reject the request with authorization error
4. WHEN a banned user attempts to login to the app, THE Auth_Server SHALL reject the login and indicate user is banned
5. IF a user is not registered to the app, THEN THE Auth_Server SHALL create a banned user_app record to prevent future registration

### Requirement 4: Unban User from App

**User Story:** As an App_Owner, I want to unban a user from my app, so that they can access my app again.

#### Acceptance Criteria

1. WHEN an App_Owner unbans a user, THE Auth_Server SHALL update the user_app status to "active" and clear banned_at
2. WHEN a non-owner attempts to unban a user, THE Auth_Server SHALL reject the request with authorization error
3. WHEN unbanning a user who is not banned, THE Auth_Server SHALL return success without changes

### Requirement 5: Remove User from App

**User Story:** As an App_Owner, I want to remove a user from my app, so that they lose access but can re-register later.

#### Acceptance Criteria

1. WHEN an App_Owner removes a user from their app, THE Auth_Server SHALL delete the user_app association and all user_app_roles for that user in that app
2. WHEN a non-owner attempts to remove a user, THE Auth_Server SHALL reject the request with authorization error
3. WHEN removing a user who is not registered, THE Auth_Server SHALL return success without changes
4. WHEN a removed user attempts to access the app, THE Auth_Server SHALL treat them as unregistered

### Requirement 6: List App Users

**User Story:** As an App_Owner, I want to view all users in my app, so that I can manage them effectively.

#### Acceptance Criteria

1. WHEN an App_Owner requests user list, THE Auth_Server SHALL return all users registered to the app with their status
2. THE Auth_Server SHALL include user email, status, roles, banned_at, and banned_reason in the response
3. WHEN a non-owner requests user list, THE Auth_Server SHALL reject the request with authorization error
4. THE Auth_Server SHALL support pagination for large user lists

### Requirement 7: System Admin Management

**User Story:** As a System_Admin, I want to manage all users and apps, so that I can maintain the entire system.

#### Acceptance Criteria

1. THE Auth_Server SHALL have a special "system_admin" flag on users table
2. WHEN a System_Admin performs any app management action, THE Auth_Server SHALL allow it regardless of app ownership
3. WHEN a System_Admin bans/unbans/removes a user from any app, THE Auth_Server SHALL execute the action
4. THE Auth_Server SHALL allow System_Admin to list all apps and all users
5. THE Auth_Server SHALL allow System_Admin to deactivate any user globally (set is_active = false)

### Requirement 8: API Endpoints for App User Management

**User Story:** As a client, I want RESTful API endpoints for app user management, so that I can integrate these features.

#### Acceptance Criteria

1. THE Auth_Server SHALL expose POST /apps/{app_id}/users/{user_id}/ban for banning a user
2. THE Auth_Server SHALL expose POST /apps/{app_id}/users/{user_id}/unban for unbanning a user
3. THE Auth_Server SHALL expose DELETE /apps/{app_id}/users/{user_id} for removing a user
4. THE Auth_Server SHALL expose GET /apps/{app_id}/users for listing app users
5. THE Auth_Server SHALL expose POST /apps/{app_id}/register for user app registration
6. THE Auth_Server SHALL expose GET /admin/users for System_Admin to list all users
7. THE Auth_Server SHALL expose GET /admin/apps for System_Admin to list all apps
8. THE Auth_Server SHALL expose POST /admin/users/{user_id}/deactivate for System_Admin to deactivate a user

### Requirement 9: Database Schema Updates

**User Story:** As a developer, I want updated database schema to support app user management.

#### Acceptance Criteria

1. THE Auth_Server SHALL add owner_id column to apps table (FK to users, nullable for existing apps)
2. THE Auth_Server SHALL create user_apps table with: user_id, app_id, status (enum: active, banned), banned_at, banned_reason, created_at
3. THE Auth_Server SHALL add is_system_admin column to users table (boolean, default false)
4. THE Auth_Server SHALL add unique constraint on (user_id, app_id) in user_apps table


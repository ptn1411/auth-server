# Requirements Document

## Introduction

Tài liệu này mô tả các yêu cầu cho việc mở rộng Auth Frontend với các tính năng quản lý App cho user thường, tính năng Admin Panel với router riêng, và tích hợp đầy đủ các API còn lại từ SDK.

## Glossary

- **Frontend**: Ứng dụng React sử dụng Vite, shadcn/ui, Zustand
- **Admin_Panel**: Khu vực quản trị dành cho system admin với router `/admin/*`
- **App_Management**: Tính năng quản lý ứng dụng bên ngoài (external apps)
- **User**: Người dùng thông thường đã đăng nhập
- **System_Admin**: Người dùng có quyền `is_system_admin = true`
- **Connected_App**: Ứng dụng OAuth đã được user cấp quyền truy cập
- **Webhook**: Endpoint nhận thông báo sự kiện từ hệ thống
- **API_Key**: Khóa xác thực cho ứng dụng bên ngoài
- **IP_Rule**: Quy tắc whitelist/blacklist IP

## Requirements

### Requirement 1: App Management cho User

**User Story:** As a user, I want to manage my external applications, so that I can integrate third-party services with the auth system.

#### Acceptance Criteria

1. WHEN a user navigates to the Apps page, THE Frontend SHALL display a list of apps owned by the user
2. WHEN a user clicks "Create App", THE Frontend SHALL show a form to create new app with code and name fields
3. WHEN a user creates an app successfully, THE Frontend SHALL display the app secret once and warn user to save it
4. WHEN a user clicks "Regenerate Secret" on an app, THE Frontend SHALL confirm the action and display new secret
5. WHEN a user views app details, THE Frontend SHALL show app info, roles, permissions, webhooks, API keys, and IP rules

### Requirement 2: Role Management trong App

**User Story:** As an app owner, I want to manage roles within my app, so that I can control user permissions.

#### Acceptance Criteria

1. WHEN viewing app details, THE Frontend SHALL display a list of roles for that app
2. WHEN a user clicks "Create Role", THE Frontend SHALL show a form to create new role with name field
3. WHEN a user creates a role successfully, THE Frontend SHALL add it to the roles list
4. WHEN a user views app users, THE Frontend SHALL show option to assign/remove roles for each user

### Requirement 3: Permission Management trong App

**User Story:** As an app owner, I want to manage permissions within my app, so that I can define granular access controls.

#### Acceptance Criteria

1. WHEN viewing app details, THE Frontend SHALL display a list of permissions for that app
2. WHEN a user clicks "Create Permission", THE Frontend SHALL show a form with permission code field
3. WHEN a user creates a permission successfully, THE Frontend SHALL add it to the permissions list

### Requirement 4: App User Management

**User Story:** As an app owner, I want to manage users registered to my app, so that I can control access and handle violations.

#### Acceptance Criteria

1. WHEN viewing app details, THE Frontend SHALL display a paginated list of users registered to the app
2. WHEN a user clicks "Ban User", THE Frontend SHALL confirm and ban that user from the app
3. WHEN a user clicks "Unban User", THE Frontend SHALL unban that user
4. WHEN a user clicks "Remove User", THE Frontend SHALL confirm and remove user from the app

### Requirement 5: Webhook Management

**User Story:** As an app owner, I want to manage webhooks for my app, so that I can receive event notifications.

#### Acceptance Criteria

1. WHEN viewing app details, THE Frontend SHALL display a list of webhooks
2. WHEN a user clicks "Create Webhook", THE Frontend SHALL show a form with URL and events selection
3. WHEN a webhook is created, THE Frontend SHALL display the webhook secret once
4. WHEN a user edits a webhook, THE Frontend SHALL allow updating URL, events, and active status
5. WHEN a user deletes a webhook, THE Frontend SHALL confirm and remove it

### Requirement 6: API Key Management

**User Story:** As an app owner, I want to manage API keys for my app, so that I can provide secure access to external services.

#### Acceptance Criteria

1. WHEN viewing app details, THE Frontend SHALL display a list of API keys
2. WHEN a user clicks "Create API Key", THE Frontend SHALL show a form with name, scopes, and expiration
3. WHEN an API key is created, THE Frontend SHALL display the full key once and warn to save it
4. WHEN a user edits an API key, THE Frontend SHALL allow updating name, scopes, and active status
5. WHEN a user revokes an API key, THE Frontend SHALL confirm and revoke it

### Requirement 7: App IP Rules Management

**User Story:** As an app owner, I want to manage IP rules for my app, so that I can control access by IP address.

#### Acceptance Criteria

1. WHEN viewing app details, THE Frontend SHALL display a list of IP rules
2. WHEN a user clicks "Create IP Rule", THE Frontend SHALL show a form with IP, rule type, reason, and expiration
3. WHEN a user deletes an IP rule, THE Frontend SHALL confirm and remove it

### Requirement 8: Connected Apps (OAuth)

**User Story:** As a user, I want to view and manage apps I've authorized via OAuth, so that I can control my data access.

#### Acceptance Criteria

1. WHEN a user navigates to Connected Apps page, THE Frontend SHALL display list of OAuth apps with granted scopes
2. WHEN a user clicks "Revoke Access", THE Frontend SHALL confirm and revoke the app's consent
3. WHEN consent is revoked, THE Frontend SHALL remove the app from the list

### Requirement 9: Admin Panel Router

**User Story:** As a system admin, I want a dedicated admin section, so that I can manage the entire system.

#### Acceptance Criteria

1. WHEN a system admin is logged in, THE Frontend SHALL show Admin link in navigation
2. WHEN navigating to `/admin/*`, THE Frontend SHALL verify user is system admin
3. IF a non-admin user accesses admin routes, THEN THE Frontend SHALL redirect to dashboard
4. WHEN in admin panel, THE Frontend SHALL display admin-specific sidebar navigation

### Requirement 10: Admin User Management

**User Story:** As a system admin, I want to manage all users, so that I can maintain system security and handle issues.

#### Acceptance Criteria

1. WHEN admin navigates to Users page, THE Frontend SHALL display paginated list of all users
2. WHEN admin searches users, THE Frontend SHALL filter by email, active status, or admin status
3. WHEN admin views user details, THE Frontend SHALL show full user info and roles across apps
4. WHEN admin edits a user, THE Frontend SHALL allow updating email, active status, email verified, and admin status
5. WHEN admin deactivates a user, THE Frontend SHALL confirm and deactivate the account
6. WHEN admin activates a user, THE Frontend SHALL activate the account
7. WHEN admin unlocks a user, THE Frontend SHALL unlock the locked account
8. WHEN admin deletes a user, THE Frontend SHALL confirm with warning and delete permanently

### Requirement 11: Admin App Management

**User Story:** As a system admin, I want to manage all apps, so that I can oversee the platform.

#### Acceptance Criteria

1. WHEN admin navigates to Apps page, THE Frontend SHALL display paginated list of all apps
2. WHEN admin views app details, THE Frontend SHALL show full app info including owner
3. WHEN admin edits an app, THE Frontend SHALL allow updating name and code
4. WHEN admin deletes an app, THE Frontend SHALL confirm with warning and delete permanently

### Requirement 12: Admin Audit Logs

**User Story:** As a system admin, I want to view system-wide audit logs, so that I can monitor security events.

#### Acceptance Criteria

1. WHEN admin navigates to Audit Logs page, THE Frontend SHALL display paginated system-wide logs
2. WHEN viewing logs, THE Frontend SHALL show user, action, IP, user agent, and timestamp
3. WHEN admin filters logs, THE Frontend SHALL support filtering by user or action type

### Requirement 13: Admin IP Rules Management

**User Story:** As a system admin, I want to manage global IP rules, so that I can protect the entire system.

#### Acceptance Criteria

1. WHEN admin navigates to IP Rules page, THE Frontend SHALL display all global IP rules
2. WHEN admin creates an IP rule, THE Frontend SHALL show form with IP, rule type, reason, and expiration
3. WHEN admin checks an IP, THE Frontend SHALL show if IP is allowed or blocked
4. WHEN admin deletes an IP rule, THE Frontend SHALL confirm and remove it

### Requirement 14: Admin Bulk Operations

**User Story:** As a system admin, I want to perform bulk operations, so that I can efficiently manage large numbers of users.

#### Acceptance Criteria

1. WHEN admin selects multiple users, THE Frontend SHALL enable bulk actions
2. WHEN admin performs bulk role assignment, THE Frontend SHALL assign role to all selected users
3. WHEN admin exports users, THE Frontend SHALL download user data as JSON/CSV
4. WHEN admin imports users, THE Frontend SHALL upload and process user data file

### Requirement 15: Navigation và UI Updates

**User Story:** As a user, I want clear navigation to all features, so that I can easily access what I need.

#### Acceptance Criteria

1. WHEN user is authenticated, THE Frontend SHALL show sidebar with all available sections
2. WHEN user is system admin, THE Frontend SHALL show Admin section in navigation
3. WHEN in admin panel, THE Frontend SHALL show admin-specific navigation items
4. THE Frontend SHALL maintain consistent UI patterns across all new pages

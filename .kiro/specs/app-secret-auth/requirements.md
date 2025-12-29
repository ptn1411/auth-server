# Requirements Document

## Introduction

Tính năng App Secret Authentication cho phép các ứng dụng (App) xác thực với auth server bằng cặp App ID và Secret Key thay vì sử dụng JWT token của user. Điều này cho phép các ứng dụng backend tự động quản lý roles và permissions mà không cần user intervention.

## Glossary

- **App**: Ứng dụng client đã đăng ký trong hệ thống auth server
- **App_Secret**: Chuỗi bí mật được sinh ngẫu nhiên, dùng để xác thực App
- **App_ID**: UUID định danh duy nhất của App
- **App_Credentials**: Cặp App_ID và App_Secret dùng để xác thực
- **Secret_Hash**: Giá trị hash của App_Secret được lưu trong database
- **Auth_Server**: Hệ thống xác thực và phân quyền trung tâm

## Requirements

### Requirement 1: Sinh App Secret khi tạo App

**User Story:** As an App_Owner, I want the system to generate a secret key when I create an app, so that I can use it for machine-to-machine authentication.

#### Acceptance Criteria

1. WHEN an App is created, THE Auth_Server SHALL generate a cryptographically secure random App_Secret
2. WHEN an App_Secret is generated, THE Auth_Server SHALL return the plain-text secret only once during creation
3. WHEN storing the App_Secret, THE Auth_Server SHALL store only the hashed value using bcrypt
4. THE App_Secret SHALL be at least 32 characters long with alphanumeric and special characters

### Requirement 2: Regenerate App Secret

**User Story:** As an App_Owner, I want to regenerate my app's secret key, so that I can rotate credentials for security purposes.

#### Acceptance Criteria

1. WHEN an App_Owner requests secret regeneration, THE Auth_Server SHALL generate a new App_Secret
2. WHEN a new App_Secret is generated, THE Auth_Server SHALL invalidate the previous secret immediately
3. WHEN regenerating secret, THE Auth_Server SHALL return the new plain-text secret only once
4. IF a non-owner user attempts to regenerate secret, THEN THE Auth_Server SHALL reject the request with 403 Forbidden

### Requirement 3: Xác thực bằng App Credentials

**User Story:** As an App, I want to authenticate using my ID and Secret, so that I can access the API without user intervention.

#### Acceptance Criteria

1. WHEN an App provides valid App_ID and App_Secret, THE Auth_Server SHALL authenticate the request
2. WHEN authentication succeeds, THE Auth_Server SHALL return an access token with app context
3. IF the App_Secret is invalid, THEN THE Auth_Server SHALL reject with 401 Unauthorized
4. IF the App_ID does not exist, THEN THE Auth_Server SHALL reject with 401 Unauthorized
5. THE Auth_Server SHALL use constant-time comparison when verifying App_Secret

### Requirement 4: Quản lý Roles bằng App Credentials

**User Story:** As an authenticated App, I want to manage roles within my app scope, so that I can automate role management.

#### Acceptance Criteria

1. WHEN an authenticated App creates a role, THE Auth_Server SHALL create the role scoped to that App
2. WHEN an authenticated App lists roles, THE Auth_Server SHALL return only roles belonging to that App
3. WHEN an authenticated App updates a role, THE Auth_Server SHALL allow modification only for roles in that App
4. WHEN an authenticated App deletes a role, THE Auth_Server SHALL remove only roles belonging to that App
5. IF an App attempts to access roles of another App, THEN THE Auth_Server SHALL reject with 403 Forbidden

### Requirement 5: Quản lý Permissions bằng App Credentials

**User Story:** As an authenticated App, I want to manage permissions within my app scope, so that I can automate permission management.

#### Acceptance Criteria

1. WHEN an authenticated App creates a permission, THE Auth_Server SHALL create the permission scoped to that App
2. WHEN an authenticated App lists permissions, THE Auth_Server SHALL return only permissions belonging to that App
3. WHEN an authenticated App updates a permission, THE Auth_Server SHALL allow modification only for permissions in that App
4. WHEN an authenticated App deletes a permission, THE Auth_Server SHALL remove only permissions belonging to that App
5. IF an App attempts to access permissions of another App, THEN THE Auth_Server SHALL reject with 403 Forbidden

### Requirement 6: Gán Permissions cho Roles bằng App Credentials

**User Story:** As an authenticated App, I want to assign permissions to roles, so that I can configure access control programmatically.

#### Acceptance Criteria

1. WHEN an authenticated App assigns a permission to a role, THE Auth_Server SHALL verify both belong to the same App
2. WHEN assignment is valid, THE Auth_Server SHALL create the role-permission association
3. IF the role or permission belongs to a different App, THEN THE Auth_Server SHALL reject with 403 Forbidden
4. WHEN an authenticated App removes a permission from a role, THE Auth_Server SHALL remove the association

### Requirement 7: API Endpoints cho App Authentication

**User Story:** As a developer, I want clear API endpoints for app authentication, so that I can integrate my backend services.

#### Acceptance Criteria

1. THE Auth_Server SHALL expose POST /apps/auth endpoint for App credential authentication
2. THE Auth_Server SHALL expose POST /apps/{id}/secret/regenerate endpoint for secret regeneration
3. WHEN using app-authenticated endpoints, THE Auth_Server SHALL accept Bearer token in Authorization header
4. THE Auth_Server SHALL include app_id in the token claims for app-authenticated requests

### Requirement 8: Lấy thông tin User từ Token

**User Story:** As a User, I want to retrieve my profile information using my access token, so that I can display my account details in the application.

#### Acceptance Criteria

1. THE Auth_Server SHALL expose GET /users/me endpoint for retrieving current user information
2. WHEN a valid access token is provided, THE Auth_Server SHALL return the user's profile information
3. WHEN returning user information, THE Auth_Server SHALL include id, email, is_active, email_verified, and created_at
4. THE Auth_Server SHALL NOT return sensitive information like password_hash in the response
5. IF the access token is invalid or expired, THEN THE Auth_Server SHALL reject with 401 Unauthorized
6. IF the user account has been deactivated, THEN THE Auth_Server SHALL reject with 403 Forbidden

### Requirement 9: Bảo mật App Secret

**User Story:** As a security administrator, I want app secrets to be securely managed, so that credentials cannot be compromised.

#### Acceptance Criteria

1. THE Auth_Server SHALL never log or expose App_Secret in plain text after creation
2. THE Auth_Server SHALL use bcrypt with cost factor of at least 10 for hashing App_Secret
3. WHEN an App_Secret verification fails, THE Auth_Server SHALL not reveal whether the App_ID or Secret was incorrect
4. THE Auth_Server SHALL implement rate limiting on the /apps/auth endpoint

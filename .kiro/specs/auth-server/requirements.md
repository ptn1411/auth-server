# Requirements Document

## Introduction

Auth Server trung tâm được xây dựng bằng Rust để quản lý xác thực và phân quyền cho nhiều hệ thống con (App). Server này cung cấp SSO (Single Sign-On), quản lý User, App, Role và Permission theo scope của từng App. Auth Server không xử lý business logic của app con.

## Glossary

- **Auth_Server**: Hệ thống xác thực trung tâm xử lý đăng ký, đăng nhập, và quản lý token
- **User**: Người dùng toàn cục trong hệ thống, được định danh bằng email duy nhất
- **App**: Hệ thống con (client) đăng ký với Auth Server, có code định danh duy nhất
- **Role**: Vai trò được gắn với một App cụ thể, không có role global
- **Permission**: Quyền hạn được gắn với một App cụ thể
- **JWT**: JSON Web Token dùng để xác thực, sử dụng thuật toán RS256
- **Refresh_Token**: Token dùng để lấy access token mới khi token cũ hết hạn
- **RBAC**: Role-Based Access Control - mô hình phân quyền dựa trên vai trò

## Requirements

### Requirement 1: User Registration

**User Story:** As a user, I want to register an account with my email and password, so that I can access the system.

#### Acceptance Criteria

1. WHEN a user submits registration with valid email and password, THE Auth_Server SHALL create a new user account with hashed password using argon2
2. WHEN a user attempts to register with an email that already exists, THE Auth_Server SHALL reject the registration and return an error indicating email is taken
3. WHEN a user submits registration with invalid email format, THE Auth_Server SHALL reject the registration and return a validation error
4. WHEN a user submits registration with password that does not meet requirements, THE Auth_Server SHALL reject the registration and return a validation error
5. THE Auth_Server SHALL never store passwords in plain text

### Requirement 2: User Login

**User Story:** As a user, I want to login with my email and password, so that I can receive a JWT token for authentication.

#### Acceptance Criteria

1. WHEN a user submits valid email and password, THE Auth_Server SHALL verify credentials and return a JWT access token and refresh token
2. WHEN a user submits invalid email or password, THE Auth_Server SHALL reject the login and return an authentication error
3. WHEN a user account is inactive (is_active = false), THE Auth_Server SHALL reject the login
4. THE Auth_Server SHALL generate JWT tokens using RS256 algorithm with proper payload structure containing user_id, apps, roles, and permissions
5. THE Auth_Server SHALL set access token expiry to 15 minutes

### Requirement 3: Token Refresh

**User Story:** As a user, I want to refresh my access token using a refresh token, so that I can maintain my session without re-entering credentials.

#### Acceptance Criteria

1. WHEN a user submits a valid refresh token, THE Auth_Server SHALL generate and return a new access token
2. WHEN a user submits an expired or invalid refresh token, THE Auth_Server SHALL reject the request and return an error
3. THE Auth_Server SHALL include updated roles and permissions in the new access token

### Requirement 4: Password Reset

**User Story:** As a user, I want to reset my password if I forget it, so that I can regain access to my account.

#### Acceptance Criteria

1. WHEN a user requests password reset with a registered email, THE Auth_Server SHALL generate a reset token and initiate the reset process
2. WHEN a user requests password reset with an unregistered email, THE Auth_Server SHALL not reveal whether the email exists (security measure)
3. WHEN a user submits a valid reset token with new password, THE Auth_Server SHALL update the password hash using argon2
4. WHEN a user submits an expired or invalid reset token, THE Auth_Server SHALL reject the password reset

### Requirement 5: App Management

**User Story:** As an administrator, I want to register and manage apps, so that each system can have its own authentication scope.

#### Acceptance Criteria

1. WHEN an administrator creates a new app with unique code and name, THE Auth_Server SHALL register the app in the system
2. WHEN an administrator attempts to create an app with a code that already exists, THE Auth_Server SHALL reject the creation and return an error
3. THE Auth_Server SHALL store app information including id (UUID), code (unique), and name

### Requirement 6: Role Management

**User Story:** As an administrator, I want to create and manage roles for each app, so that users can be assigned appropriate access levels.

#### Acceptance Criteria

1. WHEN an administrator creates a role for a specific app, THE Auth_Server SHALL create the role scoped to that app only
2. WHEN an administrator attempts to create a duplicate role name within the same app, THE Auth_Server SHALL reject the creation
3. THE Auth_Server SHALL not allow global roles - all roles must be scoped to an app
4. THE Auth_Server SHALL store role information including id (UUID), app_id (FK), and name

### Requirement 7: Permission Management

**User Story:** As an administrator, I want to create and manage permissions for each app, so that fine-grained access control can be implemented.

#### Acceptance Criteria

1. WHEN an administrator creates a permission for a specific app, THE Auth_Server SHALL create the permission scoped to that app only
2. WHEN an administrator attempts to create a duplicate permission code within the same app, THE Auth_Server SHALL reject the creation
3. THE Auth_Server SHALL not allow hard-coded permissions - all permissions must be configurable
4. THE Auth_Server SHALL store permission information including id (UUID), app_id (FK), and code

### Requirement 8: User Role Assignment

**User Story:** As an administrator, I want to assign roles to users within specific apps, so that users have appropriate access in each system.

#### Acceptance Criteria

1. WHEN an administrator assigns a role to a user for a specific app, THE Auth_Server SHALL create the user-app-role association
2. WHEN an administrator attempts to assign a non-existent role or to a non-existent user, THE Auth_Server SHALL reject the assignment
3. THE Auth_Server SHALL allow a user to have different roles in different apps
4. THE Auth_Server SHALL store user-app-role associations with user_id, app_id, and role_id

### Requirement 9: Role Permission Assignment

**User Story:** As an administrator, I want to assign permissions to roles, so that role-based access control is properly configured.

#### Acceptance Criteria

1. WHEN an administrator assigns a permission to a role, THE Auth_Server SHALL create the role-permission association
2. WHEN an administrator attempts to assign a permission from a different app to a role, THE Auth_Server SHALL reject the assignment
3. THE Auth_Server SHALL ensure permissions can only be assigned to roles within the same app scope

### Requirement 10: JWT Token Structure

**User Story:** As a client app, I want to receive properly structured JWT tokens, so that I can verify user identity and permissions.

#### Acceptance Criteria

1. THE Auth_Server SHALL generate JWT tokens with payload containing: sub (user_id), apps (object with app codes as keys), and exp (expiration timestamp)
2. THE Auth_Server SHALL include roles array and permissions array for each app in the token payload
3. THE Auth_Server SHALL sign all tokens using RS256 algorithm
4. THE Auth_Server SHALL not use shared secrets for JWT verification - only public/private key pairs

### Requirement 11: JWT Verification Middleware

**User Story:** As a developer, I want middleware that verifies JWT tokens, so that protected endpoints are secured.

#### Acceptance Criteria

1. WHEN a request contains a valid JWT in Authorization header, THE Auth_Server SHALL verify the token and inject claims into request context
2. WHEN a request contains an expired JWT, THE Auth_Server SHALL reject the request with appropriate error
3. WHEN a request contains an invalid or malformed JWT, THE Auth_Server SHALL reject the request with appropriate error
4. THE Auth_Server SHALL check token expiry on every protected request

### Requirement 12: RBAC Authorization

**User Story:** As a client app, I want to check user permissions within my app scope, so that I can enforce access control.

#### Acceptance Criteria

1. THE Auth_Server SHALL provide a mechanism to check if a user has a specific permission within an app scope: can(app_code, permission) => boolean
2. THE Auth_Server SHALL ensure an app can only read permissions within its own scope
3. THE Auth_Server SHALL prevent cross-app permission access - app A cannot use permissions from app B

### Requirement 13: Database Schema

**User Story:** As a developer, I want a well-defined database schema, so that data integrity is maintained.

#### Acceptance Criteria

1. THE Auth_Server SHALL use MySQL as the database
2. THE Auth_Server SHALL implement the users table with: id (UUID, PK), email (unique), password_hash, is_active, email_verified, created_at
3. THE Auth_Server SHALL implement the apps table with: id (UUID, PK), code (unique), name
4. THE Auth_Server SHALL implement the roles table with: id (UUID, PK), app_id (FK), name
5. THE Auth_Server SHALL implement the permissions table with: id (UUID, PK), app_id (FK), code
6. THE Auth_Server SHALL implement the user_app_roles table with: user_id, app_id, role_id
7. THE Auth_Server SHALL implement the role_permissions table with: role_id, permission_id

### Requirement 14: API Endpoints

**User Story:** As a client, I want RESTful API endpoints, so that I can integrate with the Auth Server.

#### Acceptance Criteria

1. THE Auth_Server SHALL expose POST /auth/register for user registration
2. THE Auth_Server SHALL expose POST /auth/login for user authentication
3. THE Auth_Server SHALL expose POST /auth/refresh for token refresh
4. THE Auth_Server SHALL expose POST /auth/forgot-password for initiating password reset
5. THE Auth_Server SHALL expose POST /auth/reset-password for completing password reset
6. THE Auth_Server SHALL expose POST /apps for creating new apps
7. THE Auth_Server SHALL expose POST /apps/{app_id}/roles for creating roles
8. THE Auth_Server SHALL expose POST /apps/{app_id}/permissions for creating permissions
9. THE Auth_Server SHALL expose POST /apps/{app_id}/users/{user_id}/roles for assigning roles to users
10. THE Auth_Server SHALL communicate using HTTP(S) with JSON format

### Requirement 15: Technology Stack

**User Story:** As a developer, I want to use specified technologies, so that the system meets technical requirements.

#### Acceptance Criteria

1. THE Auth_Server SHALL be implemented in Rust programming language
2. THE Auth_Server SHALL use axum as the web framework
3. THE Auth_Server SHALL use tokio as the async runtime
4. THE Auth_Server SHALL use sqlx as the ORM for MySQL
5. THE Auth_Server SHALL use argon2 for password hashing
6. THE Auth_Server SHALL use jsonwebtoken crate for JWT operations
7. THE Auth_Server SHALL use uuid crate for UUID generation

# Requirements Document

## Introduction

Frontend application được xây dựng bằng Vite + React + TypeScript để tương tác với Auth Server API. Ứng dụng cung cấp giao diện người dùng cho việc đăng ký, đăng nhập, quản lý token, và quản lý apps/roles/permissions.

## Glossary

- **Auth_Frontend**: Ứng dụng React frontend cho Auth Server
- **User**: Người dùng cuối tương tác với hệ thống
- **Auth_Service**: Module xử lý authentication (login, register, token refresh)
- **API_Client**: Module gọi API đến backend Auth Server
- **Token_Manager**: Module quản lý JWT tokens (lưu trữ, refresh tự động)
- **Protected_Route**: Route yêu cầu user đã đăng nhập
- **Dashboard**: Trang chính sau khi đăng nhập

## Requirements

### Requirement 1: User Registration

**User Story:** As a user, I want to register a new account, so that I can access the system.

#### Acceptance Criteria

1. WHEN a user navigates to the registration page, THE Auth_Frontend SHALL display a registration form with email and password fields
2. WHEN a user submits valid registration data, THE Auth_Frontend SHALL send a POST request to /auth/register and display success message
3. WHEN the backend returns email validation error, THE Auth_Frontend SHALL display "Invalid email format" error message
4. WHEN the backend returns weak password error, THE Auth_Frontend SHALL display "Password does not meet requirements" error message
5. WHEN the backend returns email exists error, THE Auth_Frontend SHALL display "Email already exists" error message
6. WHEN registration is successful, THE Auth_Frontend SHALL redirect user to login page

### Requirement 2: User Login

**User Story:** As a user, I want to login to my account, so that I can access protected features.

#### Acceptance Criteria

1. WHEN a user navigates to the login page, THE Auth_Frontend SHALL display a login form with email and password fields
2. WHEN a user submits valid credentials, THE Auth_Frontend SHALL send a POST request to /auth/login
3. WHEN login is successful, THE Token_Manager SHALL store access_token and refresh_token in localStorage
4. WHEN login is successful, THE Auth_Frontend SHALL redirect user to dashboard
5. WHEN the backend returns invalid credentials error, THE Auth_Frontend SHALL display "Invalid email or password" error message
6. WHEN the backend returns user inactive error, THE Auth_Frontend SHALL display "Account is inactive" error message

### Requirement 3: Token Management

**User Story:** As a user, I want my session to be maintained automatically, so that I don't have to login repeatedly.

#### Acceptance Criteria

1. THE Token_Manager SHALL store tokens securely in localStorage
2. WHEN access_token is about to expire (within 1 minute), THE Token_Manager SHALL automatically refresh the token using /auth/refresh
3. WHEN refresh token fails, THE Auth_Frontend SHALL redirect user to login page
4. WHEN user logs out, THE Token_Manager SHALL clear all stored tokens
5. THE API_Client SHALL automatically attach access_token to all protected API requests

### Requirement 4: Password Reset

**User Story:** As a user, I want to reset my password if I forget it, so that I can regain access to my account.

#### Acceptance Criteria

1. WHEN a user clicks "Forgot Password" on login page, THE Auth_Frontend SHALL navigate to forgot password page
2. WHEN a user submits email for password reset, THE Auth_Frontend SHALL send POST request to /auth/forgot-password
3. THE Auth_Frontend SHALL display "If the email exists, a password reset link has been sent" message
4. WHEN a user navigates to reset password page with token, THE Auth_Frontend SHALL display new password form
5. WHEN a user submits new password, THE Auth_Frontend SHALL send POST request to /auth/reset-password
6. WHEN password reset is successful, THE Auth_Frontend SHALL redirect to login page with success message

### Requirement 5: App Management

**User Story:** As an authenticated user, I want to create and view apps, so that I can manage my applications.

#### Acceptance Criteria

1. WHEN an authenticated user navigates to apps page, THE Auth_Frontend SHALL display list of apps
2. WHEN a user clicks "Create App", THE Auth_Frontend SHALL display a form with code and name fields
3. WHEN a user submits valid app data, THE Auth_Frontend SHALL send POST request to /apps
4. WHEN app creation is successful, THE Auth_Frontend SHALL add the new app to the list
5. WHEN the backend returns app code exists error, THE Auth_Frontend SHALL display "App code already exists" error message

### Requirement 6: Role Management

**User Story:** As an authenticated user, I want to create roles for my apps, so that I can define access levels.

#### Acceptance Criteria

1. WHEN a user selects an app, THE Auth_Frontend SHALL display roles for that app
2. WHEN a user clicks "Create Role", THE Auth_Frontend SHALL display a form with role name field
3. WHEN a user submits valid role data, THE Auth_Frontend SHALL send POST request to /apps/{app_id}/roles
4. WHEN role creation is successful, THE Auth_Frontend SHALL add the new role to the list
5. WHEN the backend returns role name exists error, THE Auth_Frontend SHALL display "Role name already exists" error message

### Requirement 7: Permission Management

**User Story:** As an authenticated user, I want to create permissions for my apps, so that I can define granular access control.

#### Acceptance Criteria

1. WHEN a user selects an app, THE Auth_Frontend SHALL display permissions for that app
2. WHEN a user clicks "Create Permission", THE Auth_Frontend SHALL display a form with permission code field
3. WHEN a user submits valid permission data, THE Auth_Frontend SHALL send POST request to /apps/{app_id}/permissions
4. WHEN permission creation is successful, THE Auth_Frontend SHALL add the new permission to the list
5. WHEN the backend returns permission code exists error, THE Auth_Frontend SHALL display "Permission code already exists" error message

### Requirement 8: User Role Assignment

**User Story:** As an authenticated user, I want to assign roles to users, so that I can control their access.

#### Acceptance Criteria

1. WHEN a user navigates to user management for an app, THE Auth_Frontend SHALL display user list
2. WHEN a user clicks "Assign Role", THE Auth_Frontend SHALL display role selection dropdown
3. WHEN a user selects a role and confirms, THE Auth_Frontend SHALL send POST request to /apps/{app_id}/users/{user_id}/roles
4. WHEN role assignment is successful, THE Auth_Frontend SHALL update the user's role display

### Requirement 9: Protected Routes

**User Story:** As a system, I want to protect certain routes, so that only authenticated users can access them.

#### Acceptance Criteria

1. WHEN an unauthenticated user tries to access a protected route, THE Auth_Frontend SHALL redirect to login page
2. WHEN an authenticated user accesses a protected route, THE Auth_Frontend SHALL render the protected content
3. THE Auth_Frontend SHALL check token validity on each protected route access

### Requirement 10: UI/UX

**User Story:** As a user, I want a clean and responsive interface, so that I can use the application comfortably.

#### Acceptance Criteria

1. THE Auth_Frontend SHALL use a modern UI component library (e.g., Tailwind CSS, shadcn/ui)
2. THE Auth_Frontend SHALL be responsive and work on mobile devices
3. THE Auth_Frontend SHALL display loading states during API calls
4. THE Auth_Frontend SHALL display toast notifications for success/error messages
5. THE Auth_Frontend SHALL have a consistent navigation layout

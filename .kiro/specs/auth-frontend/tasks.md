# Implementation Plan: Auth Frontend

## Overview

Triển khai frontend React với Vite cho Auth Server. Sử dụng TypeScript, React Router, React Query, Tailwind CSS và shadcn/ui.

## Tasks

- [ ] 1. Setup project và cấu hình cơ bản
  - [ ] 1.1 Khởi tạo Vite React TypeScript project trong thư mục `frontend/`
    - Chạy `npm create vite@latest frontend -- --template react-ts`
    - Cài đặt dependencies: react-router-dom, @tanstack/react-query, axios, jwt-decode
    - _Requirements: 10.1_
  - [ ] 1.2 Cấu hình Tailwind CSS và shadcn/ui
    - Cài đặt và cấu hình Tailwind CSS
    - Cài đặt shadcn/ui CLI và init
    - Thêm các components cơ bản: Button, Input, Card, Form, Toast
    - _Requirements: 10.1, 10.2_
  - [ ] 1.3 Tạo cấu trúc thư mục theo design
    - Tạo folders: components/, pages/, services/, hooks/, contexts/, types/, lib/
    - _Requirements: N/A_

- [ ] 2. Implement Types và API Client
  - [ ] 2.1 Tạo TypeScript interfaces trong `types/index.ts`
    - Định nghĩa User, TokenResponse, LoginRequest, RegisterRequest, etc.
    - Định nghĩa App, Role, Permission types
    - Định nghĩa ApiError type
    - _Requirements: 1.1-1.6, 2.1-2.6, 5.1-5.5, 6.1-6.5, 7.1-7.5_
  - [ ] 2.2 Implement API Client với Axios interceptors trong `services/api.ts`
    - Tạo axios instance với base URL từ env
    - Implement request interceptor để attach token
    - Implement response interceptor để handle token refresh
    - _Requirements: 3.2, 3.3, 3.5_
  - [ ]* 2.3 Write property test cho API Request Token Attachment
    - **Property 5: API Request Token Attachment**
    - **Validates: Requirements 3.5**

- [ ] 3. Implement Auth Context và Token Manager
  - [ ] 3.1 Tạo AuthContext trong `contexts/AuthContext.tsx`
    - Implement AuthProvider với state management
    - Implement login, register, logout, refreshToken functions
    - Check token validity on mount
    - _Requirements: 2.3, 2.4, 3.1, 3.4_
  - [ ] 3.2 Tạo useAuth hook trong `hooks/useAuth.ts`
    - Export useAuth hook từ AuthContext
    - _Requirements: 2.3, 3.4_
  - [ ]* 3.3 Write property test cho Token Storage Round-Trip
    - **Property 2: Token Storage Round-Trip**
    - **Validates: Requirements 2.3, 3.4**
  - [ ]* 3.4 Write property test cho Logout Clears All Tokens
    - **Property 3: Logout Clears All Tokens**
    - **Validates: Requirements 3.4**

- [ ] 4. Implement Error Handling
  - [ ] 4.1 Tạo error utilities trong `lib/errors.ts`
    - Implement errorMessages mapping
    - Implement getErrorMessage function
    - _Requirements: 1.3, 1.4, 1.5, 2.5, 2.6, 5.5, 6.5, 7.5_
  - [ ]* 4.2 Write property test cho Error Message Mapping
    - **Property 1: API Error to UI Message Mapping**
    - **Validates: Requirements 1.3, 1.4, 1.5, 2.5, 2.6, 5.5, 6.5, 7.5**

- [ ] 5. Implement Auth Services
  - [ ] 5.1 Tạo auth service trong `services/auth.ts`
    - Implement login, register, refresh, forgotPassword, resetPassword functions
    - _Requirements: 1.2, 2.2, 4.2, 4.5_
  - [ ]* 5.2 Write property test cho Form Submission Triggers Correct API Call (auth)
    - **Property 7: Form Submission Triggers Correct API Call**
    - **Validates: Requirements 1.2, 2.2, 4.2, 4.5**

- [ ] 6. Checkpoint - Ensure core services work
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Implement Layout Components
  - [ ] 7.1 Tạo MainLayout trong `components/layout/MainLayout.tsx`
    - Header với navigation và user menu
    - Sidebar cho authenticated pages
    - Responsive design
    - _Requirements: 10.2, 10.5_
  - [ ] 7.2 Tạo ProtectedRoute component
    - Check authentication status
    - Redirect to login if not authenticated
    - _Requirements: 9.1, 9.2_
  - [ ]* 7.3 Write property test cho Protected Route Access Control
    - **Property 4: Protected Route Access Control**
    - **Validates: Requirements 9.1, 9.2**

- [ ] 8. Implement Auth Pages
  - [ ] 8.1 Tạo LoginPage trong `pages/auth/LoginPage.tsx`
    - Login form với email và password
    - Error handling và loading states
    - Link to register và forgot password
    - _Requirements: 2.1, 2.4, 2.5, 2.6, 10.3, 10.4_
  - [ ] 8.2 Tạo RegisterPage trong `pages/auth/RegisterPage.tsx`
    - Register form với email và password
    - Error handling và loading states
    - Redirect to login on success
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 10.3, 10.4_
  - [ ] 8.3 Tạo ForgotPasswordPage trong `pages/auth/ForgotPasswordPage.tsx`
    - Email input form
    - Success message display
    - _Requirements: 4.1, 4.2, 4.3, 10.3, 10.4_
  - [ ] 8.4 Tạo ResetPasswordPage trong `pages/auth/ResetPasswordPage.tsx`
    - New password form với token from URL
    - Redirect to login on success
    - _Requirements: 4.4, 4.5, 4.6, 10.3, 10.4_

- [ ] 9. Implement App Management
  - [ ] 9.1 Tạo apps service trong `services/apps.ts`
    - Implement getApps, createApp functions
    - _Requirements: 5.3_
  - [ ] 9.2 Tạo AppsPage trong `pages/apps/AppsPage.tsx`
    - Display list of apps
    - Create app form/modal
    - Error handling
    - _Requirements: 5.1, 5.2, 5.4, 5.5, 10.3, 10.4_
  - [ ]* 9.3 Write property test cho Successful Creation Updates List (apps)
    - **Property 6: Successful Creation Updates List**
    - **Validates: Requirements 5.4**

- [ ] 10. Implement Role và Permission Management
  - [ ] 10.1 Tạo roles service trong `services/roles.ts`
    - Implement getRoles, createRole functions
    - _Requirements: 6.3_
  - [ ] 10.2 Tạo permissions service trong `services/permissions.ts`
    - Implement getPermissions, createPermission functions
    - _Requirements: 7.3_
  - [ ] 10.3 Tạo AppDetailPage trong `pages/apps/AppDetailPage.tsx`
    - Display roles và permissions cho selected app
    - Create role/permission forms
    - Error handling
    - _Requirements: 6.1, 6.2, 6.4, 6.5, 7.1, 7.2, 7.4, 7.5, 10.3, 10.4_

- [ ] 11. Implement User Role Assignment
  - [ ] 11.1 Tạo user-roles service trong `services/userRoles.ts`
    - Implement assignRole function
    - _Requirements: 8.3_
  - [ ] 11.2 Thêm user management section vào AppDetailPage
    - Display users với roles
    - Assign role dropdown
    - _Requirements: 8.1, 8.2, 8.4_

- [ ] 12. Setup Routing
  - [ ] 12.1 Cấu hình React Router trong `App.tsx`
    - Public routes: /login, /register, /forgot-password, /reset-password
    - Protected routes: /dashboard, /apps, /apps/:id
    - 404 page
    - _Requirements: 9.1, 9.2_

- [ ] 13. Implement Dashboard
  - [ ] 13.1 Tạo DashboardPage trong `pages/dashboard/DashboardPage.tsx`
    - Welcome message
    - Quick links to apps
    - _Requirements: 2.4_

- [ ] 14. Final Integration và Polish
  - [ ] 14.1 Tạo Toast notifications setup
    - Configure toast provider
    - Use toasts for success/error messages
    - _Requirements: 10.4_
  - [ ] 14.2 Add loading states và skeletons
    - Loading spinners cho API calls
    - Skeleton loaders cho lists
    - _Requirements: 10.3_

- [ ] 15. Final Checkpoint
  - Ensure all tests pass, ask the user if questions arise.
  - Verify all pages render correctly
  - Test authentication flow end-to-end

## Notes

- Tasks marked with `*` are optional property-based tests
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases

# Implementation Plan: Auth Frontend

## Overview

Triển khai Auth Frontend sử dụng React + Vite + shadcn/ui + Zustand, tích hợp với SDK TypeScript có sẵn. Implementation sẽ được chia thành các phase: setup project, core infrastructure, auth flows, và protected features.

## Tasks

- [x] 1. Project Setup và Core Infrastructure
  - [x] 1.1 Khởi tạo Vite React TypeScript project trong thư mục `frontend/`
    - Chạy `npm create vite@latest frontend -- --template react-ts`
    - Cài đặt dependencies: `zustand`, `react-router-dom`, `zod`, `react-hook-form`, `@hookform/resolvers`
    - _Requirements: 13.1_

  - [x] 1.2 Cài đặt và cấu hình Tailwind CSS + shadcn/ui
    - Cài đặt Tailwind CSS
    - Khởi tạo shadcn/ui với `npx shadcn@latest init`
    - Thêm các components cần thiết: button, input, card, form, toast, dialog, dropdown-menu, avatar, badge, table, tabs
    - _Requirements: 13.1, 13.2_

  - [x] 1.3 Cấu hình SDK và Auth Client
    - Copy hoặc link SDK từ `../sdk`
    - Tạo `src/lib/auth-client.ts` để khởi tạo AuthServerClient instance
    - Cấu hình environment variables cho API URL
    - _Requirements: 2.2, 2.5_

  - [x] 1.4 Tạo Zustand Auth Store
    - Tạo `src/stores/authStore.ts` với state và actions
    - Implement persist middleware cho token storage
    - Implement login, register, logout, completeMfa, refreshUser actions
    - _Requirements: 2.5, 12.3_

  - [x] 1.5 Tạo Theme Store và Dark Mode
    - Tạo `src/stores/themeStore.ts`
    - Implement theme toggle (light/dark/system)
    - Apply theme class to document
    - _Requirements: 13.4_

- [x] 2. Layout và Navigation
  - [x] 2.1 Tạo Layout Components
    - Tạo `src/components/layout/Header.tsx` với user menu và theme toggle
    - Tạo `src/components/layout/Sidebar.tsx` với navigation links
    - Tạo `src/components/layout/Layout.tsx` wrapper component
    - _Requirements: 10.1, 13.2_

  - [x] 2.2 Cấu hình React Router
    - Tạo `src/App.tsx` với route definitions
    - Tạo `src/components/ProtectedRoute.tsx` cho route protection
    - Setup routes cho tất cả pages
    - _Requirements: 12.1, 12.2_

- [x] 3. Authentication Pages
  - [x] 3.1 Tạo Login Page
    - Tạo `src/components/auth/LoginForm.tsx` với email/password fields
    - Tạo `src/pages/LoginPage.tsx`
    - Implement form validation với Zod
    - Handle login success và MFA redirect
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [x] 3.2 Tạo Register Page
    - Tạo `src/components/auth/RegisterForm.tsx` với validation
    - Tạo `src/pages/RegisterPage.tsx`
    - Implement password strength indicator
    - Handle registration success
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [x] 3.3 Tạo MFA Verification Page
    - Tạo `src/components/auth/MfaForm.tsx` với 6-digit input
    - Tạo `src/pages/MfaPage.tsx`
    - Support backup code input
    - Handle MFA verification
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [x] 3.4 Tạo Password Recovery Pages
    - Tạo `src/components/auth/ForgotPasswordForm.tsx`
    - Tạo `src/components/auth/ResetPasswordForm.tsx`
    - Tạo `src/pages/ForgotPasswordPage.tsx`
    - Tạo `src/pages/ResetPasswordPage.tsx`
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [x] 3.5 Tạo Email Verification Page
    - Tạo `src/pages/VerifyEmailPage.tsx`
    - Handle token từ URL và verify
    - _Requirements: 1.2_

- [x] 4. Checkpoint - Auth Flows
  - Ensure all auth pages render correctly
  - Test login/register flows với mock hoặc real API
  - Ask user if questions arise

- [x] 5. Dashboard và Profile
  - [x] 5.1 Tạo Dashboard Page
    - Tạo `src/components/dashboard/DashboardStats.tsx`
    - Tạo `src/components/dashboard/RecentActivity.tsx`
    - Tạo `src/pages/DashboardPage.tsx`
    - Display user summary và quick actions
    - _Requirements: 10.1, 10.2, 10.3, 10.4_

  - [x] 5.2 Tạo Profile Page
    - Tạo `src/components/profile/ProfileCard.tsx`
    - Tạo `src/components/profile/UpdateProfileForm.tsx`
    - Tạo `src/components/profile/ChangePasswordForm.tsx`
    - Tạo `src/pages/ProfilePage.tsx`
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 6. Security Features
  - [x] 6.1 Tạo Sessions Page
    - Tạo `src/components/security/SessionList.tsx`
    - Tạo `src/pages/SessionsPage.tsx`
    - Display sessions với device info
    - Implement revoke session actions
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [x] 6.2 Tạo MFA Setup Components
    - Tạo `src/components/security/MfaSetup.tsx` với QR code
    - Tạo `src/components/security/BackupCodes.tsx`
    - Implement TOTP setup flow
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [x] 6.3 Tạo Passkey Management
    - Tạo `src/hooks/useWebAuthn.ts` cho WebAuthn API
    - Tạo `src/components/security/PasskeyList.tsx`
    - Tạo `src/components/security/PasskeyRegister.tsx`
    - Implement passkey registration và management
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

  - [x] 6.4 Tạo Security Page
    - Tạo `src/pages/SecurityPage.tsx`
    - Combine MFA setup và Passkey management
    - _Requirements: 7.1, 8.1_

- [x] 7. Passkey Authentication
  - [x] 7.1 Implement Passkey Login
    - Update LoginForm với passkey option
    - Implement WebAuthn authentication flow
    - Handle passkey login success
    - _Requirements: 9.1, 9.2, 9.3_

- [x] 8. Audit Logs
  - [x] 8.1 Tạo Audit Logs Page
    - Tạo `src/pages/AuditLogsPage.tsx`
    - Display paginated audit logs
    - Show action, IP, user agent, timestamp
    - Implement pagination
    - _Requirements: 11.1, 11.2, 11.3_

- [ ] 9. Final Checkpoint
  - Ensure all pages render correctly
  - Test all user flows
  - Verify responsive design
  - Ask user if questions arise

## Notes

- SDK được import từ `../sdk` hoặc có thể publish lên npm
- Environment variable `VITE_API_URL` cho API endpoint
- Tất cả forms sử dụng react-hook-form + zod validation
- Toast notifications cho success/error feedback
- Loading states cho tất cả async operations

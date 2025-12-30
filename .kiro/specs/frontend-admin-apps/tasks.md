# Implementation Plan: Frontend Admin & Apps Management

## Overview

Triển khai các tính năng quản lý App cho user và Admin Panel hoàn chỉnh. Implementation sử dụng TypeScript với React + Vite + shadcn/ui + Zustand, tích hợp SDK có sẵn.

## Tasks

- [x] 1. Setup Stores và Shared Components
  - [x] 1.1 Tạo Apps Store
    - Tạo `src/stores/appsStore.ts` với state và actions cho app management
    - Implement fetchApps, createApp, regenerateSecret
    - Implement role, permission, user, webhook, API key, IP rule actions
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3_

  - [x] 1.2 Tạo Admin Store
    - Tạo `src/stores/adminStore.ts` với state và actions cho admin features
    - Implement user management actions (fetch, search, update, delete, activate, deactivate, unlock)
    - Implement app management actions
    - Implement audit logs và IP rules actions
    - Implement bulk operations (export, import, bulk assign role)
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8, 11.1, 11.2, 11.3, 11.4, 12.1, 12.2, 12.3, 13.1, 13.2, 13.3, 13.4, 14.1, 14.2, 14.3, 14.4_

  - [x] 1.3 Tạo Connected Apps Store
    - Tạo `src/stores/connectedAppsStore.ts`
    - Implement fetchConnectedApps và revokeConsent
    - _Requirements: 8.1, 8.2, 8.3_

  - [x] 1.4 Tạo Shared Components
    - Tạo `src/components/shared/ConfirmDialog.tsx` cho confirmation dialogs
    - Tạo `src/components/shared/Pagination.tsx` cho paginated lists
    - Tạo `src/components/shared/SecretDisplay.tsx` cho hiển thị secrets một lần
    - _Requirements: 1.3, 1.4, 4.2, 4.4, 5.3, 6.3_

- [x] 2. App Management Components
  - [x] 2.1 Tạo App List Components
    - Tạo `src/components/apps/AppList.tsx`
    - Tạo `src/components/apps/AppCard.tsx`
    - Tạo `src/components/apps/CreateAppDialog.tsx` với form validation
    - Tạo `src/components/apps/AppSecretDialog.tsx` cho hiển thị secret
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [ ]* 2.2 Write property test for list rendering
    - **Property 1: List Rendering Completeness**
    - **Validates: Requirements 1.1**

  - [x] 2.3 Tạo Role Management Components
    - Tạo `src/components/apps/RoleList.tsx`
    - Tạo `src/components/apps/CreateRoleDialog.tsx`
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 2.4 Tạo Permission Management Components
    - Tạo `src/components/apps/PermissionList.tsx`
    - Tạo `src/components/apps/CreatePermissionDialog.tsx`
    - _Requirements: 3.1, 3.2, 3.3_

  - [x] 2.5 Tạo App User Management Components
    - Tạo `src/components/apps/AppUserList.tsx` với pagination
    - Tạo `src/components/apps/UserRoleDialog.tsx` cho assign/remove roles
    - _Requirements: 2.4, 4.1, 4.2, 4.3, 4.4_

  - [x] 2.6 Tạo Webhook Management Components
    - Tạo `src/components/apps/WebhookList.tsx`
    - Tạo `src/components/apps/WebhookDialog.tsx` cho create/edit
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 2.7 Tạo API Key Management Components
    - Tạo `src/components/apps/ApiKeyList.tsx`
    - Tạo `src/components/apps/ApiKeyDialog.tsx` cho create/edit
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [x] 2.8 Tạo IP Rule Management Components
    - Tạo `src/components/apps/IpRuleList.tsx`
    - Tạo `src/components/apps/IpRuleDialog.tsx`
    - _Requirements: 7.1, 7.2, 7.3_

- [x] 3. App Pages
  - [x] 3.1 Tạo Apps Page
    - Tạo `src/pages/AppsPage.tsx`
    - Display list of user's apps với create button
    - _Requirements: 1.1, 1.2_

  - [x] 3.2 Tạo App Detail Page
    - Tạo `src/pages/AppDetailPage.tsx` với tabs
    - Implement tabs: overview, users, roles, permissions, webhooks, api-keys, ip-rules
    - _Requirements: 1.5, 2.1, 2.4, 3.1, 4.1, 5.1, 6.1, 7.1_

- [x] 4. Connected Apps Feature
  - [x] 4.1 Tạo Connected Apps Components
    - Tạo `src/components/connected-apps/ConnectedAppList.tsx`
    - Tạo `src/components/connected-apps/ConnectedAppCard.tsx`
    - _Requirements: 8.1, 8.2, 8.3_

  - [x] 4.2 Tạo Connected Apps Page
    - Tạo `src/pages/ConnectedAppsPage.tsx`
    - Display OAuth apps với revoke option
    - _Requirements: 8.1, 8.2, 8.3_

  - [ ]* 4.3 Write property test for state update after mutation
    - **Property 2: State Update After Mutation**
    - **Validates: Requirements 8.3**

- [ ] 5. Checkpoint - User Features
  - Ensure all user app management features work
  - Test create app, manage roles/permissions, webhooks, API keys
  - Ask user if questions arise

- [x] 6. Admin Layout và Protection
  - [x] 6.1 Tạo Admin Protected Route
    - Tạo `src/components/admin/AdminProtectedRoute.tsx`
    - Check is_system_admin và redirect nếu không phải admin
    - _Requirements: 9.2, 9.3_

  - [ ]* 6.2 Write property test for admin access control
    - **Property 3: Admin Access Control**
    - **Validates: Requirements 9.1, 9.2, 9.3, 15.2**

  - [x] 6.3 Tạo Admin Layout Components
    - Tạo `src/components/admin/AdminLayout.tsx`
    - Tạo `src/components/admin/AdminSidebar.tsx` với admin navigation
    - _Requirements: 9.4, 15.3_

- [x] 7. Admin User Management
  - [x] 7.1 Tạo User Table Components
    - Tạo `src/components/admin/users/UserTable.tsx` với pagination
    - Tạo `src/components/admin/users/UserSearchForm.tsx`
    - Tạo `src/components/admin/users/BulkActionsBar.tsx`
    - _Requirements: 10.1, 10.2, 14.1_

  - [ ]* 7.2 Write property test for pagination
    - **Property 4: Pagination Consistency**
    - **Validates: Requirements 10.1**

  - [ ]* 7.3 Write property test for search filter
    - **Property 5: Search Filter Accuracy**
    - **Validates: Requirements 10.2**

  - [x] 7.4 Tạo User Detail Components
    - Tạo `src/components/admin/users/UserDetailCard.tsx`
    - Tạo `src/components/admin/users/EditUserDialog.tsx`
    - Tạo `src/components/admin/users/UserRolesCard.tsx`
    - _Requirements: 10.3, 10.4_

  - [x] 7.5 Tạo Admin Users Pages
    - Tạo `src/pages/admin/AdminUsersPage.tsx`
    - Tạo `src/pages/admin/AdminUserDetailPage.tsx`
    - Implement user actions: edit, activate, deactivate, unlock, delete
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8_

- [x] 8. Admin App Management
  - [x] 8.1 Tạo Admin App Components
    - Tạo `src/components/admin/apps/AdminAppTable.tsx`
    - Tạo `src/components/admin/apps/AdminAppDetailCard.tsx`
    - Tạo `src/components/admin/apps/EditAppDialog.tsx`
    - _Requirements: 11.1, 11.2, 11.3, 11.4_

  - [x] 8.2 Tạo Admin Apps Pages
    - Tạo `src/pages/admin/AdminAppsPage.tsx`
    - Tạo `src/pages/admin/AdminAppDetailPage.tsx`
    - _Requirements: 11.1, 11.2, 11.3, 11.4_

- [x] 9. Admin Audit Logs
  - [x] 9.1 Tạo Audit Log Components
    - Tạo `src/components/admin/audit/AuditLogTable.tsx`
    - Tạo `src/components/admin/audit/AuditLogFilters.tsx`
    - _Requirements: 12.1, 12.2, 12.3_

  - [ ]* 9.2 Write property test for audit log display
    - **Property 6: Audit Log Display Completeness**
    - **Validates: Requirements 12.2**

  - [x] 9.3 Tạo Admin Audit Logs Page
    - Tạo `src/pages/admin/AdminAuditLogsPage.tsx`
    - _Requirements: 12.1, 12.2, 12.3_

- [x] 10. Admin IP Rules
  - [x] 10.1 Tạo IP Rule Components
    - Tạo `src/components/admin/ip-rules/IpRuleTable.tsx`
    - Tạo `src/components/admin/ip-rules/CreateIpRuleDialog.tsx`
    - Tạo `src/components/admin/ip-rules/IpCheckForm.tsx`
    - _Requirements: 13.1, 13.2, 13.3, 13.4_

  - [ ]* 10.2 Write property test for IP check
    - **Property 8: IP Check Result Display**
    - **Validates: Requirements 13.3**

  - [x] 10.3 Tạo Admin IP Rules Page
    - Tạo `src/pages/admin/AdminIpRulesPage.tsx`
    - _Requirements: 13.1, 13.2, 13.3, 13.4_

- [x] 11. Admin Bulk Operations
  - [x] 11.1 Implement Bulk Selection
    - Update UserTable với checkbox selection
    - Implement select all functionality
    - _Requirements: 14.1_

  - [ ]* 11.2 Write property test for bulk selection
    - **Property 7: Bulk Selection State**
    - **Validates: Requirements 14.1**

  - [x] 11.3 Implement Bulk Actions
    - Implement bulk role assignment dialog
    - Implement export users (JSON download)
    - Implement import users (file upload)
    - _Requirements: 14.2, 14.3, 14.4_

- [x] 12. Admin Dashboard
  - [x] 12.1 Tạo Admin Dashboard Page
    - Tạo `src/pages/admin/AdminDashboardPage.tsx`
    - Display system stats và quick links
    - _Requirements: 9.1_

- [x] 13. Navigation Updates
  - [x] 13.1 Update Sidebar Navigation
    - Update `src/components/layout/Sidebar.tsx` với Apps và Connected Apps links
    - Add Admin section cho system admins
    - _Requirements: 15.1, 15.2_

  - [x] 13.2 Update Router Configuration
    - Update `src/App.tsx` với new routes
    - Add admin routes với AdminProtectedRoute
    - Update `src/pages/index.ts` exports
    - _Requirements: 9.1, 9.2, 15.1_

- [x] 14. Update Auth Client Exports
  - [x] 14.1 Update auth-client.ts
    - Export all new types từ SDK
    - _Requirements: 1.1, 8.1, 10.1, 11.1_

- [ ] 15. Final Checkpoint
  - Ensure all pages render correctly
  - Test all user flows
  - Test all admin flows
  - Verify responsive design
  - Ask user if questions arise

## Notes

- Tasks marked with `*` are optional property-based tests
- All forms use react-hook-form + zod validation
- Toast notifications for success/error feedback
- Loading states for all async operations
- Confirmation dialogs for destructive actions
- SDK types are imported from `auth-server-sdk`

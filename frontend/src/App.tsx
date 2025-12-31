import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import { Toaster } from '@/components/ui/sonner';
import { Layout } from '@/components/layout';
import { ProtectedRoute } from '@/components/ProtectedRoute';
import { AdminLayout, AdminProtectedRoute } from '@/components/admin';
import {
  LoginPage,
  RegisterPage,
  MfaPage,
  ForgotPasswordPage,
  ResetPasswordPage,
  VerifyEmailPage,
  DashboardPage,
  ProfilePage,
  SessionsPage,
  SecurityPage,
  AuditLogsPage,
  AppsPage,
  AppDetailPage,
  ConnectedAppsPage,
  OAuthClientsPage,
  OAuthAuthorizePage,
  OAuthCallbackPage,
  AdminDashboardPage,
  AdminUsersPage,
  AdminUserDetailPage,
  AdminAppsPage,
  AdminAppDetailPage,
  AdminAuditLogsPage,
  AdminIpRulesPage,
  AdminScopesPage,
} from '@/pages';

function AppContent() {
  const { initialize, isLoading } = useAuthStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <Routes>
      {/* Public routes wrapped in Layout */}
      <Route element={<Layout />}>
        {/* Auth routes - redirect to dashboard if already authenticated */}
        <Route
          path="/login"
          element={
            <ProtectedRoute requireAuth={false}>
              <LoginPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/register"
          element={
            <ProtectedRoute requireAuth={false}>
              <RegisterPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/mfa"
          element={<MfaPage />}
        />
        <Route
          path="/forgot-password"
          element={
            <ProtectedRoute requireAuth={false}>
              <ForgotPasswordPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/reset-password"
          element={
            <ProtectedRoute requireAuth={false}>
              <ResetPasswordPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/verify-email"
          element={<VerifyEmailPage />}
        />

        {/* Protected routes */}
        <Route
          path="/dashboard"
          element={
            <ProtectedRoute>
              <DashboardPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/profile"
          element={
            <ProtectedRoute>
              <ProfilePage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/sessions"
          element={
            <ProtectedRoute>
              <SessionsPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/security"
          element={
            <ProtectedRoute>
              <SecurityPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/audit-logs"
          element={
            <ProtectedRoute>
              <AuditLogsPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/apps"
          element={
            <ProtectedRoute>
              <AppsPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/apps/:appId"
          element={
            <ProtectedRoute>
              <AppDetailPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/connected-apps"
          element={
            <ProtectedRoute>
              <ConnectedAppsPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/oauth-clients"
          element={
            <ProtectedRoute>
              <OAuthClientsPage />
            </ProtectedRoute>
          }
        />

        {/* OAuth Authorization (public but requires auth) */}
        <Route
          path="/oauth/authorize"
          element={<OAuthAuthorizePage />}
        />

        {/* OAuth Callback (for popup flow) */}
        <Route
          path="/oauth/callback"
          element={<OAuthCallbackPage />}
        />

        {/* Default redirect */}
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        
        {/* 404 - redirect to dashboard */}
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Route>

      {/* Admin routes with AdminLayout */}
      <Route
        path="/admin"
        element={
          <AdminProtectedRoute>
            <AdminLayout />
          </AdminProtectedRoute>
        }
      >
        <Route index element={<AdminDashboardPage />} />
        <Route path="users" element={<AdminUsersPage />} />
        <Route path="users/:userId" element={<AdminUserDetailPage />} />
        <Route path="apps" element={<AdminAppsPage />} />
        <Route path="apps/:appId" element={<AdminAppDetailPage />} />
        <Route path="audit-logs" element={<AdminAuditLogsPage />} />
        <Route path="ip-rules" element={<AdminIpRulesPage />} />
        <Route path="scopes" element={<AdminScopesPage />} />
      </Route>
    </Routes>
  );
}

function App() {
  return (
    <BrowserRouter>
      <AppContent />
      <Toaster />
    </BrowserRouter>
  );
}

export default App;

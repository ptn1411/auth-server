# Design Document: Auth Frontend

## Overview

Auth Frontend là ứng dụng React được xây dựng với Vite, sử dụng shadcn/ui cho UI components và tích hợp SDK TypeScript để giao tiếp với Auth Server. Ứng dụng cung cấp đầy đủ các tính năng xác thực bao gồm đăng ký, đăng nhập, MFA, passkey, và quản lý tài khoản.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Auth Frontend                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Pages     │  │ Components  │  │    Hooks            │  │
│  │  - Login    │  │ - Forms     │  │ - useAuthStore      │  │
│  │  - Register │  │ - Layout    │  │ - useAuthClient     │  │
│  │  - Dashboard│  │ - UI        │  │ - useWebAuthn       │  │
│  │  - Profile  │  │             │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐│
│  │                Zustand Auth Store                       ││
│  │  - User state, tokens, authentication actions           ││
│  │  - Persist middleware for token storage                 ││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐│
│  │                   Auth SDK Client                       ││
│  │  - API calls, token management, error handling          ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   Auth Server   │
                    │      API        │
                    └─────────────────┘
```

## Components and Interfaces

### Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── ui/                    # shadcn/ui components
│   │   ├── layout/
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── Layout.tsx
│   │   ├── auth/
│   │   │   ├── LoginForm.tsx
│   │   │   ├── RegisterForm.tsx
│   │   │   ├── MfaForm.tsx
│   │   │   ├── ForgotPasswordForm.tsx
│   │   │   └── ResetPasswordForm.tsx
│   │   ├── profile/
│   │   │   ├── ProfileCard.tsx
│   │   │   ├── ChangePasswordForm.tsx
│   │   │   └── UpdateProfileForm.tsx
│   │   ├── security/
│   │   │   ├── SessionList.tsx
│   │   │   ├── MfaSetup.tsx
│   │   │   ├── PasskeyList.tsx
│   │   │   └── PasskeyRegister.tsx
│   │   └── dashboard/
│   │       ├── DashboardStats.tsx
│   │       └── RecentActivity.tsx
│   ├── pages/
│   │   ├── LoginPage.tsx
│   │   ├── RegisterPage.tsx
│   │   ├── ForgotPasswordPage.tsx
│   │   ├── ResetPasswordPage.tsx
│   │   ├── VerifyEmailPage.tsx
│   │   ├── DashboardPage.tsx
│   │   ├── ProfilePage.tsx
│   │   ├── SessionsPage.tsx
│   │   ├── SecurityPage.tsx
│   │   └── AuditLogsPage.tsx
│   ├── hooks/
│   │   ├── useAuthClient.ts
│   │   └── useWebAuthn.ts
│   ├── stores/
│   │   ├── authStore.ts
│   │   └── themeStore.ts
│   ├── lib/
│   │   ├── auth-client.ts
│   │   └── utils.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── components.json
```

### Core Interfaces

```typescript
// Zustand Auth Store
interface AuthState {
  user: UserProfile | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  mfaPending: MfaRequiredResponse | null;
}

interface AuthActions {
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  completeMfa: (code: string) => Promise<void>;
  refreshUser: () => Promise<void>;
  setUser: (user: UserProfile | null) => void;
  setLoading: (loading: boolean) => void;
  clearMfaPending: () => void;
}

type AuthStore = AuthState & AuthActions;

// Zustand Store Implementation
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      // State
      user: null,
      isAuthenticated: false,
      isLoading: true,
      mfaPending: null,

      // Actions
      login: async (email, password) => {
        set({ isLoading: true });
        const response = await authClient.login({ email, password });
        if ('mfa_required' in response) {
          set({ mfaPending: response, isLoading: false });
        } else {
          const user = await authClient.getProfile();
          set({ user, isAuthenticated: true, isLoading: false });
        }
      },

      register: async (email, password) => {
        set({ isLoading: true });
        await authClient.register({ email, password });
        set({ isLoading: false });
      },

      logout: async () => {
        await authClient.logout();
        set({ user: null, isAuthenticated: false, mfaPending: null });
      },

      completeMfa: async (code) => {
        const { mfaPending } = get();
        if (!mfaPending) throw new Error('No MFA pending');
        set({ isLoading: true });
        await authClient.completeMfaLogin({ 
          mfa_token: mfaPending.mfa_token, 
          code 
        });
        const user = await authClient.getProfile();
        set({ user, isAuthenticated: true, mfaPending: null, isLoading: false });
      },

      refreshUser: async () => {
        const user = await authClient.getProfile();
        set({ user });
      },

      setUser: (user) => set({ user, isAuthenticated: !!user }),
      setLoading: (isLoading) => set({ isLoading }),
      clearMfaPending: () => set({ mfaPending: null }),
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({ 
        isAuthenticated: state.isAuthenticated 
      }),
    }
  )
);

// Theme Store
interface ThemeState {
  theme: 'light' | 'dark' | 'system';
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      theme: 'system',
      setTheme: (theme) => set({ theme }),
    }),
    { name: 'theme-storage' }
  )
);

// Protected Route Props
interface ProtectedRouteProps {
  children: React.ReactNode;
  requireAuth?: boolean;
}
```

### Component Specifications

#### LoginForm
- Email và password inputs với validation
- Submit handler gọi useAuthStore().login
- Hiển thị loading state và errors
- Link đến register và forgot password

#### RegisterForm
- Email và password inputs với validation
- Password confirmation field
- Password strength indicator
- Submit handler gọi useAuthStore().register

#### MfaForm
- 6-digit code input
- Support backup code input
- Auto-submit khi đủ 6 digits
- Countdown timer cho resend

#### ProfileCard
- Hiển thị user info (email, created_at)
- MFA status badge
- Email verification status
- Edit profile button

#### SessionList
- Table hiển thị sessions
- Device/browser info
- IP address và location
- Revoke button cho mỗi session
- "Revoke all other sessions" button

#### MfaSetup
- QR code display cho TOTP
- Manual secret entry option
- Verification code input
- Backup codes display sau khi setup

#### PasskeyList
- List registered passkeys
- Device name và last used
- Rename và delete actions
- Register new passkey button

## Data Models

### Local Storage Schema

```typescript
// Token storage
interface StoredTokens {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
}

// Theme preference
type Theme = 'light' | 'dark' | 'system';
```

### Form Validation Schemas (Zod)

```typescript
// Login schema
const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(1, 'Password is required'),
});

// Register schema
const registerSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string()
    .min(8, 'Password must be at least 8 characters')
    .regex(/[A-Z]/, 'Password must contain uppercase letter')
    .regex(/[a-z]/, 'Password must contain lowercase letter')
    .regex(/[0-9]/, 'Password must contain number'),
  confirmPassword: z.string(),
}).refine(data => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});

// Change password schema
const changePasswordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z.string()
    .min(8, 'Password must be at least 8 characters'),
  confirmPassword: z.string(),
}).refine(data => data.newPassword === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Form Validation Consistency

*For any* form input that violates validation rules (empty required fields, invalid email format, weak password), the form SHALL display appropriate error messages and prevent submission.

**Validates: Requirements 1.2, 1.3, 5.4**

### Property 2: Authentication State Persistence

*For any* successful login response containing tokens, the Auth_Frontend SHALL store tokens and update isAuthenticated state to true, and the user object SHALL be populated.

**Validates: Requirements 2.5**

### Property 3: Protected Route Access Control

*For any* protected route and unauthenticated state (no valid tokens), navigation to that route SHALL result in redirect to login page.

**Validates: Requirements 12.1**

### Property 4: Session Data Display Completeness

*For any* session object returned from API, the rendered session item SHALL display device info, IP address, and last activity timestamp.

**Validates: Requirements 6.4**

### Property 5: Audit Log Data Display Completeness

*For any* audit log entry returned from API, the rendered log item SHALL display action, IP address, user agent, and timestamp.

**Validates: Requirements 11.2**

### Property 6: Loading State Visibility

*For any* async operation (API call), the UI SHALL display a loading indicator while the operation is in progress.

**Validates: Requirements 13.3**

## Error Handling

### API Error Handling

```typescript
// Error types from SDK
type ApiErrorType = 
  | 'validation_error'
  | 'authentication_error'
  | 'authorization_error'
  | 'not_found'
  | 'conflict'
  | 'rate_limit'
  | 'server_error'
  | 'network_error';

// Error handling strategy
const handleApiError = (error: AuthServerError): string => {
  switch (error.error) {
    case 'invalid_credentials':
      return 'Email hoặc mật khẩu không đúng';
    case 'email_exists':
      return 'Email đã được đăng ký';
    case 'invalid_token':
      return 'Phiên đăng nhập đã hết hạn';
    case 'mfa_required':
      return 'Cần xác thực 2 yếu tố';
    case 'invalid_mfa_code':
      return 'Mã xác thực không đúng';
    case 'rate_limit':
      return 'Quá nhiều yêu cầu, vui lòng thử lại sau';
    case 'network_error':
      return 'Lỗi kết nối, vui lòng kiểm tra mạng';
    default:
      return 'Đã xảy ra lỗi, vui lòng thử lại';
  }
};
```

### Form Validation Errors

- Hiển thị inline errors dưới mỗi field
- Highlight field có lỗi với border đỏ
- Clear error khi user bắt đầu sửa

### Network Error Handling

- Retry button cho network errors
- Toast notification cho transient errors
- Redirect to login cho authentication errors

## Testing Strategy

### Unit Tests

Unit tests sẽ sử dụng Vitest và React Testing Library để test:

- Component rendering
- Form validation logic
- Hook behavior
- Utility functions

### Property-Based Tests

Property-based tests sẽ sử dụng fast-check để verify:

- Form validation consistency across inputs
- Auth state transitions
- Route protection logic

### Integration Tests

Integration tests sẽ mock SDK và test:

- Login/register flows
- MFA verification flow
- Session management
- Profile updates

### Test Configuration

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    globals: true,
  },
});
```

### Property Test Annotations

Mỗi property test sẽ được annotate với format:
```typescript
// Feature: auth-frontend, Property 1: Form Validation Consistency
// Validates: Requirements 1.2, 1.3, 5.4
```

# Auth Server SDK - Hướng Dẫn Tích Hợp

Tài liệu hướng dẫn tích hợp Auth Server SDK vào dự án mới.

## Mục Lục

1. [Cài Đặt](#1-cài-đặt)
2. [Khởi Tạo Client](#2-khởi-tạo-client)
3. [Authentication](#3-authentication)
4. [User Profile](#4-user-profile)
5. [Multi-Factor Authentication (MFA)](#5-multi-factor-authentication-mfa)
6. [Session Management](#6-session-management)
7. [App Management](#7-app-management)
8. [API Key Authentication](#8-api-key-authentication)
9. [OAuth Integration](#9-oauth-integration)
10. [WebAuthn/Passkey](#10-webauthnpasskey)
11. [Admin APIs](#11-admin-apis)
12. [Error Handling](#12-error-handling)
13. [Best Practices](#13-best-practices)

---

## 1. Cài Đặt

### NPM
```bash
npm install auth-server-sdk
```

### Yarn
```bash
yarn add auth-server-sdk
```

### PNPM
```bash
pnpm add auth-server-sdk
```

---

## 2. Khởi Tạo Client

```typescript
import { AuthServerClient } from 'auth-server-sdk';

const client = new AuthServerClient({
  baseUrl: 'https://auth.your-domain.com',
  timeout: 30000, // optional, default 30s
});
```

### Kiểm tra kết nối
```typescript
// Health check
const health = await client.health();
console.log(health); // { status: 'ok', version: '1.0.0' }

// Ready check (bao gồm database)
const ready = await client.ready();
console.log(ready); // { status: 'ok', version: '1.0.0' }
```

---

## 3. Authentication

### 3.1 Đăng ký tài khoản
```typescript
try {
  const user = await client.auth.register({
    email: 'user@example.com',
    password: 'SecurePassword123!',
  });
  console.log('Registered:', user.id);
} catch (error) {
  if (error instanceof AuthServerError) {
    console.error(error.error, error.message);
  }
}
```

### 3.2 Đăng nhập
```typescript
import { LoginResponse, MfaRequiredResponse } from 'auth-server-sdk';

const result = await client.auth.login({
  email: 'user@example.com',
  password: 'SecurePassword123!',
});

// Kiểm tra MFA
if ('mfa_required' in result && result.mfa_required) {
  // Cần xác thực MFA
  const mfaResult = result as MfaRequiredResponse;
  console.log('MFA required, methods:', mfaResult.methods);
  
  // Hoàn tất MFA
  const tokens = await client.auth.completeMfaLogin({
    mfa_token: mfaResult.mfa_token,
    code: '123456', // TOTP code từ authenticator app
  });
  console.log('Login successful with MFA');
} else {
  // Đăng nhập thành công (không có MFA)
  const tokens = result as LoginResponse;
  console.log('Login successful, token expires in:', tokens.expires_in);
}
```

### 3.3 Refresh Token
```typescript
// SDK tự động refresh token khi access_token hết hạn
// Hoặc có thể gọi thủ công:
const newTokens = await client.auth.refresh({
  refresh_token: 'your-refresh-token',
});
```

### 3.4 Đăng xuất
```typescript
// Đăng xuất session hiện tại
await client.auth.logout();

// Đăng xuất tất cả sessions
await client.auth.logout({ all_sessions: true });
```

### 3.5 Quên mật khẩu
```typescript
// Gửi email reset password
await client.auth.forgotPassword({ email: 'user@example.com' });

// Reset password với token từ email
await client.auth.resetPassword({
  token: 'reset-token-from-email',
  new_password: 'NewSecurePassword123!',
});
```

### 3.6 Xác thực Email
```typescript
// Xác thực email với token
await client.auth.verifyEmail({ token: 'verification-token' });

// Gửi lại email xác thực
await client.auth.resendVerification();
```

---

## 4. User Profile

### 4.1 Lấy thông tin profile
```typescript
const profile = await client.user.getProfile();
console.log('User:', profile.email);
console.log('MFA enabled:', profile.mfa_enabled);
console.log('Email verified:', profile.email_verified);
```

### 4.2 Cập nhật profile
```typescript
const updated = await client.user.updateProfile({
  email: 'newemail@example.com',
});
```

### 4.3 Đổi mật khẩu
```typescript
await client.user.changePassword({
  current_password: 'OldPassword123!',
  new_password: 'NewPassword456!',
});
```

### 4.4 Xem các app đã kết nối (OAuth)
```typescript
const connectedApps = await client.user.getConnectedApps();
for (const app of connectedApps.apps) {
  console.log(`${app.client_name}: ${app.scopes.join(', ')}`);
}

// Thu hồi quyền truy cập của app
await client.user.revokeAppConsent('client-id');
```

---

## 5. Multi-Factor Authentication (MFA)

### 5.1 Thiết lập TOTP (Google Authenticator, Authy, etc.)
```typescript
// Bước 1: Khởi tạo setup
const setup = await client.mfa.setupTotp();
console.log('Secret:', setup.secret);
console.log('QR Code URI:', setup.provisioning_uri);
// Hiển thị QR code cho user quét bằng authenticator app

// Bước 2: Xác thực code để hoàn tất setup
await client.mfa.verifyTotpSetup({
  method_id: setup.method_id,
  code: '123456', // Code từ authenticator app
});
console.log('TOTP setup complete!');
```

### 5.2 Xem các phương thức MFA đã thiết lập
```typescript
const methods = await client.mfa.getMethods();
console.log('MFA enabled:', methods.mfa_enabled);
for (const method of methods.methods) {
  console.log(`${method.method_type}: verified=${method.is_verified}`);
}
```

### 5.3 Tắt MFA
```typescript
await client.mfa.disable({
  method_id: 'method-uuid',
  code: '123456', // Code xác thực
});
```

### 5.4 Tạo lại Backup Codes
```typescript
const backupCodes = await client.mfa.regenerateBackupCodes({
  code: '123456', // TOTP code để xác thực
});
console.log('New backup codes:', backupCodes.backup_codes);
// Lưu ý: Hiển thị cho user lưu lại, chỉ hiển thị 1 lần!
```

---

## 6. Session Management

### 6.1 Xem danh sách sessions
```typescript
const sessions = await client.auth.getSessions();
for (const session of sessions.sessions) {
  console.log(`Session ${session.id}:`);
  console.log(`  IP: ${session.ip_address}`);
  console.log(`  User Agent: ${session.user_agent}`);
  console.log(`  Last used: ${session.last_used_at}`);
}
```

### 6.2 Thu hồi session cụ thể
```typescript
await client.auth.revokeSession({ session_id: 'session-uuid' });
```

### 6.3 Thu hồi tất cả sessions khác
```typescript
// Giữ lại session hiện tại, đăng xuất tất cả thiết bị khác
await client.auth.revokeOtherSessions();
```

### 6.4 Xem Audit Logs
```typescript
const logs = await client.auth.getAuditLogs({ page: 1, limit: 20 });
for (const log of logs.logs) {
  console.log(`${log.created_at}: ${log.action} from ${log.ip_address}`);
}
```

---

## 7. App Management

### 7.1 Tạo App mới
```typescript
const app = await client.apps.create({
  code: 'my-app',
  name: 'My Application',
});
console.log('App ID:', app.id);
console.log('App Secret:', app.secret); // Lưu lại, chỉ hiển thị 1 lần!
```

### 7.2 Danh sách Apps
```typescript
const apps = await client.apps.list({ page: 1, limit: 10 });
for (const app of apps.data) {
  console.log(`${app.name} (${app.code})`);
}
```

### 7.3 Xác thực App (lấy app token)
```typescript
const appAuth = await client.apps.authenticate({
  app_id: 'app-uuid',
  secret: 'app-secret',
});
// Token được tự động lưu trong client
console.log('App authenticated');
```

### 7.4 Quản lý Users trong App
```typescript
// Lấy danh sách users
const users = await client.apps.getUsers('app-id', { page: 1, limit: 20 });

// Ban user
await client.apps.banUser('app-id', 'user-id');

// Unban user
await client.apps.unbanUser('app-id', 'user-id');

// Xóa user khỏi app
await client.apps.removeUser('app-id', 'user-id');
```

### 7.5 Roles & Permissions
```typescript
// Tạo role
const role = await client.apps.createRole('app-id', { name: 'admin' });

// Tạo permission
const permission = await client.apps.createPermission('app-id', { 
  code: 'users:read' 
});

// Gán permission cho role
await client.apps.assignPermissionToRole('app-id', role.id, {
  permission_id: permission.id,
});

// Gán role cho user
await client.apps.assignRole('app-id', 'user-id', { role_id: role.id });

// Lấy roles của user
const userRoles = await client.apps.getUserRoles('app-id', 'user-id');
```

### 7.6 Webhooks
```typescript
// Tạo webhook
const webhook = await client.apps.createWebhook('app-id', {
  url: 'https://your-app.com/webhooks/auth',
  events: ['user.created', 'user.login', 'user.password_changed'],
});
console.log('Webhook secret:', webhook.secret); // Dùng để verify webhook signature

// Danh sách webhooks
const webhooks = await client.apps.listWebhooks('app-id');

// Cập nhật webhook
await client.apps.updateWebhook('app-id', webhook.id, {
  events: ['user.created', 'user.deleted'],
  is_active: true,
});

// Xóa webhook
await client.apps.deleteWebhook('app-id', webhook.id);
```

---

## 8. API Key Authentication

API Key cho phép xác thực server-to-server mà không cần user token.

### 8.1 Tạo API Key
```typescript
const apiKey = await client.apps.createApiKey('app-id', {
  name: 'Production API Key',
  scopes: ['users:read', 'users:write'],
  expires_at: '2025-12-31T23:59:59Z', // optional
});
console.log('API Key:', apiKey.key); // Lưu lại! Chỉ hiển thị 1 lần
console.log('Key prefix:', apiKey.key_prefix); // Dùng để identify key
```

### 8.2 Sử dụng API Key
```typescript
// Set API key cho client
client.setAuthApiKey('your-api-key-here');

// Các API trong appSelf module sẽ tự động dùng X-API-Key header
const roles = await client.appSelf.listRoles('app-id');
const permissions = await client.appSelf.listPermissions('app-id');
```

### 8.3 Quản lý API Keys
```typescript
// Danh sách API keys
const keys = await client.apps.listApiKeys('app-id');

// Cập nhật API key
await client.apps.updateApiKey('app-id', 'key-id', {
  name: 'Updated Name',
  is_active: false, // Disable key
});

// Thu hồi API key
await client.apps.revokeApiKey('app-id', 'key-id');

// Xóa API key
await client.apps.deleteApiKey('app-id', 'key-id');
```

### 8.4 IP Rules cho App
```typescript
// Whitelist IP
await client.apps.createIpRule('app-id', {
  ip_address: '192.168.1.100',
  rule_type: 'whitelist',
  reason: 'Office IP',
});

// Blacklist IP range
await client.apps.createIpRule('app-id', {
  ip_address: '10.0.0.0',
  ip_range: '10.0.0.0/24',
  rule_type: 'blacklist',
  reason: 'Suspicious activity',
  expires_at: '2025-06-01T00:00:00Z',
});

// Danh sách IP rules
const rules = await client.apps.listIpRules('app-id');
```

---

## 9. OAuth Integration

### 9.1 Tạo OAuth Client
```typescript
const oauthClient = await client.oauth.createClient({
  name: 'My OAuth App',
  redirect_uris: [
    'https://myapp.com/callback',
    'https://myapp.com/auth/callback',
  ],
  is_internal: false,
});
console.log('Client ID:', oauthClient.client_id);
console.log('Client Secret:', oauthClient.client_secret);
```

### 9.2 Quản lý OAuth Clients
```typescript
// Danh sách clients
const clients = await client.oauth.listClients();

// Cập nhật client
await client.oauth.updateClient('client-id', {
  name: 'Updated App Name',
  redirect_uris: ['https://newdomain.com/callback'],
});

// Regenerate secret
const newSecret = await client.oauth.regenerateClientSecret('client-id');

// Xóa client
await client.oauth.deleteClient('client-id');
```

### 9.3 OAuth Scopes
```typescript
// Danh sách public scopes
const scopes = await client.oauth.listPublicScopes();
for (const scope of scopes.scopes) {
  console.log(`${scope.code}: ${scope.description}`);
}
```

### 9.4 OpenID Connect
```typescript
// Lấy OpenID Configuration
const config = await client.oauth.getOpenIdConfiguration();
console.log('Authorization endpoint:', config.authorization_endpoint);
console.log('Token endpoint:', config.token_endpoint);

// Lấy User Info (cần OAuth access token)
const userInfo = await client.oauth.getUserInfo();
console.log('User:', userInfo.email);
```

### 9.5 Thu hồi Token
```typescript
await client.oauth.revokeToken({
  token: 'access-or-refresh-token',
  token_type_hint: 'access_token', // hoặc 'refresh_token'
});
```

---

## 10. WebAuthn/Passkey

### 10.1 Đăng ký Passkey mới
```typescript
// Bước 1: Lấy registration options
const options = await client.webauthn.startRegistration({
  device_name: 'MacBook Pro',
});

// Bước 2: Gọi WebAuthn API của browser
const credential = await navigator.credentials.create({
  publicKey: {
    challenge: Uint8Array.from(atob(options.challenge), c => c.charCodeAt(0)),
    rp: options.rp,
    user: {
      id: Uint8Array.from(atob(options.user.id), c => c.charCodeAt(0)),
      name: options.user.name,
      displayName: options.user.display_name,
    },
    pubKeyCredParams: options.pub_key_cred_params,
    timeout: options.timeout,
    attestation: options.attestation as AttestationConveyancePreference,
    authenticatorSelection: {
      authenticatorAttachment: options.authenticator_selection.authenticator_attachment as AuthenticatorAttachment,
      residentKey: options.authenticator_selection.resident_key as ResidentKeyRequirement,
      userVerification: options.authenticator_selection.user_verification as UserVerificationRequirement,
    },
  },
});

// Bước 3: Gửi credential về server
const passkey = await client.webauthn.finishRegistration({
  id: credential.id,
  raw_id: btoa(String.fromCharCode(...new Uint8Array(credential.rawId))),
  response: {
    client_data_json: btoa(String.fromCharCode(...new Uint8Array(credential.response.clientDataJSON))),
    attestation_object: btoa(String.fromCharCode(...new Uint8Array(credential.response.attestationObject))),
  },
  type: credential.type,
  device_name: 'MacBook Pro',
});
console.log('Passkey registered:', passkey.id);
```

### 10.2 Đăng nhập bằng Passkey
```typescript
// Bước 1: Lấy authentication options
const options = await client.webauthn.startAuthentication({
  email: 'user@example.com', // optional
});

// Bước 2: Gọi WebAuthn API của browser
const assertion = await navigator.credentials.get({
  publicKey: {
    challenge: Uint8Array.from(atob(options.challenge), c => c.charCodeAt(0)),
    timeout: options.timeout,
    rpId: options.rp_id,
    allowCredentials: options.allow_credentials.map(cred => ({
      id: Uint8Array.from(atob(cred.id), c => c.charCodeAt(0)),
      type: cred.type as PublicKeyCredentialType,
      transports: cred.transports as AuthenticatorTransport[],
    })),
    userVerification: options.user_verification as UserVerificationRequirement,
  },
});

// Bước 3: Gửi assertion về server
const tokens = await client.webauthn.finishAuthentication({
  id: assertion.id,
  raw_id: btoa(String.fromCharCode(...new Uint8Array(assertion.rawId))),
  response: {
    client_data_json: btoa(String.fromCharCode(...new Uint8Array(assertion.response.clientDataJSON))),
    authenticator_data: btoa(String.fromCharCode(...new Uint8Array(assertion.response.authenticatorData))),
    signature: btoa(String.fromCharCode(...new Uint8Array(assertion.response.signature))),
    user_handle: assertion.response.userHandle 
      ? btoa(String.fromCharCode(...new Uint8Array(assertion.response.userHandle)))
      : undefined,
  },
  type: assertion.type,
});
console.log('Logged in with passkey!');
```

### 10.3 Quản lý Passkeys
```typescript
// Danh sách passkeys
const passkeys = await client.webauthn.list();
for (const pk of passkeys) {
  console.log(`${pk.device_name}: last used ${pk.last_used_at}`);
}

// Đổi tên passkey
await client.webauthn.rename('passkey-id', { name: 'iPhone 15 Pro' });

// Xóa passkey
await client.webauthn.remove('passkey-id');
```

---

## 11. Admin APIs

> ⚠️ Các API này yêu cầu quyền System Admin

### 11.1 Quản lý Users
```typescript
// Danh sách users
const users = await client.admin.listUsers({ page: 1, limit: 20 });

// Tìm kiếm users
const searchResults = await client.admin.searchUsers({
  email: 'john',
  is_active: true,
  page: 1,
  limit: 10,
});

// Chi tiết user
const user = await client.admin.getUser('user-id');

// Cập nhật user
await client.admin.updateUser('user-id', {
  is_active: false,
  is_system_admin: true,
});

// Deactivate/Activate user
await client.admin.deactivateUser('user-id');
await client.admin.activateUser('user-id');

// Unlock user (sau khi bị lock do login fail nhiều lần)
await client.admin.unlockUser('user-id');

// Xóa user
await client.admin.deleteUser('user-id');
```

### 11.2 Quản lý Apps
```typescript
// Danh sách apps
const apps = await client.admin.listApps({ page: 1, limit: 20 });

// Chi tiết app
const app = await client.admin.getApp('app-id');

// Cập nhật app
await client.admin.updateApp('app-id', {
  name: 'New App Name',
});

// Xóa app
await client.admin.deleteApp('app-id');
```

### 11.3 Quản lý OAuth Scopes
```typescript
// Danh sách scopes
const scopes = await client.admin.listScopes({ page: 1, limit: 50 });

// Tạo scope mới
const scope = await client.admin.createScope({
  code: 'custom:read',
  description: 'Read custom resources',
});

// Cập nhật scope
await client.admin.updateScope('scope-id', {
  description: 'Updated description',
});

// Activate/Deactivate scope
await client.admin.activateScope('scope-id');
await client.admin.deactivateScope('scope-id');

// Xóa scope
await client.admin.deleteScope('scope-id');
```

### 11.4 Global IP Rules
```typescript
// Tạo global IP rule
await client.admin.createIpRule({
  ip_address: '1.2.3.4',
  rule_type: 'blacklist',
  reason: 'Malicious activity',
});

// Danh sách IP rules
const rules = await client.admin.listIpRules();

// Kiểm tra IP
const check = await client.admin.checkIp('1.2.3.4');
console.log('IP allowed:', check.allowed);

// Xóa IP rule
await client.admin.deleteIpRule('rule-id');
```

### 11.5 Audit Logs (Admin)
```typescript
const logs = await client.admin.getAuditLogs({
  page: 1,
  limit: 50,
  user_id: 'specific-user-id', // optional filter
});
```

### 11.6 Import/Export Users
```typescript
// Export users
const exportData = await client.admin.exportUsers();

// Import users
const result = await client.admin.importUsers({
  users: [
    { email: 'user1@example.com', password: 'Password123!' },
    { email: 'user2@example.com', password: 'Password456!' },
  ],
});
console.log('Imported:', result.imported);
console.log('Failed:', result.failed);
```

### 11.7 Bulk Role Assignment
```typescript
await client.admin.bulkAssignRole({
  user_ids: ['user-1', 'user-2', 'user-3'],
  app_id: 'app-id',
  role_id: 'role-id',
});
```

---

## 12. Error Handling

### 12.1 AuthServerError
```typescript
import { AuthServerClient, AuthServerError } from 'auth-server-sdk';

try {
  await client.auth.login({ email: 'test@example.com', password: 'wrong' });
} catch (error) {
  if (error instanceof AuthServerError) {
    console.log('Error code:', error.error);      // e.g., 'invalid_credentials'
    console.log('Status:', error.statusCode);     // e.g., 401
    console.log('Message:', error.message);       // Human-readable message
    
    // Handle specific errors
    switch (error.error) {
      case 'invalid_credentials':
        console.log('Wrong email or password');
        break;
      case 'user_inactive':
        console.log('Account is deactivated');
        break;
      case 'user_locked':
        console.log('Account is locked due to too many failed attempts');
        break;
      case 'email_not_verified':
        console.log('Please verify your email first');
        break;
      case 'mfa_required':
        console.log('MFA verification needed');
        break;
      default:
        console.log('Unknown error:', error.message);
    }
  }
}
```

### 12.2 Common Error Codes

| Error Code | Status | Description |
|------------|--------|-------------|
| `invalid_credentials` | 401 | Email hoặc password sai |
| `invalid_token` | 401 | Token không hợp lệ hoặc hết hạn |
| `token_expired` | 401 | Token đã hết hạn |
| `user_inactive` | 403 | Tài khoản bị deactivate |
| `user_locked` | 403 | Tài khoản bị lock |
| `user_banned` | 403 | User bị ban khỏi app |
| `email_not_verified` | 403 | Email chưa được xác thực |
| `mfa_required` | 403 | Cần xác thực MFA |
| `insufficient_scope` | 403 | Không đủ quyền (scope) |
| `not_found` | 404 | Resource không tồn tại |
| `validation_error` | 400 | Dữ liệu không hợp lệ |
| `email_exists` | 409 | Email đã được sử dụng |
| `rate_limit_exceeded` | 429 | Quá nhiều requests |
| `internal_error` | 500 | Lỗi server |

### 12.3 Network Errors
```typescript
try {
  await client.auth.login({ email: 'test@example.com', password: 'test' });
} catch (error) {
  if (error instanceof AuthServerError && error.error === 'network_error') {
    console.log('Network error - check your connection');
  }
}
```

---

## 13. Best Practices

### 13.1 Token Storage (Browser)
```typescript
// Lưu tokens vào localStorage hoặc sessionStorage
const tokens = await client.auth.login({ email, password });

// Không nên lưu refresh_token trong localStorage cho production
// Sử dụng httpOnly cookies hoặc secure storage

// Khôi phục tokens khi reload page
const savedAccessToken = localStorage.getItem('access_token');
const savedRefreshToken = localStorage.getItem('refresh_token');
if (savedAccessToken) {
  client.setTokens(savedAccessToken, savedRefreshToken);
}
```

### 13.2 Token Storage (Node.js/Server)
```typescript
// Lưu tokens trong memory hoặc secure storage
class TokenStore {
  private tokens: Map<string, { access: string; refresh: string }> = new Map();
  
  save(userId: string, accessToken: string, refreshToken: string) {
    this.tokens.set(userId, { access: accessToken, refresh: refreshToken });
  }
  
  get(userId: string) {
    return this.tokens.get(userId);
  }
}
```

### 13.3 Retry Logic
```typescript
async function withRetry<T>(
  fn: () => Promise<T>,
  maxRetries = 3,
  delay = 1000
): Promise<T> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (error instanceof AuthServerError) {
        // Không retry cho client errors (4xx)
        if (error.statusCode >= 400 && error.statusCode < 500) {
          throw error;
        }
      }
      if (i === maxRetries - 1) throw error;
      await new Promise(r => setTimeout(r, delay * (i + 1)));
    }
  }
  throw new Error('Max retries exceeded');
}

// Usage
const profile = await withRetry(() => client.user.getProfile());
```

### 13.4 Singleton Pattern
```typescript
// lib/auth-client.ts
import { AuthServerClient } from 'auth-server-sdk';

let client: AuthServerClient | null = null;

export function getAuthClient(): AuthServerClient {
  if (!client) {
    client = new AuthServerClient({
      baseUrl: process.env.AUTH_SERVER_URL || 'https://auth.example.com',
    });
  }
  return client;
}
```

### 13.5 React Hook Example
```typescript
// hooks/useAuth.ts
import { useState, useEffect, createContext, useContext } from 'react';
import { AuthServerClient, UserProfile } from 'auth-server-sdk';

const client = new AuthServerClient({ baseUrl: 'https://auth.example.com' });

interface AuthContextType {
  user: UserProfile | null;
  loading: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Check for existing session
    const token = localStorage.getItem('access_token');
    if (token) {
      client.setTokens(token, localStorage.getItem('refresh_token') || undefined);
      client.user.getProfile()
        .then(setUser)
        .catch(() => localStorage.clear())
        .finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, []);

  const login = async (email: string, password: string) => {
    const result = await client.auth.login({ email, password });
    if ('access_token' in result) {
      localStorage.setItem('access_token', result.access_token);
      localStorage.setItem('refresh_token', result.refresh_token);
      const profile = await client.user.getProfile();
      setUser(profile);
    }
  };

  const logout = async () => {
    await client.auth.logout();
    localStorage.clear();
    client.clearTokens();
    setUser(null);
  };

  return (
    <AuthContext.Provider value={{ user, loading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error('useAuth must be used within AuthProvider');
  return context;
};
```

---

## API Reference

Xem chi tiết types và interfaces trong file `sdk/src/types.ts`.

### API Modules

| Module | Description |
|--------|-------------|
| `client.auth` | Authentication (login, register, sessions) |
| `client.mfa` | Multi-factor authentication |
| `client.user` | User profile management |
| `client.apps` | App management (owner APIs) |
| `client.appSelf` | App self-management (API key auth) |
| `client.webauthn` | WebAuthn/Passkey |
| `client.oauth` | OAuth clients and tokens |
| `client.admin` | Admin operations |

---

## Support

- GitHub Issues: [auth-server/issues](https://github.com/user/auth-server/issues)
- Documentation: [docs/](../docs/)

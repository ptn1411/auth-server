# Auth Server SDK

TypeScript SDK for Auth Server API.

## Installation

```bash
npm install auth-server-sdk
```

## Usage

### Basic Setup

```typescript
import { AuthServerClient } from 'auth-server-sdk';

const client = new AuthServerClient({
  baseUrl: 'http://localhost:3000',
  timeout: 30000, // optional, default 30s
});
```

### Authentication

```typescript
// Register
const user = await client.register({
  email: 'user@example.com',
  password: 'SecurePassword123!',
});

// Login
const loginResult = await client.login({
  email: 'user@example.com',
  password: 'SecurePassword123!',
});

// Handle MFA if required
if ('mfa_required' in loginResult) {
  const tokens = await client.completeMfaLogin({
    mfa_token: loginResult.mfa_token,
    code: '123456',
  });
}

// Refresh token
await client.refresh();

// Logout
await client.logout();
```

### User Profile

```typescript
// Get profile
const profile = await client.getProfile();

// Update profile
await client.updateProfile({ email: 'new@example.com' });

// Change password
await client.changePassword({
  current_password: 'OldPassword123!',
  new_password: 'NewPassword123!',
});
```

### MFA Management

```typescript
// Setup TOTP
const setup = await client.setupTotp();
console.log('Scan QR:', setup.provisioning_uri);

// Verify TOTP setup
const backupCodes = await client.verifyTotpSetup({
  method_id: setup.method_id,
  code: '123456',
});

// Get MFA methods
const methods = await client.getMfaMethods();

// Disable MFA
await client.disableMfa();
```

### Session Management

```typescript
// List sessions
const sessions = await client.getSessions();

// Revoke specific session
await client.revokeSession({ session_id: 'session-id' });

// Revoke all other sessions
await client.revokeOtherSessions();
```

### App Management

```typescript
// Create app
const app = await client.createApp({
  code: 'my-app',
  name: 'My Application',
});
console.log('App Secret:', app.secret);

// Authenticate app
const appAuth = await client.authenticateApp({
  app_id: app.id,
  secret: app.secret!,
});

// Regenerate secret
const newSecret = await client.regenerateAppSecret(app.id);
```

### Role & Permission Management

```typescript
// Create role
const role = await client.createRole(appId, { name: 'admin' });

// Create permission
const permission = await client.createPermission(appId, { code: 'read:users' });

// Assign role to user
await client.assignRole(appId, userId, { role_id: role.id });

// Get user roles
const roles = await client.getUserRolesInApp(appId, userId);

// Remove role
await client.removeRole(appId, userId, role.id);
```

### Admin Operations

```typescript
// List all users
const users = await client.adminListUsers({ page: 1, limit: 10 });

// Search users
const searchResults = await client.adminSearchUsers({
  email: 'test',
  is_active: true,
});

// Get user details
const user = await client.adminGetUser(userId);

// Update user
await client.adminUpdateUser(userId, { is_active: false });

// Deactivate/Activate user
await client.adminDeactivateUser(userId);
await client.adminActivateUser(userId);

// Unlock user account
await client.adminUnlockUser(userId);

// Delete user
await client.adminDeleteUser(userId);

// List all apps
const apps = await client.adminListApps();

// Update app
await client.adminUpdateApp(appId, { name: 'New Name' });

// Delete app
await client.adminDeleteApp(appId);

// Get audit logs
const logs = await client.adminGetAuditLogs({ page: 1, limit: 50 });
```

### Error Handling

```typescript
import { AuthServerClient, AuthServerError } from 'auth-server-sdk';

try {
  await client.login({ email: 'user@example.com', password: 'wrong' });
} catch (error) {
  if (error instanceof AuthServerError) {
    console.log('Error:', error.error);
    console.log('Message:', error.message);
    console.log('Status:', error.statusCode);
  }
}
```

## API Reference

### AuthServerClient

#### Constructor

```typescript
new AuthServerClient(config: AuthServerConfig)
```

- `baseUrl`: Base URL of the Auth Server
- `timeout`: Request timeout in milliseconds (default: 30000)

#### Token Management

- `setTokens(accessToken, refreshToken?)`: Set tokens manually
- `getAccessToken()`: Get current access token
- `clearTokens()`: Clear stored tokens

#### Health Check

- `health()`: Check server health
- `ready()`: Check server readiness

#### Authentication

- `register(data)`: Register new user
- `login(data)`: Login user
- `completeMfaLogin(data)`: Complete MFA login
- `refresh(data?)`: Refresh access token
- `forgotPassword(data)`: Request password reset
- `resetPassword(data)`: Reset password
- `verifyEmail(data)`: Verify email
- `resendVerification()`: Resend verification email
- `logout(data?)`: Logout user

#### User Profile

- `getProfile()`: Get current user profile
- `updateProfile(data)`: Update profile
- `changePassword(data)`: Change password

#### Session Management

- `getSessions()`: List active sessions
- `revokeSession(data)`: Revoke specific session
- `revokeOtherSessions()`: Revoke all other sessions

#### MFA

- `setupTotp()`: Initialize TOTP setup
- `verifyTotpSetup(data)`: Verify TOTP setup
- `getMfaMethods()`: List MFA methods
- `disableMfa()`: Disable MFA
- `regenerateBackupCodes()`: Regenerate backup codes

#### Audit Logs

- `getAuditLogs(params?)`: Get user audit logs

#### App Management

- `createApp(data)`: Create new app
- `authenticateApp(data)`: Authenticate app
- `regenerateAppSecret(appId)`: Regenerate app secret

#### Role Management

- `createRole(appId, data)`: Create role
- `assignRole(appId, userId, data)`: Assign role to user
- `getUserRolesInApp(appId, userId)`: Get user roles
- `removeRole(appId, userId, roleId)`: Remove role from user

#### Permission Management

- `createPermission(appId, data)`: Create permission

#### App User Management

- `registerToApp(appId)`: Register to app
- `getAppUsers(appId, params?)`: List app users
- `banUser(appId, userId)`: Ban user
- `unbanUser(appId, userId)`: Unban user
- `removeUserFromApp(appId, userId)`: Remove user from app

#### OAuth Account

- `getConnectedApps()`: List connected OAuth apps
- `revokeAppConsent(clientId)`: Revoke app consent

#### Admin

- `adminListUsers(params?)`: List all users
- `adminSearchUsers(params?)`: Search users
- `adminGetUser(userId)`: Get user details
- `adminUpdateUser(userId, data)`: Update user
- `adminDeleteUser(userId)`: Delete user
- `adminDeactivateUser(userId)`: Deactivate user
- `adminActivateUser(userId)`: Activate user
- `adminUnlockUser(userId)`: Unlock user account
- `adminGetUserRoles(userId)`: Get user roles
- `adminListApps(params?)`: List all apps
- `adminGetApp(appId)`: Get app details
- `adminUpdateApp(appId, data)`: Update app
- `adminDeleteApp(appId)`: Delete app
- `adminGetAuditLogs(params?)`: Get all audit logs
- `adminExportUsers()`: Export all users
- `adminImportUsers(users)`: Import users
- `adminBulkAssignRole(userIds, roleId)`: Bulk assign role

## License

MIT


## Advanced Features

### Webhooks

```typescript
// Create webhook
const webhook = await client.createWebhook(appId, {
  url: 'https://example.com/webhook',
  events: ['user.login', 'user.register', 'user.logout'],
});
console.log('Webhook Secret:', webhook.secret);

// List webhooks
const webhooks = await client.listWebhooks(appId);

// Get webhook
const wh = await client.getWebhook(appId, webhookId);

// Update webhook
await client.updateWebhook(appId, webhookId, {
  events: ['user.login'],
  is_active: false,
});

// Delete webhook
await client.deleteWebhook(appId, webhookId);
```

### API Keys (Server-to-Server)

```typescript
// Create API key
const apiKey = await client.createApiKey(appId, {
  name: 'Production Key',
  scopes: ['read:users', 'write:users'],
  expires_at: '2025-12-31T23:59:59Z', // optional
});
console.log('API Key:', apiKey.key); // Only shown once!

// List API keys
const keys = await client.listApiKeys(appId);

// Get API key
const key = await client.getApiKey(appId, keyId);

// Update API key
await client.updateApiKey(appId, keyId, {
  name: 'Updated Name',
  scopes: ['read:users'],
});

// Revoke API key
await client.revokeApiKey(appId, keyId);

// Delete API key
await client.deleteApiKey(appId, keyId);
```

### IP Rules (Whitelist/Blacklist)

```typescript
// App-level IP rules
const rule = await client.createAppIpRule(appId, {
  ip_address: '192.168.1.100',
  rule_type: 'whitelist',
  reason: 'Office IP',
});

const appRules = await client.listAppIpRules(appId);

// Admin global IP rules
const globalRule = await client.adminCreateIpRule({
  ip_address: '10.0.0.0',
  ip_range: '10.0.0.0/8',
  rule_type: 'blacklist',
  reason: 'Suspicious network',
});

const globalRules = await client.adminListIpRules();

// Check IP access
const check = await client.adminCheckIp('192.168.1.100');
console.log('Allowed:', check.allowed);

// Delete rule
await client.adminDeleteIpRule(ruleId);
```

### WebAuthn/Passkeys

```typescript
// Start passkey registration (requires browser WebAuthn API)
const regOptions = await client.startPasskeyRegistration({
  device_name: 'My MacBook',
});

// After browser creates credential:
const passkey = await client.finishPasskeyRegistration({
  id: credential.id,
  raw_id: btoa(credential.rawId),
  response: {
    client_data_json: btoa(credential.response.clientDataJSON),
    attestation_object: btoa(credential.response.attestationObject),
  },
  type: 'public-key',
  device_name: 'My MacBook',
});

// Start passkey authentication
const authOptions = await client.startPasskeyAuthentication({
  email: 'user@example.com', // optional hint
});

// After browser gets assertion:
const tokens = await client.finishPasskeyAuthentication({
  id: assertion.id,
  raw_id: btoa(assertion.rawId),
  response: {
    client_data_json: btoa(assertion.response.clientDataJSON),
    authenticator_data: btoa(assertion.response.authenticatorData),
    signature: btoa(assertion.response.signature),
    user_handle: assertion.response.userHandle ? btoa(assertion.response.userHandle) : undefined,
  },
  type: 'public-key',
});

// List passkeys
const passkeys = await client.listPasskeys();

// Rename passkey
await client.renamePasskey(credentialId, { name: 'Work Laptop' });

// Delete passkey
await client.deletePasskey(credentialId);
```

### API Reference (Advanced Features)

#### Webhooks

- `createWebhook(appId, data)`: Create webhook
- `listWebhooks(appId)`: List webhooks
- `getWebhook(appId, webhookId)`: Get webhook
- `updateWebhook(appId, webhookId, data)`: Update webhook
- `deleteWebhook(appId, webhookId)`: Delete webhook

#### API Keys

- `createApiKey(appId, data)`: Create API key
- `listApiKeys(appId)`: List API keys
- `getApiKey(appId, keyId)`: Get API key
- `updateApiKey(appId, keyId, data)`: Update API key
- `revokeApiKey(appId, keyId)`: Revoke API key
- `deleteApiKey(appId, keyId)`: Delete API key

#### IP Rules

- `createAppIpRule(appId, data)`: Create app IP rule
- `listAppIpRules(appId)`: List app IP rules
- `adminCreateIpRule(data)`: Create global IP rule
- `adminListIpRules()`: List global IP rules
- `adminCheckIp(ip, appId?)`: Check IP access
- `adminDeleteIpRule(ruleId)`: Delete IP rule

#### WebAuthn/Passkeys

- `startPasskeyRegistration(data?)`: Start registration
- `finishPasskeyRegistration(data)`: Complete registration
- `startPasskeyAuthentication(data?)`: Start authentication
- `finishPasskeyAuthentication(data)`: Complete authentication
- `listPasskeys()`: List user passkeys
- `renamePasskey(credentialId, data)`: Rename passkey
- `deletePasskey(credentialId)`: Delete passkey

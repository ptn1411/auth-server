# Auth Server SDK

TypeScript/JavaScript SDK for Auth Server API.

## Installation

### From npm

```bash
npm install auth-server-sdk
```

### From GitHub Packages

```bash
npm install @username/auth-server-sdk --registry=https://npm.pkg.github.com
```

### From GitHub Releases

Download the tarball from [Releases](https://github.com/user/auth-server/releases) and install:

```bash
npm install ./auth-server-sdk-1.0.0.tgz
```

## Quick Start

```typescript
import { AuthServerClient } from 'auth-server-sdk';

const client = new AuthServerClient({
  baseUrl: 'http://localhost:3000',
});

// Register a new user
const user = await client.register({
  email: 'user@example.com',
  password: 'SecurePass123!',
});

// Login
const loginResponse = await client.login({
  email: 'user@example.com',
  password: 'SecurePass123!',
});

// Access token is automatically stored
console.log(client.getAccessToken());

// Get user profile
const profile = await client.getProfile();
```

## Features

- **Authentication**: Register, Login, Logout, Password Reset, Email Verification
- **MFA**: TOTP Setup, Backup Codes
- **Session Management**: List sessions, Revoke sessions
- **App Management**: Create apps, Manage roles & permissions
- **Webhooks**: Create, Update, Delete webhooks
- **API Keys**: Create, Revoke API keys
- **IP Rules**: Whitelist/Blacklist IP addresses
- **WebAuthn/Passkey**: Passwordless authentication
- **Admin API**: User management, App management, Audit logs

## API Reference

### Authentication

```typescript
// Register
await client.register({ email, password });

// Login
const response = await client.login({ email, password });
if ('mfa_required' in response) {
  // Handle MFA
  await client.completeMfaLogin({ mfa_token: response.mfa_token, code: '123456' });
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
  current_password: 'old',
  new_password: 'new',
});
```

### MFA

```typescript
// Setup TOTP
const setup = await client.setupTotp();
console.log(setup.provisioning_uri); // QR code URL

// Verify TOTP setup
const codes = await client.verifyTotpSetup({
  method_id: setup.method_id,
  code: '123456',
});
console.log(codes.backup_codes);

// Get MFA methods
const methods = await client.getMfaMethods();

// Disable MFA
await client.disableMfa();
```

### App Management

```typescript
// Create app
const app = await client.createApp({
  code: 'my-app',
  name: 'My Application',
});

// Authenticate app
const appAuth = await client.authenticateApp({
  app_id: app.id,
  secret: app.secret,
});

// Create role
const role = await client.createRole(app.id, { name: 'admin' });

// Assign role to user
await client.assignRole(app.id, userId, { role_id: role.id });

// Create permission
const permission = await client.createPermission(app.id, { code: 'read:users' });
```

### Webhooks

```typescript
// Create webhook
const webhook = await client.createWebhook(appId, {
  url: 'https://example.com/webhook',
  events: ['user.created', 'user.login'],
});

// List webhooks
const webhooks = await client.listWebhooks(appId);

// Update webhook
await client.updateWebhook(appId, webhookId, { is_active: false });

// Delete webhook
await client.deleteWebhook(appId, webhookId);
```

### API Keys

```typescript
// Create API key
const apiKey = await client.createApiKey(appId, {
  name: 'Production Key',
  scopes: ['read:users', 'write:users'],
});
console.log(apiKey.key); // Only shown once!

// List API keys
const keys = await client.listApiKeys(appId);

// Revoke API key
await client.revokeApiKey(appId, keyId);
```

### WebAuthn/Passkey

```typescript
// Start registration
const options = await client.startPasskeyRegistration({
  device_name: 'My Device',
});

// Finish registration (with browser WebAuthn API response)
await client.finishPasskeyRegistration(credential);

// Start authentication
const authOptions = await client.startPasskeyAuthentication();

// Finish authentication
const authResponse = await client.finishPasskeyAuthentication(assertion);
```

### Admin API

```typescript
// List users
const users = await client.adminListUsers({ page: 1, limit: 10 });

// Search users
const results = await client.adminSearchUsers({ email: 'test@' });

// Update user
await client.adminUpdateUser(userId, { is_active: false });

// Deactivate/Activate user
await client.adminDeactivateUser(userId);
await client.adminActivateUser(userId);

// Unlock user
await client.adminUnlockUser(userId);
```

## Error Handling

```typescript
import { AuthServerClient, AuthServerError } from 'auth-server-sdk';

try {
  await client.login({ email, password });
} catch (error) {
  if (error instanceof AuthServerError) {
    console.log(error.error);      // 'invalid_credentials'
    console.log(error.statusCode); // 401
    console.log(error.message);    // 'Invalid credentials'
  }
}
```

## Token Management

```typescript
// Manually set tokens
client.setTokens(accessToken, refreshToken);

// Get current access token
const token = client.getAccessToken();

// Clear tokens
client.clearTokens();
```

## Configuration

```typescript
const client = new AuthServerClient({
  baseUrl: 'https://auth.example.com',
  timeout: 30000, // Request timeout in ms (default: 30000)
});
```

## License

MIT

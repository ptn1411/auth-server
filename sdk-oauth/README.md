# @authserver/oauth-sdk

SDK để tích hợp OAuth login với Auth Server vào bất kỳ ứng dụng web nào.

## Tính năng

- ✅ OAuth2 Authorization Code Flow với PKCE
- ✅ Login bằng popup hoặc redirect
- ✅ Tự động refresh token
- ✅ Lưu trữ token an toàn
- ✅ TypeScript support
- ✅ React hook (optional)
- ✅ Hỗ trợ cross-tab sync
- ✅ Zero dependencies

## Cài đặt

```bash
npm install @authserver/oauth-sdk
# hoặc
yarn add @authserver/oauth-sdk
# hoặc
pnpm add @authserver/oauth-sdk
```

## Quick Start

### 1. Tạo OAuth Client

Đăng nhập vào Auth Server và tạo OAuth Client:
- Redirect URI: `http://localhost:3000/callback` (hoặc URL của bạn)
- Copy `client_id`

### 2. Khởi tạo SDK

```typescript
import { AuthServerClient } from '@authserver/oauth-sdk';

const auth = new AuthServerClient({
  serverUrl: 'https://auth.example.com',
  clientId: 'your-client-id',
  redirectUri: 'http://localhost:3000/callback',
  scopes: ['openid', 'profile', 'email'],
});
```

### 3. Login

```typescript
// Login bằng popup (recommended)
try {
  const tokens = await auth.loginWithPopup();
  console.log('Logged in!', tokens);
} catch (error) {
  console.error('Login failed:', error);
}

// Hoặc login bằng redirect
auth.loginWithRedirect();
```

### 4. Handle Callback (nếu dùng redirect)

```typescript
// Trên trang callback
if (window.location.search.includes('code=')) {
  try {
    const tokens = await auth.handleRedirectCallback();
    console.log('Logged in!', tokens);
  } catch (error) {
    console.error('Callback failed:', error);
  }
}
```

### 5. Sử dụng Token

```typescript
// Lấy access token (tự động refresh nếu cần)
const token = await auth.getAccessToken();

// Gọi API với token
const response = await fetch('https://api.example.com/data', {
  headers: {
    Authorization: `Bearer ${token}`,
  },
});

// Lấy thông tin user
const user = await auth.fetchUserInfo();
console.log(user.email, user.name);
```

### 6. Logout

```typescript
await auth.logout({ revokeToken: true });
```

## Sử dụng với React

```tsx
import { AuthServerClient, createAuthHook } from '@authserver/oauth-sdk';

// Khởi tạo client
const authClient = new AuthServerClient({
  serverUrl: 'https://auth.example.com',
  clientId: 'your-client-id',
  redirectUri: window.location.origin + '/callback',
});

// Tạo hook
const { useAuth } = createAuthHook(authClient);

// Sử dụng trong component
function App() {
  const { isAuthenticated, isLoading, user, login, logout, error } = useAuth();

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (!isAuthenticated) {
    return (
      <div>
        <button onClick={login}>Login</button>
        {error && <p style={{ color: 'red' }}>{error}</p>}
      </div>
    );
  }

  return (
    <div>
      <p>Welcome, {user?.name || user?.email}!</p>
      <button onClick={logout}>Logout</button>
    </div>
  );
}
```

## Sử dụng với Vue

```vue
<script setup>
import { ref, onMounted } from 'vue';
import { AuthServerClient } from '@authserver/oauth-sdk';

const auth = new AuthServerClient({
  serverUrl: 'https://auth.example.com',
  clientId: 'your-client-id',
  redirectUri: window.location.origin + '/callback',
});

const isAuthenticated = ref(false);
const user = ref(null);
const isLoading = ref(false);

onMounted(() => {
  const state = auth.getState();
  isAuthenticated.value = state.isAuthenticated;
  user.value = state.user;
});

async function login() {
  isLoading.value = true;
  try {
    await auth.loginWithPopup();
    const state = auth.getState();
    isAuthenticated.value = state.isAuthenticated;
    user.value = state.user;
  } catch (error) {
    console.error(error);
  } finally {
    isLoading.value = false;
  }
}

async function logout() {
  await auth.logout({ revokeToken: true });
  isAuthenticated.value = false;
  user.value = null;
}
</script>

<template>
  <div v-if="isLoading">Loading...</div>
  <div v-else-if="!isAuthenticated">
    <button @click="login">Login</button>
  </div>
  <div v-else>
    <p>Welcome, {{ user?.name || user?.email }}!</p>
    <button @click="logout">Logout</button>
  </div>
</template>
```

## Sử dụng với Vanilla JavaScript

```html
<!DOCTYPE html>
<html>
<head>
  <title>OAuth Demo</title>
</head>
<body>
  <div id="app">
    <div id="login-section">
      <button id="login-btn">Login with Auth Server</button>
    </div>
    <div id="user-section" style="display: none;">
      <p>Welcome, <span id="user-name"></span>!</p>
      <button id="logout-btn">Logout</button>
    </div>
  </div>

  <script type="module">
    import { AuthServerClient } from 'https://unpkg.com/@authserver/oauth-sdk';

    const auth = new AuthServerClient({
      serverUrl: 'https://auth.example.com',
      clientId: 'your-client-id',
      redirectUri: window.location.origin + '/callback.html',
    });

    // Check initial state
    function updateUI() {
      const state = auth.getState();
      if (state.isAuthenticated) {
        document.getElementById('login-section').style.display = 'none';
        document.getElementById('user-section').style.display = 'block';
        document.getElementById('user-name').textContent = 
          state.user?.name || state.user?.email || 'User';
      } else {
        document.getElementById('login-section').style.display = 'block';
        document.getElementById('user-section').style.display = 'none';
      }
    }

    // Login button
    document.getElementById('login-btn').addEventListener('click', async () => {
      try {
        await auth.loginWithPopup();
        await auth.fetchUserInfo();
        updateUI();
      } catch (error) {
        alert('Login failed: ' + error.message);
      }
    });

    // Logout button
    document.getElementById('logout-btn').addEventListener('click', async () => {
      await auth.logout({ revokeToken: true });
      updateUI();
    });

    // Initial UI update
    updateUI();
  </script>
</body>
</html>
```

## API Reference

### AuthServerClient

#### Constructor Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `serverUrl` | `string` | required | Auth Server URL |
| `clientId` | `string` | required | OAuth Client ID |
| `redirectUri` | `string` | required | Redirect URI sau khi login |
| `scopes` | `string[]` | `['openid', 'profile', 'email']` | OAuth scopes |
| `storage` | `Storage` | `localStorage` | Storage cho tokens |
| `storagePrefix` | `string` | `'authserver'` | Prefix cho storage keys |
| `autoRefresh` | `boolean` | `true` | Tự động refresh token |
| `refreshThreshold` | `number` | `60` | Refresh trước khi hết hạn (giây) |
| `popupWidth` | `number` | `500` | Chiều rộng popup |
| `popupHeight` | `number` | `600` | Chiều cao popup |
| `onTokenUpdate` | `function` | - | Callback khi token thay đổi |
| `onUserUpdate` | `function` | - | Callback khi user info thay đổi |

#### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `loginWithPopup()` | `Promise<TokenResponse>` | Login bằng popup |
| `loginWithRedirect()` | `void` | Login bằng redirect |
| `handleRedirectCallback()` | `Promise<TokenResponse>` | Handle callback sau redirect |
| `logout(options?)` | `Promise<void>` | Logout |
| `getAccessToken()` | `Promise<string \| null>` | Lấy access token (auto refresh) |
| `getTokens()` | `TokenResponse \| null` | Lấy stored tokens |
| `getUser()` | `UserInfo \| null` | Lấy stored user info |
| `fetchUserInfo()` | `Promise<UserInfo>` | Fetch user info từ server |
| `refreshToken()` | `Promise<TokenResponse>` | Refresh token manually |
| `isAuthenticated()` | `boolean` | Check authentication status |
| `getState()` | `AuthState` | Lấy full auth state |
| `clearTokens()` | `void` | Clear all stored data |

### Utility Functions

```typescript
import { generatePKCE, decodeJwt, isTokenExpired } from '@authserver/oauth-sdk';

// Generate PKCE challenge
const pkce = await generatePKCE();
console.log(pkce.codeVerifier, pkce.codeChallenge);

// Decode JWT (without verification)
const payload = decodeJwt(token);
console.log(payload.sub, payload.exp);

// Check if token is expired
const expired = isTokenExpired(token, 60); // 60 seconds threshold
```

## Callback Page

Nếu dùng popup login, bạn cần tạo một callback page:

```html
<!-- callback.html -->
<!DOCTYPE html>
<html>
<head><title>OAuth Callback</title></head>
<body>
  <p>Processing...</p>
  <script>
    // Parse URL parameters
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');
    const error = params.get('error');
    const errorDescription = params.get('error_description');

    // Send to opener
    if (window.opener) {
      window.opener.postMessage({
        type: 'oauth_callback',
        code,
        state,
        error,
        error_description: errorDescription,
      }, window.location.origin);
      window.close();
    }
  </script>
</body>
</html>
```

## Security

- ✅ PKCE (Proof Key for Code Exchange) - Bảo vệ authorization code
- ✅ State parameter - Chống CSRF attacks
- ✅ Nonce - Chống replay attacks (OpenID Connect)
- ✅ Secure storage - Tokens được lưu trong localStorage/sessionStorage
- ✅ Auto token refresh - Giảm thời gian token bị lộ

## Troubleshooting

### Popup bị block
- Đảm bảo gọi `loginWithPopup()` trong user interaction (click event)
- Cho phép popups cho domain của bạn

### CORS errors
- Kiểm tra Auth Server đã enable CORS
- Kiểm tra redirect_uri khớp chính xác

### Token refresh fails
- Đảm bảo đã request scope `offline_access`
- Kiểm tra refresh token chưa hết hạn

### State mismatch
- Có thể do localStorage bị clear
- Thử dùng sessionStorage thay vì localStorage

## License

MIT

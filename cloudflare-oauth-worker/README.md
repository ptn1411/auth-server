# Auth Server OAuth Proxy - Cloudflare Worker

OAuth proxy worker để kết nối các ứng dụng như Sveltia CMS, Netlify CMS với Auth Server.

## Tính năng

- ✅ OAuth2 Authorization Code Flow với PKCE
- ✅ Tương thích với Sveltia CMS / Netlify CMS
- ✅ CSRF protection với state parameter
- ✅ Domain whitelist
- ✅ Proxy userinfo endpoint

## Cài đặt

### 1. Cài dependencies

```bash
cd cloudflare-oauth-worker
npm install
```

### 2. Cấu hình

Tạo file `.dev.vars` cho development:

```env
AUTH_SERVER_URL=http://localhost:3000
OAUTH_CLIENT_ID=your-client-id
OAUTH_CLIENT_SECRET=your-client-secret
OAUTH_REDIRECT_URI=http://localhost:8787/callback
ALLOWED_DOMAINS=localhost:*,*.example.com
```

Hoặc set secrets cho production:

```bash
wrangler secret put AUTH_SERVER_URL
wrangler secret put OAUTH_CLIENT_ID
wrangler secret put OAUTH_CLIENT_SECRET
```

### 3. Tạo OAuth Client trong Auth Server

1. Đăng nhập vào Auth Server frontend (http://localhost:5173)
2. Vào "OAuth Clients" → "Create Client"
3. Điền thông tin:
   - Name: `Cloudflare OAuth Proxy`
   - Redirect URIs: 
     - Development: `http://localhost:8787/callback`
     - Production: `https://your-worker.workers.dev/callback`
4. Copy `client_id` và `client_secret`

### 4. Chạy development

```bash
npm run dev
```

Worker sẽ chạy tại http://localhost:8787

### 5. Deploy lên Cloudflare

```bash
npm run deploy
```

## Endpoints

| Endpoint | Method | Mô tả |
|----------|--------|-------|
| `/auth` | GET | Bắt đầu OAuth flow |
| `/callback` | GET | OAuth callback handler |
| `/userinfo` | GET | Proxy đến auth server userinfo |
| `/health` | GET | Health check |

## Sử dụng với Sveltia CMS

Trong `config.yml` của Sveltia CMS:

```yaml
backend:
  name: auth-server
  base_url: https://your-worker.workers.dev
  auth_endpoint: auth
```

## Sử dụng với ứng dụng khác

### Bắt đầu OAuth flow

Redirect user đến:
```
https://your-worker.workers.dev/auth?scope=openid%20profile%20email
```

Query parameters:
- `scope` (optional): OAuth scopes, default: `openid profile email`
- `site_id` (optional): Domain để check whitelist
- `provider` (optional): Provider name, default: `auth-server`

### Nhận token

Sau khi user authorize, callback page sẽ gửi postMessage:

```javascript
// Sveltia CMS format
window.opener.postMessage(
  'authorization:auth-server:success:{"provider":"auth-server","token":"..."}',
  origin
);

// Generic format
window.opener.postMessage({
  type: 'oauth_callback',
  provider: 'auth-server',
  state: 'success',
  token: '...'
}, '*');
```

## Flow diagram

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Your App  │     │   Worker    │     │ Auth Server │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       │ 1. Open popup     │                   │
       │   /auth           │                   │
       │──────────────────►│                   │
       │                   │                   │
       │                   │ 2. Redirect to    │
       │                   │   /oauth/authorize│
       │                   │──────────────────►│
       │                   │                   │
       │                   │ 3. User login &   │
       │                   │    consent        │
       │                   │◄──────────────────│
       │                   │                   │
       │                   │ 4. Redirect to    │
       │                   │   /callback       │
       │                   │◄──────────────────│
       │                   │                   │
       │                   │ 5. Exchange code  │
       │                   │   for token       │
       │                   │──────────────────►│
       │                   │                   │
       │                   │ 6. Return token   │
       │                   │◄──────────────────│
       │                   │                   │
       │ 7. postMessage    │                   │
       │   with token      │                   │
       │◄──────────────────│                   │
       │                   │                   │
```

## Troubleshooting

### CORS errors
- Đảm bảo Auth Server đã enable CORS
- Check redirect_uri khớp chính xác

### Invalid client
- Verify client_id đúng
- Check OAuth client đang active

### CSRF detected
- Cookie có thể đã expire (10 phút)
- Thử lại từ đầu

### Token exchange failed
- Check client_secret (nếu là confidential client)
- Verify code_verifier được gửi đúng

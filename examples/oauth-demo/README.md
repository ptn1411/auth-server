# OAuth2 Demo Application

A comprehensive demo application to test OAuth2 Authorization Code Flow with PKCE.

## Features

- ✅ **Authorization Code Flow with PKCE** - Secure authorization flow
- ✅ **Token Exchange** - Exchange authorization code for tokens
- ✅ **Token Refresh** - Refresh access tokens using refresh token
- ✅ **Token Revocation** - Revoke access/refresh tokens
- ✅ **User Info** - Fetch user information from `/oauth/userinfo`
- ✅ **JWT Decoding** - Decode and view access/ID token contents
- ✅ **OpenID Configuration** - Fetch OpenID Connect discovery document
- ✅ **Available Scopes** - Fetch and display available OAuth scopes

## Quick Start

### 1. Start the Auth Server

```bash
# From project root
cargo run
```

### 2. Start the Frontend (for consent screen)

```bash
cd frontend
npm run dev
```

### 3. Serve the Demo App

You need a simple HTTP server to serve the demo files. Options:

**Using Python:**

```bash
cd examples/oauth-demo
python -m http.server 8080
```

**Using Node.js (npx):**

```bash
cd examples/oauth-demo
npx serve -p 8080
```

**Using PHP:**

```bash
cd examples/oauth-demo
php -S localhost:8080
```

### 4. Create an OAuth Client

1. Go to `http://localhost:5173` (frontend)
2. Login to your account
3. Navigate to "OAuth Clients" in the sidebar
4. Click "Create Client"
5. Fill in:
   - Name: `Demo App`
   - Redirect URIs: `http://localhost:8080/callback.html`
   - Internal Application: unchecked (for external app)
6. Copy the `client_id` (you'll need it)

### 5. Test the OAuth Flow

1. Open `http://localhost:8080` in your browser
2. Enter the configuration:
   - Auth Server URL: `http://localhost:3000`
   - Client ID: (paste from step 4)
   - Redirect URI: `http://localhost:8080/callback.html`
   - Scopes: `openid profile email offline_access`
3. Click "Login with Auth Server"
4. A popup will open with the consent screen
5. Click "Authorize" to grant access
6. The popup will close and tokens will be displayed

## OAuth2 Endpoints Reference

| Endpoint                            | Method | Description                              |
| ----------------------------------- | ------ | ---------------------------------------- |
| `/oauth/authorize`                  | GET    | Start authorization flow                 |
| `/oauth/token`                      | POST   | Exchange code for tokens / Refresh token |
| `/oauth/userinfo`                   | GET    | Get user information                     |
| `/oauth/revoke`                     | POST   | Revoke token                             |
| `/oauth/scopes`                     | GET    | List available scopes                    |
| `/.well-known/openid-configuration` | GET    | OpenID Connect discovery                 |

## Files

- `index.html` - Main demo page with UI
- `style.css` - Styling
- `app.js` - OAuth flow implementation with all features
- `callback.html` - OAuth redirect callback handler

## OAuth Flow Explained

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Demo App   │     │ Auth Server │     │   Frontend  │
│ (localhost: │     │ (localhost: │     │ (localhost: │
│    8080)    │     │    3000)    │     │    5173)    │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       │ 1. Click Login    │                   │
       │──────────────────►│                   │
       │                   │                   │
       │ 2. Open popup to  │                   │
       │    /oauth/authorize                   │
       │──────────────────►│                   │
       │                   │                   │
       │                   │ 3. Redirect to    │
       │                   │    consent screen │
       │                   │──────────────────►│
       │                   │                   │
       │                   │ 4. User approves  │
       │                   │◄──────────────────│
       │                   │                   │
       │ 5. Redirect to    │                   │
       │    callback.html  │                   │
       │    with code      │                   │
       │◄──────────────────│                   │
       │                   │                   │
       │ 6. postMessage    │                   │
       │    to opener      │                   │
       │                   │                   │
       │ 7. Exchange code  │                   │
       │    for tokens     │                   │
       │──────────────────►│                   │
       │                   │                   │
       │ 8. Return tokens  │                   │
       │◄──────────────────│                   │
       │                   │                   │
       │ 9. Get user info  │                   │
       │──────────────────►│                   │
       │                   │                   │
       │ 10. Return user   │                   │
       │◄──────────────────│                   │
```

## Token Refresh Flow

```
┌─────────────┐     ┌─────────────┐
│  Demo App   │     │ Auth Server │
└──────┬──────┘     └──────┬──────┘
       │                   │
       │ POST /oauth/token │
       │ grant_type:       │
       │ refresh_token     │
       │──────────────────►│
       │                   │
       │ New access_token  │
       │◄──────────────────│
```

## Token Revocation Flow

```
┌─────────────┐     ┌─────────────┐
│  Demo App   │     │ Auth Server │
└──────┬──────┘     └──────┬──────┘
       │                   │
       │ POST /oauth/revoke│
       │ token: <token>    │
       │──────────────────►│
       │                   │
       │ 200 OK            │
       │◄──────────────────│
```

## PKCE (Proof Key for Code Exchange)

This demo implements PKCE for enhanced security:

1. **code_verifier**: Random 64-character string generated client-side
2. **code_challenge**: SHA256 hash of code_verifier, base64url encoded
3. **code_challenge_method**: Always "S256"

The code_verifier is stored in sessionStorage and sent during token exchange to prove the same client that initiated the flow is completing it.

## Available Scopes

| Scope            | Description                   |
| ---------------- | ----------------------------- |
| `openid`         | OpenID Connect authentication |
| `profile`        | User profile information      |
| `email`          | User email address            |
| `profile.read`   | Read-only profile access      |
| `email.read`     | Read-only email access        |
| `offline_access` | Request a refresh token       |

## Troubleshooting

### Popup Blocked

- Allow popups for localhost in your browser settings

### CORS Errors

- Make sure the Auth Server is running with CORS enabled
- Check that the redirect URI matches exactly

### Invalid Client

- Verify the client_id is correct
- Check that the OAuth client is active

### Invalid Redirect URI

- The redirect URI must exactly match what's registered
- For external apps, HTTPS is required (except localhost)

### Token Refresh Fails

- Make sure you requested the `offline_access` scope
- Check that the refresh token hasn't expired

### Token Revocation Fails

- The token may have already been revoked
- Check that the client_id matches the token's client

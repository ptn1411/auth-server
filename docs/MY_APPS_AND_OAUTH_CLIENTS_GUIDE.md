# Hướng dẫn My Apps và OAuth Clients

Tài liệu hướng dẫn chi tiết về hai tính năng quản lý ứng dụng trong Auth Server.

---

## Mục lục

1. [Tổng quan](#tổng-quan)
2. [So sánh My Apps vs OAuth Clients](#so-sánh-my-apps-vs-oauth-clients)
3. [My Apps](#my-apps)
   - [Công dụng thực tế](#công-dụng-thực-tế)
   - [API Endpoints](#my-apps-api-endpoints)
   - [Ví dụ sử dụng](#ví-dụ-sử-dụng-my-apps)
4. [OAuth Clients](#oauth-clients)
   - [Công dụng thực tế](#oauth-clients-công-dụng)
   - [API Endpoints](#oauth-clients-api-endpoints)
   - [OAuth2 Flows](#oauth2-flows)
   - [Ví dụ sử dụng](#ví-dụ-sử-dụng-oauth-clients)
5. [Khi nào dùng cái nào?](#khi-nào-dùng-cái-nào)

---

## Tổng quan

Auth Server cung cấp 2 cách để quản lý ứng dụng:

| Tính năng | Mục đích chính |
|-----------|----------------|
| **My Apps** | Quản lý users nội bộ trong app của bạn |
| **OAuth Clients** | Cho phép app bên thứ 3 truy cập qua OAuth2 |

---

## So sánh My Apps vs OAuth Clients

| Tính năng | My Apps | OAuth Clients |
|-----------|---------|---------------|
| **Mục đích** | Quản lý users trong app | Cho phép app bên thứ 3 truy cập |
| **Authentication** | App ID + Secret | OAuth2 flows (PKCE, etc.) |
| **User consent** | Không cần | Bắt buộc (external apps) |
| **Scopes** | Không có | Có (email, profile, etc.) |
| **Token type** | App token | OAuth2 access/refresh tokens |
| **Quản lý users** | Ban/Unban, Roles, Permissions | Không có |
| **Use case** | Backend services, Multi-tenant | Third-party integrations, "Login with X" |

---

## My Apps

### Công dụng thực tế

**My Apps** cho phép bạn tạo và quản lý các ứng dụng sử dụng Auth Server làm hệ thống xác thực trung tâm.

#### Mỗi App bao gồm:
- **App ID** (UUID) - định danh duy nhất
- **App Code** - mã ngắn gọn (VD: `my_shop_1735689600000`)
- **App Secret** - khóa bí mật để xác thực

#### Chức năng chính:

1. **Quản lý App** - Tạo, xem, đổi secret
2. **Quản lý Users trong App** - Đăng ký, ban/unban, xóa users
3. **Quản lý Roles & Permissions** - Phân quyền users trong app

### My Apps API Endpoints

#### Quản lý App

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| POST | `/apps` | Tạo app mới |
| GET | `/apps` | Liệt kê apps của bạn |
| GET | `/apps/{id}` | Xem chi tiết app |
| POST | `/apps/{id}/secret/regenerate` | Đổi secret mới |
| POST | `/apps/auth` | Xác thực app (lấy token) |

#### Quản lý Users trong App

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| POST | `/apps/{app_id}/register` | User đăng ký vào app |
| POST | `/apps/{app_id}/users/{user_id}/ban` | Ban user |
| POST | `/apps/{app_id}/users/{user_id}/unban` | Unban user |
| DELETE | `/apps/{app_id}/users/{user_id}` | Xóa user khỏi app |
| GET | `/apps/{app_id}/users` | Liệt kê users |

#### Quản lý Roles

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| POST | `/apps/{app_id}/roles` | Tạo role mới |
| GET | `/apps/{app_id}/roles` | Liệt kê roles |
| POST | `/apps/{app_id}/users/{user_id}/roles` | Gán role cho user |
| DELETE | `/apps/{app_id}/users/{user_id}/roles/{role_id}` | Xóa role |
| GET | `/apps/{app_id}/users/{user_id}/roles` | Xem roles của user |

### Ví dụ sử dụng My Apps

#### Scenario: Hệ thống E-commerce với nhiều apps

```
┌─────────────────────────────────────────────────────────────┐
│                    AUTH SERVER (Central)                     │
│  Users: 10,000 accounts                                      │
└─────────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
    │  SHOP APP   │     │  ADMIN APP  │     │ MOBILE APP  │
    │ Users: 8000 │     │ Users: 50   │     │ Users: 5000 │
    │ Roles:      │     │ Roles:      │     │ Roles:      │
    │ - customer  │     │ - admin     │     │ - user      │
    │ - vip       │     │ - moderator │     │ - premium   │
    └─────────────┘     └─────────────┘     └─────────────┘
```

#### 1. Tạo App

```bash
curl -X POST https://auth.example.com/apps \
  -H "Authorization: Bearer {your_jwt}" \
  -H "Content-Type: application/json" \
  -d '{"code": "shop", "name": "E-commerce Shop"}'
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "code": "shop_1735689600000",
  "name": "E-commerce Shop",
  "secret": "sk_live_abc123xyz789..."
}
```

> ⚠️ **Lưu ý:** Secret chỉ hiển thị 1 lần duy nhất!

#### 2. User đăng ký vào App

```bash
curl -X POST https://auth.example.com/apps/550e8400.../register \
  -H "Authorization: Bearer {user_jwt}"
```

**Response:**
```json
{
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "app_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "active",
  "banned_at": null,
  "banned_reason": null,
  "created_at": "2024-12-31T10:30:00Z"
}
```

#### 3. Tạo Role cho App

```bash
curl -X POST https://auth.example.com/apps/550e8400.../roles \
  -H "Authorization: Bearer {owner_jwt}" \
  -H "Content-Type: application/json" \
  -d '{"name": "vip"}'
```

#### 4. Gán Role cho User

```bash
curl -X POST https://auth.example.com/apps/550e8400.../users/{user_id}/roles \
  -H "Authorization: Bearer {owner_jwt}" \
  -H "Content-Type: application/json" \
  -d '{"role_id": "role_uuid_here"}'
```

#### 5. Ban User vi phạm

```bash
curl -X POST https://auth.example.com/apps/550e8400.../users/{user_id}/ban \
  -H "Authorization: Bearer {owner_jwt}" \
  -H "Content-Type: application/json" \
  -d '{"reason": "Fraudulent activity detected"}'
```

#### 6. App Authentication (Machine-to-Machine)

Backend của app xác thực để gọi API:

```bash
curl -X POST https://auth.example.com/apps/auth \
  -H "Content-Type: application/json" \
  -d '{
    "app_id": "550e8400-e29b-41d4-a716-446655440001",
    "secret": "sk_live_abc123xyz789..."
  }'
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

#### 7. Liệt kê Users trong App

```bash
curl -X GET "https://auth.example.com/apps/550e8400.../users?page=1&limit=20" \
  -H "Authorization: Bearer {owner_jwt}"
```

**Response:**
```json
{
  "data": [
    {
      "user_id": "user-uuid-1",
      "email": "customer1@example.com",
      "status": "active",
      "roles": ["customer", "vip"],
      "banned_at": null,
      "banned_reason": null,
      "created_at": "2024-12-01T10:00:00Z"
    },
    {
      "user_id": "user-uuid-2",
      "email": "baduser@example.com",
      "status": "banned",
      "roles": ["customer"],
      "banned_at": "2024-12-15T14:30:00Z",
      "banned_reason": "Fraudulent activity detected",
      "created_at": "2024-12-05T08:00:00Z"
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 8000
}
```

### Các trường hợp lỗi khi đăng ký App

| Trường hợp | HTTP Status | Error |
|------------|-------------|-------|
| User đã đăng ký | 409 | `user_already_registered` |
| User bị ban | 403 | `user_banned` |
| App không tồn tại | 404 | `app_not_found` |

---

## OAuth Clients

### OAuth Clients Công dụng

**OAuth Clients** là các ứng dụng bên thứ 3 muốn truy cập tài nguyên của user thông qua OAuth2 protocol.

#### Cho phép:
- **"Login with Your Service"** (như "Login with Google")
- **Truy cập API** với quyền hạn giới hạn (scopes)
- **User consent** - user phải đồng ý cấp quyền

### OAuth Clients API Endpoints

#### Quản lý OAuth Clients

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| POST | `/oauth/clients` | Đăng ký client mới |
| GET | `/oauth/clients` | Liệt kê clients |
| PUT | `/oauth/clients/{id}` | Cập nhật client |
| DELETE | `/oauth/clients/{id}` | Xóa client |
| POST | `/oauth/clients/{id}/secret` | Đổi secret mới |

#### OAuth2 Flow

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| GET | `/oauth/authorize` | Bắt đầu authorization flow |
| POST | `/oauth/authorize/callback` | Xử lý consent decision |
| POST | `/oauth/token` | Đổi code lấy tokens |
| POST | `/oauth/revoke` | Thu hồi token |
| GET | `/oauth/userinfo` | Lấy thông tin user |
| GET | `/oauth/scopes` | Liệt kê scopes |
| GET | `/.well-known/openid-configuration` | OpenID discovery |

#### User Consent Management

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| GET | `/account/connected-apps` | Xem apps đã kết nối |
| DELETE | `/account/connected-apps/{client_id}` | Thu hồi quyền truy cập |

### OAuth2 Flows

#### 1. Authorization Code Flow (với PKCE) - External Apps

```
User          Partner Website       Auth Server
  │                 │                    │
  │ Click Login     │                    │
  │────────────────▶│                    │
  │                 │                    │
  │    Redirect to /oauth/authorize      │
  │◀────────────────│                    │
  │                 │                    │
  │         Login + Consent Screen       │
  │─────────────────────────────────────▶│
  │                 │                    │
  │    Redirect with authorization code  │
  │◀─────────────────────────────────────│
  │                 │                    │
  │                 │ Exchange code      │
  │                 │ for tokens         │
  │                 │───────────────────▶│
  │                 │                    │
  │                 │ Access + Refresh   │
  │                 │ tokens             │
  │                 │◀───────────────────│
```

#### 2. Client Credentials Flow - Internal Apps (M2M)

Dùng cho backend services giao tiếp với nhau, không cần user consent.

#### 3. Refresh Token Flow

Dùng để làm mới access token khi hết hạn.

### Ví dụ sử dụng OAuth Clients

#### Scenario: "Login with MyAuth" cho Partner Website

##### Bước 1: Đăng ký OAuth Client

```bash
curl -X POST https://auth.example.com/oauth/clients \
  -H "Authorization: Bearer {admin_jwt}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Partner Website",
    "redirect_uris": ["https://partner.com/callback"],
    "is_internal": false
  }'
```

**Response:**
```json
{
  "client_id": "550e8400-e29b-41d4-a716-446655440001",
  "client_secret": "cs_live_abc123xyz789...",
  "name": "Partner Website",
  "redirect_uris": ["https://partner.com/callback"],
  "is_internal": false
}
```

> ⚠️ **Lưu ý:** `client_secret` chỉ hiển thị 1 lần!

##### Bước 2: User click "Login with MyAuth"

Partner website redirect user đến:

```
https://auth.example.com/oauth/authorize?
  response_type=code&
  client_id=550e8400-e29b-41d4-a716-446655440001&
  redirect_uri=https://partner.com/callback&
  scope=email profile&
  code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&
  code_challenge_method=S256&
  state=random_state_123
```

##### Bước 3: User đồng ý (Consent Screen)

```
┌─────────────────────────────────────────────┐
│         Partner Website                      │
│         muốn truy cập tài khoản của bạn     │
│                                              │
│  Quyền được yêu cầu:                        │
│  ✓ Xem email của bạn                        │
│  ✓ Xem thông tin profile                    │
│                                              │
│  [Từ chối]              [Đồng ý]            │
└─────────────────────────────────────────────┘
```

##### Bước 4: Redirect với Authorization Code

```
https://partner.com/callback?
  code=AUTH_CODE_HERE&
  state=random_state_123
```

##### Bước 5: Partner đổi Code lấy Tokens

```bash
curl -X POST https://auth.example.com/oauth/token \
  -d "grant_type=authorization_code" \
  -d "code=AUTH_CODE_HERE" \
  -d "client_id=550e8400-e29b-41d4-a716-446655440001" \
  -d "client_secret=cs_live_abc123xyz789..." \
  -d "redirect_uri=https://partner.com/callback" \
  -d "code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "rt_xyz789...",
  "token_type": "Bearer",
  "expires_in": 900,
  "scope": "email profile"
}
```

##### Bước 6: Lấy thông tin User

```bash
curl -X GET https://auth.example.com/oauth/userinfo \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIs..."
```

**Response:**
```json
{
  "sub": "user-uuid-123",
  "email": "user@example.com",
  "email_verified": true,
  "name": "user@example.com"
}
```

### Client Credentials Flow (Internal Apps)

```bash
curl -X POST https://auth.example.com/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=internal-service-id" \
  -d "client_secret=secret123" \
  -d "scope=read:users"
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "read:users"
}
```

### Refresh Token

```bash
curl -X POST https://auth.example.com/oauth/token \
  -d "grant_type=refresh_token" \
  -d "refresh_token=rt_xyz789..." \
  -d "client_id=550e8400..."
```

### User quản lý Connected Apps

#### Xem apps đã kết nối

```bash
curl -X GET https://auth.example.com/account/connected-apps \
  -H "Authorization: Bearer {user_jwt}"
```

**Response:**
```json
{
  "apps": [
    {
      "client_id": "550e8400...",
      "name": "Partner Website",
      "scopes": ["email", "profile"],
      "granted_at": "2024-12-01T10:00:00Z"
    }
  ]
}
```

#### Thu hồi quyền

```bash
curl -X DELETE https://auth.example.com/account/connected-apps/550e8400... \
  -H "Authorization: Bearer {user_jwt}"
```

### OAuth Scopes

| Scope | Cho phép truy cập |
|-------|-------------------|
| `openid` | User ID (sub) |
| `email` | Email + email_verified |
| `profile` | Tên, avatar, etc. |

### Phân loại OAuth Clients

| Loại | PKCE | User Consent | Use case |
|------|------|--------------|----------|
| **External** (`is_internal: false`) | ✅ Bắt buộc | ✅ Bắt buộc | Third-party websites |
| **Internal** (`is_internal: true`) | ❌ Không cần | ❌ Không cần | Backend services |

---

## Khi nào dùng cái nào?

### Dùng My Apps khi:

- ✅ Bạn cần quản lý users trong ứng dụng của mình
- ✅ Cần phân quyền (roles/permissions) cho users
- ✅ Cần ban/unban users
- ✅ Backend services cần xác thực với nhau
- ✅ Multi-tenant SaaS (mỗi tenant = 1 app)

### Dùng OAuth Clients khi:

- ✅ Cho phép website/app bên thứ 3 "Login with Your Service"
- ✅ Cần user consent trước khi chia sẻ thông tin
- ✅ Cần giới hạn quyền truy cập qua scopes
- ✅ Partner integrations
- ✅ Mobile apps truy cập API của bạn

### Ví dụ thực tế

| Scenario | Giải pháp |
|----------|-----------|
| Shop app quản lý customers | **My Apps** |
| Partner website muốn "Login with MyAuth" | **OAuth Clients** |
| Backend service gọi API nội bộ | **My Apps** (app auth) |
| Mobile app truy cập user data | **OAuth Clients** |
| Ban user gian lận khỏi 1 app | **My Apps** |
| User muốn thu hồi quyền của 1 app | **OAuth Clients** (connected apps) |

---

## Tổng kết

```
┌─────────────────────────────────────────────────────────────┐
│                      AUTH SERVER                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐          ┌─────────────────┐           │
│  │    MY APPS      │          │  OAUTH CLIENTS  │           │
│  │                 │          │                 │           │
│  │ • User mgmt     │          │ • OAuth2 flows  │           │
│  │ • Roles/Perms   │          │ • User consent  │           │
│  │ • Ban/Unban     │          │ • Scopes        │           │
│  │ • App auth      │          │ • Third-party   │           │
│  │                 │          │                 │           │
│  └────────┬────────┘          └────────┬────────┘           │
│           │                            │                     │
│           ▼                            ▼                     │
│  ┌─────────────────┐          ┌─────────────────┐           │
│  │ Internal Apps   │          │ External Apps   │           │
│  │ Backend Services│          │ Partner Sites   │           │
│  │ Multi-tenant    │          │ Mobile Apps     │           │
│  └─────────────────┘          └─────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

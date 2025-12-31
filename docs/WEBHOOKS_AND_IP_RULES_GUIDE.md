# Hướng dẫn Webhooks và IP Rules

Tài liệu chi tiết về tính năng Webhooks và IP Rules trong Auth Server.

---

## Mục lục

1. [Webhooks](#webhooks)
   - [Tổng quan](#tổng-quan-webhooks)
   - [Webhook Events](#webhook-events)
   - [API Endpoints](#webhook-api-endpoints)
   - [Webhook Payload](#webhook-payload)
   - [Webhook Signature](#webhook-signature)
   - [Retry Logic](#retry-logic)
   - [Ví dụ tích hợp](#ví-dụ-tích-hợp-webhooks)
2. [IP Rules](#ip-rules)
   - [Tổng quan](#tổng-quan-ip-rules)
   - [Loại Rules](#loại-ip-rules)
   - [API Endpoints](#ip-rules-api-endpoints)
   - [Cách hoạt động](#cách-hoạt-động-ip-rules)
   - [Ví dụ sử dụng](#ví-dụ-sử-dụng-ip-rules)

---

## Webhooks

### Tổng quan Webhooks

Webhooks cho phép app của bạn nhận thông báo HTTP POST real-time khi có sự kiện xảy ra. Thay vì phải polling API liên tục, Auth Server sẽ chủ động gửi thông báo đến URL bạn cấu hình.

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│   User Action   │ ──────▶ │   Auth Server   │ ──────▶ │   Your Server   │
│  (login, ban)   │         │  trigger event  │  POST   │  /webhooks      │
└─────────────────┘         └─────────────────┘         └─────────────────┘
```

### Webhook Events

#### User Events

| Event | Mô tả | Trigger Point |
|-------|-------|---------------|
| `user.registered` | User đăng ký tài khoản mới | POST /auth/register |
| `user.login` | User đăng nhập thành công | POST /auth/login (với app_id) |
| `user.logout` | User đăng xuất | POST /auth/logout |
| `user.password_changed` | User đổi mật khẩu | POST /users/me/change-password |
| `user.password_reset` | User reset mật khẩu | POST /auth/reset-password |
| `user.email_verified` | User xác thực email | POST /auth/verify-email |
| `user.mfa_enabled` | User bật MFA | POST /auth/mfa/totp/verify |
| `user.mfa_disabled` | User tắt MFA | DELETE /auth/mfa |
| `user.locked` | User bị khóa (login sai nhiều lần) | Automatic |
| `user.unlocked` | User được mở khóa | POST /admin/users/{id}/unlock |
| `user.deactivated` | User bị vô hiệu hóa | POST /admin/users/{id}/deactivate |
| `user.activated` | User được kích hoạt lại | POST /admin/users/{id}/activate |

#### User-App Events

| Event | Mô tả | Trigger Point |
|-------|-------|---------------|
| `user.app.joined` | User đăng ký vào app | POST /apps/{app_id}/register |
| `user.app.banned` | User bị ban khỏi app | POST /apps/{app_id}/users/{id}/ban |
| `user.app.unbanned` | User được unban | POST /apps/{app_id}/users/{id}/unban |
| `user.app.removed` | User bị xóa khỏi app | DELETE /apps/{app_id}/users/{id} |

#### App Events

| Event | Mô tả | Trigger Point |
|-------|-------|---------------|
| `app.created` | App mới được tạo | POST /apps |
| `app.secret_regenerated` | App secret được đổi | POST /apps/{id}/secret/regenerate |

#### Role Events

| Event | Mô tả | Trigger Point |
|-------|-------|---------------|
| `role.assigned` | Role được gán cho user | POST /apps/{app_id}/users/{id}/roles |
| `role.removed` | Role bị xóa khỏi user | DELETE /apps/{app_id}/users/{id}/roles/{role_id} |

### Webhook API Endpoints

| Method | Endpoint | Chức năng | Auth |
|--------|----------|-----------|------|
| POST | `/apps/{app_id}/webhooks` | Tạo webhook mới | JWT (owner) |
| GET | `/apps/{app_id}/webhooks` | Liệt kê webhooks | JWT (owner) |
| GET | `/apps/{app_id}/webhooks/{id}` | Xem chi tiết | JWT (owner) |
| PUT | `/apps/{app_id}/webhooks/{id}` | Cập nhật webhook | JWT (owner) |
| DELETE | `/apps/{app_id}/webhooks/{id}` | Xóa webhook | JWT (owner) |

#### Tạo Webhook

```bash
curl -X POST https://auth.example.com/apps/{app_id}/webhooks \
  -H "Authorization: Bearer {jwt_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-server.com/webhooks/auth",
    "events": ["user.login", "user.app.banned", "user.app.joined"]
  }'
```

**Response:**
```json
{
  "id": "webhook-uuid",
  "app_id": "app-uuid",
  "url": "https://your-server.com/webhooks/auth",
  "secret": "whsec_abc123xyz789...",
  "events": ["user.login", "user.app.banned", "user.app.joined"],
  "is_active": true,
  "created_at": "2024-12-31T10:00:00Z"
}
```

> ⚠️ **Quan trọng:** `secret` chỉ hiển thị 1 lần khi tạo webhook. Lưu lại để verify signature!

#### Cập nhật Webhook

```bash
curl -X PUT https://auth.example.com/apps/{app_id}/webhooks/{webhook_id} \
  -H "Authorization: Bearer {jwt_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://new-server.com/webhooks",
    "events": ["user.login"],
    "is_active": false
  }'
```

### Webhook Payload

Khi event xảy ra, Auth Server gửi HTTP POST đến webhook URL với payload:

#### Headers

| Header | Mô tả |
|--------|-------|
| `Content-Type` | `application/json` |
| `X-Webhook-Event` | Tên event (VD: `user.login`) |
| `X-Webhook-Signature` | HMAC-SHA256 signature |
| `X-Webhook-Timestamp` | Unix timestamp khi gửi |

#### Body Examples

**user.login:**
```json
{
  "event": "user.login",
  "user_id": "user-uuid",
  "app_id": "app-uuid",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "session_id": "session-uuid",
  "timestamp": "2024-12-31T10:30:00Z"
}
```

**user.app.banned:**
```json
{
  "event": "user.app.banned",
  "user_id": "user-uuid",
  "app_id": "app-uuid",
  "banned_by": "admin-uuid",
  "reason": "Violation of terms",
  "timestamp": "2024-12-31T10:30:00Z"
}
```

**user.app.joined:**
```json
{
  "event": "user.app.joined",
  "user_id": "user-uuid",
  "app_id": "app-uuid",
  "status": "active",
  "timestamp": "2024-12-31T10:30:00Z"
}
```

**user.app.unbanned:**
```json
{
  "event": "user.app.unbanned",
  "user_id": "user-uuid",
  "app_id": "app-uuid",
  "unbanned_by": "admin-uuid",
  "timestamp": "2024-12-31T10:30:00Z"
}
```

**user.app.removed:**
```json
{
  "event": "user.app.removed",
  "user_id": "user-uuid",
  "app_id": "app-uuid",
  "removed_by": "admin-uuid",
  "timestamp": "2024-12-31T10:30:00Z"
}
```

### Webhook Signature

Để đảm bảo webhook request đến từ Auth Server, verify signature bằng HMAC-SHA256:

```
signature = HMAC-SHA256(webhook_secret, payload_body)
```

#### Verify trong Node.js

```javascript
const crypto = require('crypto');

function verifyWebhookSignature(payload, signature, secret) {
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expectedSignature)
  );
}

// Express middleware
app.post('/webhooks/auth', express.raw({ type: 'application/json' }), (req, res) => {
  const signature = req.headers['x-webhook-signature'];
  const timestamp = req.headers['x-webhook-timestamp'];
  const payload = req.body.toString();
  
  // Verify signature
  if (!verifyWebhookSignature(payload, signature, WEBHOOK_SECRET)) {
    return res.status(401).send('Invalid signature');
  }
  
  // Verify timestamp (prevent replay attacks - 5 min tolerance)
  const now = Math.floor(Date.now() / 1000);
  if (Math.abs(now - parseInt(timestamp)) > 300) {
    return res.status(401).send('Timestamp too old');
  }
  
  const event = JSON.parse(payload);
  console.log('Received event:', event.event);
  
  // Process event...
  
  res.status(200).send('OK');
});
```

#### Verify trong Python

```python
import hmac
import hashlib
import time

def verify_webhook_signature(payload: bytes, signature: str, secret: str) -> bool:
    expected = hmac.new(
        secret.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)

# Flask example
@app.route('/webhooks/auth', methods=['POST'])
def handle_webhook():
    signature = request.headers.get('X-Webhook-Signature')
    timestamp = request.headers.get('X-Webhook-Timestamp')
    payload = request.get_data()
    
    # Verify signature
    if not verify_webhook_signature(payload, signature, WEBHOOK_SECRET):
        return 'Invalid signature', 401
    
    # Verify timestamp
    if abs(time.time() - int(timestamp)) > 300:
        return 'Timestamp too old', 401
    
    event = request.get_json()
    print(f"Received event: {event['event']}")
    
    return 'OK', 200
```

#### Verify trong Rust

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn verify_webhook_signature(payload: &str, signature: &str, secret: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}
```

### Retry Logic

Auth Server tự động retry webhook deliveries khi thất bại:

| Attempt | Delay | Tổng thời gian |
|---------|-------|----------------|
| 1 | Ngay lập tức | 0 |
| 2 | 5 phút | 5 phút |
| 3 | 5 phút | 10 phút |
| 4 | 5 phút | 15 phút |
| 5 | 5 phút | 20 phút |

- **Max attempts:** 5 lần
- **Timeout:** 30 giây mỗi request
- **Success:** HTTP 2xx response
- **Failure:** HTTP 4xx/5xx hoặc timeout

### Ví dụ tích hợp Webhooks

#### Use Case: Sync user data khi có thay đổi

```javascript
// webhook-handler.js
const express = require('express');
const app = express();

app.post('/webhooks/auth', express.json(), async (req, res) => {
  const { event, user_id, app_id } = req.body;
  
  switch (event) {
    case 'user.app.joined':
      // Tạo profile mới trong database của bạn
      await createUserProfile(user_id, app_id);
      break;
      
    case 'user.app.banned':
      // Disable user trong hệ thống
      await disableUser(user_id);
      // Gửi email thông báo
      await sendBanNotification(user_id, req.body.reason);
      break;
      
    case 'user.app.unbanned':
      // Re-enable user
      await enableUser(user_id);
      break;
      
    case 'user.app.removed':
      // Cleanup user data
      await deleteUserData(user_id, app_id);
      break;
      
    case 'user.login':
      // Track login activity
      await logUserActivity(user_id, 'login', req.body.ip_address);
      break;
  }
  
  res.status(200).send('OK');
});
```

---

## IP Rules

### Tổng quan IP Rules

IP Rules cho phép bạn kiểm soát truy cập dựa trên địa chỉ IP. Có thể áp dụng:
- **Global:** Áp dụng cho toàn bộ Auth Server
- **Per-App:** Chỉ áp dụng cho một app cụ thể

```
┌─────────────────────────────────────────────────────────────┐
│                        Request                               │
│                    IP: 192.168.1.100                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     IP Rules Check                           │
│  1. Check App-specific rules (if app_id provided)           │
│  2. Check Global rules                                       │
│  3. Whitelist > Blacklist (whitelist wins)                  │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
        ┌──────────┐                   ┌──────────┐
        │ ALLOWED  │                   │ BLOCKED  │
        └──────────┘                   └──────────┘
```

### Loại IP Rules

| Type | Mô tả |
|------|-------|
| `whitelist` | Cho phép IP này truy cập (ưu tiên cao hơn blacklist) |
| `blacklist` | Chặn IP này truy cập |

### IP Rules API Endpoints

#### Global Rules (Admin only)

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| POST | `/admin/ip-rules` | Tạo global rule |
| GET | `/admin/ip-rules` | Liệt kê global rules |
| GET | `/admin/ip-rules/check?ip={ip}` | Kiểm tra IP |
| DELETE | `/admin/ip-rules/{rule_id}` | Xóa rule |

#### App-specific Rules (App owner)

| Method | Endpoint | Chức năng |
|--------|----------|-----------|
| POST | `/apps/{app_id}/ip-rules` | Tạo rule cho app |
| GET | `/apps/{app_id}/ip-rules` | Liệt kê rules của app |

### Cách hoạt động IP Rules

IP Rules được check tại các điểm sau:

#### 1. Login với app_id

Khi user login với `app_id`, IP được check trước khi cho phép đăng nhập:

```bash
curl -X POST https://auth.example.com/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123",
    "app_id": "app-uuid"
  }'
```

Nếu IP bị block → Response 403:
```json
{
  "error": "user_banned",
  "message": "IP address 192.168.1.100 is blocked for this app"
}
```

#### 2. Register to App

Khi user đăng ký vào app:

```bash
curl -X POST https://auth.example.com/apps/{app_id}/register \
  -H "Authorization: Bearer {jwt_token}"
```

#### 3. API Key Authentication

Khi sử dụng API Key, IP được check cho app tương ứng:

```bash
curl -X GET https://auth.example.com/api/v1/users \
  -H "X-API-Key: ak_live_abc123..."
```

### IP Rules API Examples

#### Tạo Global Blacklist Rule

```bash
curl -X POST https://auth.example.com/admin/ip-rules \
  -H "Authorization: Bearer {admin_jwt}" \
  -H "Content-Type: application/json" \
  -d '{
    "ip_address": "192.168.1.100",
    "rule_type": "blacklist",
    "reason": "Suspicious activity detected",
    "expires_at": "2025-01-31T23:59:59Z"
  }'
```

**Response:**
```json
{
  "id": "rule-uuid",
  "app_id": null,
  "ip_address": "192.168.1.100",
  "ip_range": null,
  "rule_type": "blacklist",
  "reason": "Suspicious activity detected",
  "expires_at": "2025-01-31T23:59:59Z",
  "created_by": "admin-uuid",
  "created_at": "2024-12-31T10:00:00Z"
}
```

#### Tạo App-specific Whitelist Rule

```bash
curl -X POST https://auth.example.com/apps/{app_id}/ip-rules \
  -H "Authorization: Bearer {owner_jwt}" \
  -H "Content-Type: application/json" \
  -d '{
    "ip_address": "10.0.0.0",
    "ip_range": "10.0.0.0/8",
    "rule_type": "whitelist",
    "reason": "Internal network"
  }'
```

#### Kiểm tra IP

```bash
curl -X GET "https://auth.example.com/admin/ip-rules/check?ip=192.168.1.100" \
  -H "Authorization: Bearer {admin_jwt}"
```

**Response:**
```json
{
  "ip": "192.168.1.100",
  "result": "blocked",
  "matched_rule": {
    "id": "rule-uuid",
    "rule_type": "blacklist",
    "reason": "Suspicious activity detected"
  }
}
```

#### Liệt kê Rules

```bash
# Global rules
curl -X GET https://auth.example.com/admin/ip-rules \
  -H "Authorization: Bearer {admin_jwt}"

# App-specific rules
curl -X GET https://auth.example.com/apps/{app_id}/ip-rules \
  -H "Authorization: Bearer {owner_jwt}"
```

**Response:**
```json
[
  {
    "id": "rule-uuid-1",
    "app_id": null,
    "ip_address": "192.168.1.100",
    "ip_range": null,
    "rule_type": "blacklist",
    "reason": "Suspicious activity",
    "expires_at": "2025-01-31T23:59:59Z",
    "is_active": true,
    "created_at": "2024-12-31T10:00:00Z"
  },
  {
    "id": "rule-uuid-2",
    "app_id": "app-uuid",
    "ip_address": "10.0.0.0",
    "ip_range": "10.0.0.0/8",
    "rule_type": "whitelist",
    "reason": "Internal network",
    "expires_at": null,
    "is_active": true,
    "created_at": "2024-12-31T11:00:00Z"
  }
]
```

### Ví dụ sử dụng IP Rules

#### Use Case 1: Block IP sau khi phát hiện attack

```bash
# 1. Phát hiện IP đáng ngờ từ logs
# 2. Block IP ngay lập tức (expires sau 24h)

curl -X POST https://auth.example.com/admin/ip-rules \
  -H "Authorization: Bearer {admin_jwt}" \
  -H "Content-Type: application/json" \
  -d '{
    "ip_address": "203.0.113.50",
    "rule_type": "blacklist",
    "reason": "Brute force attack detected",
    "expires_at": "2025-01-01T10:00:00Z"
  }'
```

#### Use Case 2: Whitelist office IP range

```bash
# Cho phép tất cả IP từ office network
curl -X POST https://auth.example.com/apps/{app_id}/ip-rules \
  -H "Authorization: Bearer {owner_jwt}" \
  -H "Content-Type: application/json" \
  -d '{
    "ip_address": "192.168.0.0",
    "ip_range": "192.168.0.0/16",
    "rule_type": "whitelist",
    "reason": "Office network"
  }'
```

#### Use Case 3: Restrict app to specific IPs only

```bash
# 1. Blacklist all (0.0.0.0)
curl -X POST https://auth.example.com/apps/{app_id}/ip-rules \
  -H "Authorization: Bearer {owner_jwt}" \
  -d '{"ip_address": "0.0.0.0", "rule_type": "blacklist", "reason": "Block all by default"}'

# 2. Whitelist specific IPs
curl -X POST https://auth.example.com/apps/{app_id}/ip-rules \
  -H "Authorization: Bearer {owner_jwt}" \
  -d '{"ip_address": "203.0.113.10", "rule_type": "whitelist", "reason": "Production server"}'

curl -X POST https://auth.example.com/apps/{app_id}/ip-rules \
  -H "Authorization: Bearer {owner_jwt}" \
  -d '{"ip_address": "203.0.113.20", "rule_type": "whitelist", "reason": "Staging server"}'
```

### IP Rules Priority

Khi có nhiều rules match:

1. **Whitelist luôn thắng Blacklist** - Nếu IP match cả whitelist và blacklist, IP được cho phép
2. **App-specific rules check trước** - Nếu có app_id, check app rules trước
3. **Expired rules bị bỏ qua** - Rules có `expires_at` trong quá khứ không có hiệu lực
4. **No rule = Allowed** - Nếu không có rule nào match, IP được cho phép

---

## Configuration

### Environment Variables

```bash
# Webhook worker interval (seconds)
WEBHOOK_WORKER_INTERVAL_SECS=10
```

### Webhook Worker

Auth Server chạy background worker để process pending webhook deliveries:

- **Polling interval:** Configurable (default 10s)
- **Batch size:** 100 deliveries per cycle
- **Graceful shutdown:** Worker stops khi server shutdown

---

## Best Practices

### Webhooks

1. **Luôn verify signature** - Đừng trust webhook request mà không verify
2. **Respond nhanh** - Return 200 ngay, process async nếu cần
3. **Idempotent handling** - Webhook có thể được gửi nhiều lần (retry)
4. **Log webhook events** - Để debug và audit
5. **Use HTTPS** - Webhook URL phải là HTTPS (trừ localhost)

### IP Rules

1. **Whitelist > Blacklist** - Dùng whitelist cho trusted IPs
2. **Set expiry** - Đặt expires_at cho temporary blocks
3. **Document reasons** - Luôn ghi reason để biết tại sao block
4. **Monitor logs** - Theo dõi blocked requests
5. **Test trước khi apply** - Dùng `/check` endpoint để test

---

## Troubleshooting

### Webhook không nhận được

1. Check webhook `is_active` = true
2. Check URL accessible từ Auth Server
3. Check events array có event cần nhận
4. Check webhook delivery logs (nếu có)
5. Verify SSL certificate (nếu HTTPS)

### IP bị block nhầm

1. Check với `/admin/ip-rules/check?ip={ip}`
2. Xem rule nào match
3. Thêm whitelist rule nếu cần
4. Hoặc xóa blacklist rule

### Webhook signature không match

1. Verify đang dùng đúng secret
2. Verify payload là raw body (không parse trước)
3. Check encoding (UTF-8)
4. Verify algorithm là HMAC-SHA256

# Auth Server API Tests

Bộ test API sử dụng Jest + Supertest để kiểm tra tất cả các endpoint của Auth Server.

## Cài đặt

```bash
cd tests
npm install
```

## Cấu hình

Tạo file `.env` với nội dung:

```env
API_URL=http://localhost:3000
TEST_ADMIN_EMAIL=admin@test.com
TEST_ADMIN_PASSWORD=Admin123!@#
```

## Chạy tests

### Yêu cầu
- Auth Server phải đang chạy (`cargo run --release`)
- Database đã được migrate
- Có user admin với `is_system_admin = true`

### Chạy tất cả tests

```bash
npm test
```

### Chạy từng nhóm test

```bash
npm run test:auth      # Authentication tests
npm run test:admin     # Admin API tests
npm run test:apps      # Apps management tests
npm run test:security  # Security features tests
```

### Chạy với coverage

```bash
npm run test:coverage
```

## Cấu trúc tests

```
tests/
├── package.json          # Dependencies
├── jest.config.js        # Jest configuration
├── setup.js              # Global test setup
├── helpers.js            # Test utilities
├── .env                  # Environment variables
├── health.test.js        # Health check endpoints
├── auth.test.js          # Authentication (register, login, refresh)
├── user-profile.test.js  # User profile management
├── apps.test.js          # App management
├── roles.test.js         # Role assignment
├── admin.test.js         # Admin CRUD operations
└── security.test.js      # Security features (MFA, sessions, audit)
```

## Test Coverage

| Module | Endpoints |
|--------|-----------|
| Health | `/health`, `/ready` |
| Auth | `/auth/register`, `/auth/login`, `/auth/refresh`, `/auth/forgot-password` |
| Profile | `/users/me`, `/users/me/change-password` |
| Apps | `/apps`, `/apps/:id/roles`, `/apps/:id/permissions`, `/apps/auth` |
| Roles | `/apps/:id/users/:id/roles` |
| Admin Users | `/admin/users`, `/admin/users/:id`, `/admin/users/:id/activate`, `/admin/users/:id/deactivate` |
| Admin Apps | `/admin/apps`, `/admin/apps/:id` |
| Security | `/auth/logout`, `/auth/sessions`, `/auth/mfa/*`, `/auth/audit-logs` |

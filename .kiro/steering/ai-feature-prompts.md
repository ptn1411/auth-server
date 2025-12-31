# AI Feature Development Prompts

Bộ câu lệnh chuẩn để yêu cầu AI tạo chức năng mới theo đúng coding standards của dự án auth-server.

## 1. Tạo CRUD Endpoint Hoàn Chỉnh

### Prompt Template:
```
Tạo CRUD endpoints cho entity [TÊN_ENTITY] với các fields:
- [field1]: [type] (required/optional)
- [field2]: [type] (required/optional)
...

Yêu cầu:
1. Tạo migration SQL trong migrations/
2. Tạo Model trong src/models/
3. Tạo DTOs (Request/Response) trong src/dto/
4. Tạo Repository trong src/repositories/
5. Tạo Service trong src/services/
6. Tạo Handlers trong src/handlers/
7. Đăng ký routes trong src/main.rs
8. Tuân theo coding standards trong #[[file:.kiro/steering/rust-coding-standards.md]]

Endpoints cần tạo:
- POST /[resource] - Create
- GET /[resource] - List with pagination
- GET /[resource]/:id - Get by ID
- PUT /[resource]/:id - Update
- DELETE /[resource]/:id - Delete

Authentication: [public/jwt_required/admin_only/app_owner_only]
```

### Ví dụ cụ thể:
```
Tạo CRUD endpoints cho entity Notification với các fields:
- id: UUID (auto-generated)
- user_id: UUID (required, foreign key to users)
- title: String (required, max 200 chars)
- message: String (required)
- is_read: bool (default false)
- notification_type: enum (info, warning, error)
- created_at: DateTime (auto)

Yêu cầu:
1. Tạo migration SQL trong migrations/
2. Tạo Model trong src/models/
3. Tạo DTOs trong src/dto/
4. Tạo Repository trong src/repositories/
5. Tạo Service trong src/services/
6. Tạo Handlers trong src/handlers/
7. Đăng ký routes trong src/main.rs
8. Tuân theo coding standards trong #[[file:.kiro/steering/rust-coding-standards.md]]

Endpoints:
- POST /notifications - Create (jwt_required)
- GET /notifications - List user's notifications (jwt_required)
- GET /notifications/:id - Get by ID (jwt_required, owner only)
- PUT /notifications/:id/read - Mark as read (jwt_required)
- DELETE /notifications/:id - Delete (jwt_required, owner only)

Authentication: jwt_required, user chỉ thấy notifications của mình
```

---

## 2. Tạo Endpoint Đơn Lẻ

### Prompt Template:
```
Tạo endpoint [METHOD] [PATH] với chức năng: [MÔ TẢ]

Request body:
- [field1]: [type] (required/optional)
...

Response:
- [field1]: [type]
...

Yêu cầu:
1. Tạo DTO request/response trong src/dto/[module].rs
2. Thêm method vào Service trong src/services/[module].rs
3. Tạo Handler trong src/handlers/[module].rs
4. Đăng ký route trong src/main.rs
5. Authentication: [public/jwt_required/admin_only]
6. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]]
```

### Ví dụ:
```
Tạo endpoint POST /users/me/avatar với chức năng: Upload avatar cho user

Request body:
- avatar_url: String (required, valid URL)

Response:
- id: UUID
- email: String
- avatar_url: String
- updated_at: DateTime

Yêu cầu:
1. Tạo DTO trong src/dto/auth.rs
2. Thêm method update_avatar vào UserProfileService
3. Tạo Handler trong src/handlers/user_profile.rs
4. Đăng ký route trong protected_user_routes
5. Authentication: jwt_required
6. Validate URL format
7. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]]
```

---

## 3. Thêm Field Mới vào Entity Có Sẵn

### Prompt Template:
```
Thêm field mới vào entity [ENTITY_NAME]:
- [field_name]: [type] (required/optional, default: [value])

Yêu cầu:
1. Tạo migration ALTER TABLE trong migrations/
2. Update Model trong src/models/[entity].rs
3. Update tất cả SELECT queries trong Repository
4. Update DTOs nếu cần expose field mới
5. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 15.2
```

### Ví dụ:
```
Thêm field mới vào entity User:
- last_login_at: DateTime (optional, nullable)
- login_count: i32 (required, default: 0)

Yêu cầu:
1. Tạo migration ALTER TABLE trong migrations/
2. Update User model trong src/models/user.rs
3. Update tất cả SELECT queries trong UserRepository
4. Update UserProfileResponse để include last_login_at
5. Update AuthService.login() để set last_login_at và increment login_count
6. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 15.2
```

---

## 4. Tạo Service Mới

### Prompt Template:
```
Tạo service mới: [SERVICE_NAME]Service với các chức năng:
1. [method1]: [mô tả]
2. [method2]: [mô tả]
...

Dependencies:
- [Repository1]
- [Repository2]
- [OtherService]

Yêu cầu:
1. Tạo file src/services/[name].rs
2. Export trong src/services/mod.rs
3. Inject dependencies qua constructor
4. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 6
```

### Ví dụ:
```
Tạo service mới: NotificationService với các chức năng:
1. send_email_notification: Gửi notification qua email
2. send_push_notification: Gửi push notification
3. mark_as_read: Đánh dấu đã đọc
4. get_unread_count: Đếm số notification chưa đọc

Dependencies:
- NotificationRepository
- UserRepository
- EmailService

Yêu cầu:
1. Tạo file src/services/notification.rs
2. Export trong src/services/mod.rs
3. Inject dependencies qua constructor
4. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 6
```

---

## 5. Tạo Middleware Mới

### Prompt Template:
```
Tạo middleware: [MIDDLEWARE_NAME] với chức năng: [MÔ TẢ]

Logic:
1. [step1]
2. [step2]
...

Reject với error: [ERROR_TYPE] khi [CONDITION]

Yêu cầu:
1. Tạo file src/middleware/[name].rs
2. Export trong src/middleware/mod.rs
3. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 8
```

### Ví dụ:
```
Tạo middleware: rate_limit_middleware với chức năng: Giới hạn request rate

Logic:
1. Extract IP từ X-Forwarded-For hoặc X-Real-IP header
2. Check rate limit trong Redis/Database
3. Nếu vượt limit, reject request
4. Nếu OK, increment counter và cho pass

Reject với error: AuthError::RateLimitExceeded khi vượt quá 100 requests/minute

Yêu cầu:
1. Tạo file src/middleware/rate_limit.rs
2. Export trong src/middleware/mod.rs
3. Configurable: limit và window duration
4. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 8
```

---

## 6. Tạo Error Type Mới

### Prompt Template:
```
Thêm error types mới vào [ERROR_ENUM]:
1. [ErrorName1]: message "[message]", status [HTTP_STATUS]
2. [ErrorName2]: message "[message]", status [HTTP_STATUS]

Yêu cầu:
1. Thêm variants vào enum trong src/error.rs
2. Update IntoResponse implementation
3. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 2 và 15.6
```

### Ví dụ:
```
Thêm error types mới vào AuthError:
1. TooManyDevices: message "Maximum devices limit reached", status 403
2. DeviceNotTrusted: message "Device not recognized, please verify", status 403
3. SessionExpired: message "Session has expired", status 401

Yêu cầu:
1. Thêm variants vào AuthError enum trong src/error.rs
2. Update IntoResponse với error codes: too_many_devices, device_not_trusted, session_expired
3. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 2 và 15.6
```

---

## 7. Tạo Repository Method Mới

### Prompt Template:
```
Thêm method mới vào [REPOSITORY_NAME]:
- Method: [method_name]
- Params: [param1]: [type], [param2]: [type]
- Returns: [return_type]
- SQL: [SELECT/INSERT/UPDATE/DELETE] với điều kiện [CONDITIONS]

Yêu cầu:
1. Thêm method vào src/repositories/[name].rs
2. Handle errors đúng cách
3. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 5
```

### Ví dụ:
```
Thêm method mới vào UserRepository:
- Method: find_by_phone
- Params: phone: &str
- Returns: Result<Option<User>, AuthError>
- SQL: SELECT user với phone = ?

Thêm method:
- Method: find_active_users_by_app
- Params: app_id: Uuid, page: u32, limit: u32
- Returns: Result<Vec<User>, AuthError>
- SQL: SELECT users JOIN user_apps WHERE app_id = ? AND status = 'active' với pagination

Yêu cầu:
1. Thêm methods vào src/repositories/user.rs
2. Handle errors, convert sang AuthError
3. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 5
```

---

## 8. Tạo Integration Test

### Prompt Template:
```
Tạo integration tests cho endpoint [METHOD] [PATH]:

Test cases:
1. [test_name]: [mô tả] - expect [expected_result]
2. [test_name]: [mô tả] - expect [expected_result]
...

Yêu cầu:
1. Tạo test file trong tests/
2. Setup test database
3. Cleanup sau mỗi test
4. Test cả happy path và error cases
```

### Ví dụ:
```
Tạo integration tests cho endpoint POST /auth/register:

Test cases:
1. test_register_success: Valid email và password - expect 201 Created
2. test_register_duplicate_email: Email đã tồn tại - expect 409 Conflict
3. test_register_invalid_email: Email format sai - expect 400 Bad Request
4. test_register_weak_password: Password < 8 chars - expect 400 Bad Request
5. test_register_missing_fields: Thiếu email hoặc password - expect 400 Bad Request

Yêu cầu:
1. Tạo test file trong tests/auth_tests.rs
2. Setup test database với cleanup
3. Use reqwest hoặc axum::test để call API
4. Assert status code và response body
```

---

## 9. Refactor Code Hiện Tại

### Prompt Template:
```
Refactor [FILE/MODULE] với mục tiêu: [MỤC TIÊU]

Vấn đề hiện tại:
1. [vấn đề 1]
2. [vấn đề 2]

Yêu cầu:
1. Giữ nguyên behavior/API
2. Improve [readability/performance/maintainability]
3. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]]
4. Không breaking changes
```

### Ví dụ:
```
Refactor src/services/auth.rs với mục tiêu: Tách service quá lớn

Vấn đề hiện tại:
1. AuthService có > 800 lines
2. Mix nhiều concerns: auth, mfa, session, rate limiting
3. Khó test và maintain

Yêu cầu:
1. Tách thành: AuthService, MfaService, SessionService
2. Giữ nguyên public API của AuthService
3. Các service mới inject vào AuthService
4. Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 15.5
5. Không breaking changes với handlers
```

---

## 10. Update SDK Sau Khi Thêm API

### Prompt Template:
```
Update SDK để support endpoint mới: [METHOD] [PATH]

API Details:
- Request: [request_type]
- Response: [response_type]
- Auth: [auth_type]

Yêu cầu:
1. Thêm types vào sdk/src/types.ts
2. Thêm method vào sdk/src/client.ts
3. Export trong sdk/src/index.ts
4. Rebuild SDK
```

### Ví dụ:
```
Update SDK để support endpoint mới: POST /users/me/avatar

API Details:
- Request: { avatar_url: string }
- Response: UserProfileResponse
- Auth: JWT required

Yêu cầu:
1. Thêm UpdateAvatarRequest type vào sdk/src/types.ts
2. Thêm method updateAvatar(data: UpdateAvatarRequest) vào AuthServerClient
3. Method gọi POST /users/me/avatar với auth header
4. Rebuild SDK với npm run build
```

---

## Quick Reference - Câu Lệnh Ngắn

### Tạo nhanh endpoint:
```
Tạo endpoint [METHOD] [PATH] - [MÔ TẢ NGẮN]. Auth: [jwt/public/admin]. 
Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]]
```

### Thêm field nhanh:
```
Thêm field [FIELD_NAME]: [TYPE] vào [ENTITY]. Migration + update model + repository.
Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]] section 15.2
```

### Fix bug với context:
```
Fix bug: [MÔ TẢ BUG]
File liên quan: [FILES]
Expected: [EXPECTED_BEHAVIOR]
Actual: [ACTUAL_BEHAVIOR]
Tuân theo #[[file:.kiro/steering/rust-coding-standards.md]]
```

### Review code:
```
Review code trong [FILE/PR] theo #[[file:.kiro/steering/rust-coding-standards.md]]
Check: error handling, naming, security, performance
```

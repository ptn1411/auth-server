// ============ Common Types ============

export interface ApiError {
  error: string;
  message: string;
  status_code: number;
}

export interface PaginationParams {
  page?: number;
  limit?: number;
  [key: string]: string | number | boolean | undefined;
}

export interface PaginatedResponse<T> {
  data: T[];
  page: number;
  limit: number;
  total: number;
}

// ============ Auth Types ============

export interface RegisterRequest {
  email: string;
  password: string;
}

export interface RegisterResponse {
  id: string;
  email: string;
  is_active: boolean;
  email_verified: boolean;
  created_at: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface MfaRequiredResponse {
  mfa_required: true;
  mfa_token: string;
  methods: string[];
}

export interface RefreshRequest {
  refresh_token: string;
}

export interface RefreshResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface ForgotPasswordRequest {
  email: string;
}

export interface ResetPasswordRequest {
  token: string;
  new_password: string;
}

export interface VerifyEmailRequest {
  token: string;
}

export interface MfaVerifyRequest {
  mfa_token: string;
  code: string;
}

// ============ User Profile Types ============

export interface UserProfile {
  id: string;
  email: string;
  is_active: boolean;
  email_verified: boolean;
  mfa_enabled: boolean;
  created_at: string;
}

export interface UpdateProfileRequest {
  email?: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

// ============ App Types ============

export interface CreateAppRequest {
  code: string;
  name: string;
}

export interface AppResponse {
  id: string;
  code: string;
  name: string;
  secret?: string;
}

export interface AppAuthRequest {
  app_id: string;
  secret: string;
}

export interface AppAuthResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface RegenerateSecretResponse {
  secret: string;
}

// ============ Role Types ============

export interface CreateRoleRequest {
  name: string;
}

export interface RoleResponse {
  id: string;
  app_id: string;
  name: string;
}

export interface AssignRoleRequest {
  role_id: string;
}

// ============ Permission Types ============

export interface CreatePermissionRequest {
  code: string;
}

export interface PermissionResponse {
  id: string;
  app_id: string;
  code: string;
}

export interface AssignPermissionRequest {
  permission_id: string;
}

// ============ Security Types ============

export interface LogoutRequest {
  all_sessions?: boolean;
}

export interface Session {
  id: string;
  user_agent?: string;
  ip_address?: string;
  created_at: string;
  last_used_at: string;
}

export interface SessionsResponse {
  sessions: Session[];
}

export interface RevokeSessionRequest {
  session_id: string;
}

export interface TotpSetupResponse {
  method_id: string;
  secret: string;
  provisioning_uri: string;
}

export interface TotpVerifyRequest {
  method_id: string;
  code: string;
}

export interface MfaMethod {
  id: string;
  method_type: string;
  is_verified: boolean;
  created_at: string;
}

export interface MfaMethodsResponse {
  methods: MfaMethod[];
  mfa_enabled: boolean;
}

export interface BackupCodesResponse {
  backup_codes: string[];
}

export interface AuditLog {
  id: string;
  action: string;
  ip_address?: string;
  user_agent?: string;
  created_at: string;
}

export interface AuditLogsResponse {
  logs: AuditLog[];
  page: number;
  limit: number;
  total: number;
}

// ============ Admin Types ============

export interface AdminUserDetail {
  id: string;
  email: string;
  is_active: boolean;
  is_system_admin: boolean;
  email_verified: boolean;
  mfa_enabled: boolean;
  failed_login_attempts: number;
  locked_until?: string;
  created_at: string;
  updated_at: string;
}

export interface AdminUpdateUserRequest {
  email?: string;
  is_active?: boolean;
  email_verified?: boolean;
  is_system_admin?: boolean;
}

export interface AdminAppDetail {
  id: string;
  code: string;
  name: string;
  owner_id: string;
  has_secret: boolean;
  created_at: string;
  updated_at: string;
}

export interface AdminUpdateAppRequest {
  name?: string;
  code?: string;
}

export interface UserRolesInfo {
  user_id: string;
  apps: {
    app_id: string;
    app_code: string;
    app_name: string;
    roles: RoleResponse[];
  }[];
}

export interface SearchUsersParams extends PaginationParams {
  email?: string;
  is_active?: boolean;
  is_system_admin?: boolean;
  [key: string]: string | number | boolean | undefined;
}

// ============ App User Management Types ============

export interface AppUser {
  id: string;
  email: string;
  is_banned: boolean;
  joined_at: string;
}

export interface AppUsersResponse {
  users: AppUser[];
  page: number;
  limit: number;
  total: number;
}

// ============ OAuth Types ============

export interface ConnectedApp {
  client_id: string;
  client_name: string;
  scopes: string[];
  authorized_at: string;
}

export interface ConnectedAppsResponse {
  apps: ConnectedApp[];
}

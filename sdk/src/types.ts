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
  is_system_admin: boolean;
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


// ============ Webhook Types ============

export interface CreateWebhookRequest {
  url: string;
  events: string[];
}

export interface UpdateWebhookRequest {
  url?: string;
  events?: string[];
  is_active?: boolean;
}

export interface WebhookResponse {
  id: string;
  app_id: string;
  url: string;
  events: string[];
  is_active: boolean;
  created_at: string;
}

export interface WebhookWithSecretResponse extends WebhookResponse {
  secret: string;
}

// ============ API Key Types ============

export interface CreateApiKeyRequest {
  name: string;
  scopes?: string[];
  expires_at?: string;
}

export interface UpdateApiKeyRequest {
  name?: string;
  scopes?: string[];
  is_active?: boolean;
}

export interface ApiKeyResponse {
  id: string;
  app_id: string;
  name: string;
  key_prefix: string;
  scopes: string[];
  expires_at?: string;
  last_used_at?: string;
  is_active: boolean;
  created_at: string;
}

export interface ApiKeyWithSecretResponse extends ApiKeyResponse {
  key: string;
}

// ============ IP Rule Types ============

export interface CreateIpRuleRequest {
  ip_address: string;
  ip_range?: string;
  rule_type: 'whitelist' | 'blacklist';
  reason?: string;
  expires_at?: string;
}

export interface IpRuleResponse {
  id: string;
  app_id?: string;
  ip_address: string;
  ip_range?: string;
  rule_type: string;
  reason?: string;
  expires_at?: string;
  created_by?: string;
  created_at: string;
}

export interface IpCheckResponse {
  ip: string;
  allowed: boolean;
  rule_type?: string;
}

// ============ WebAuthn/Passkey Types ============

export interface StartRegistrationRequest {
  device_name?: string;
}

export interface RegistrationOptionsResponse {
  challenge: string;
  rp: {
    id: string;
    name: string;
  };
  user: {
    id: string;
    name: string;
    display_name: string;
  };
  pub_key_cred_params: Array<{
    type: string;
    alg: number;
  }>;
  timeout: number;
  attestation: string;
  authenticator_selection: {
    authenticator_attachment?: string;
    resident_key: string;
    user_verification: string;
  };
}

export interface FinishRegistrationRequest {
  id: string;
  raw_id: string;
  response: {
    client_data_json: string;
    attestation_object: string;
  };
  type: string;
  device_name?: string;
}

export interface StartAuthenticationRequest {
  email?: string;
}

export interface AuthenticationOptionsResponse {
  challenge: string;
  timeout: number;
  rp_id: string;
  allow_credentials: Array<{
    id: string;
    type: string;
    transports?: string[];
  }>;
  user_verification: string;
}

export interface FinishAuthenticationRequest {
  id: string;
  raw_id: string;
  response: {
    client_data_json: string;
    authenticator_data: string;
    signature: string;
    user_handle?: string;
  };
  type: string;
}

export interface PasskeyResponse {
  id: string;
  device_name?: string;
  transports?: string[];
  last_used_at?: string;
  created_at: string;
}

export interface PasskeyAuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface RenameCredentialRequest {
  name: string;
}

// ============ OAuth Scope Types ============

export interface OAuthScope {
  id: string;
  code: string;
  description: string;
  is_active: boolean;
  created_at: string;
}

export interface ListScopesResponse {
  scopes: OAuthScope[];
  total: number;
  page: number;
  limit: number;
}

export interface CreateScopeRequest {
  code: string;
  description: string;
}

export interface UpdateScopeRequest {
  description: string;
}

// ============ OAuth Client Types ============

export interface OAuthClientInfo {
  id: string;
  client_id: string;
  name: string;
  redirect_uris: string[];
  is_internal: boolean;
  is_active: boolean;
  created_at: string;
}

export interface OAuthClientWithSecret extends OAuthClientInfo {
  client_secret: string;
}

export interface CreateOAuthClientRequest {
  name: string;
  redirect_uris: string[];
  is_internal?: boolean;
}

export interface UpdateOAuthClientRequest {
  name?: string;
  redirect_uris?: string[];
  is_active?: boolean;
}

export interface ListOAuthClientsResponse {
  clients: OAuthClientInfo[];
}

export interface PublicScopeInfo {
  code: string;
  description: string;
}

export interface ListPublicScopesResponse {
  scopes: PublicScopeInfo[];
}

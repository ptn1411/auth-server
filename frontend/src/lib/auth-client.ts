import { AuthServerClient, AuthServerError } from 'auth-server-sdk';

export type {
  // Common types
  ApiError,
  PaginationParams,
  PaginatedResponse,
  // Auth types
  RegisterRequest,
  RegisterResponse,
  LoginRequest,
  LoginResponse,
  MfaRequiredResponse,
  RefreshRequest,
  RefreshResponse,
  ForgotPasswordRequest,
  ResetPasswordRequest,
  VerifyEmailRequest,
  MfaVerifyRequest,
  // User Profile types
  UserProfile,
  UpdateProfileRequest,
  ChangePasswordRequest,
  // Security types
  LogoutRequest,
  Session,
  SessionsResponse,
  RevokeSessionRequest,
  TotpSetupResponse,
  TotpVerifyRequest,
  MfaMethod,
  MfaMethodsResponse,
  BackupCodesResponse,
  AuditLog,
  AuditLogsResponse,
  // WebAuthn/Passkey types
  StartRegistrationRequest,
  RegistrationOptionsResponse,
  FinishRegistrationRequest,
  StartAuthenticationRequest,
  AuthenticationOptionsResponse,
  FinishAuthenticationRequest,
  PasskeyResponse,
  PasskeyAuthResponse,
  RenameCredentialRequest,
  // App types
  CreateAppRequest,
  AppResponse,
  AppAuthRequest,
  AppAuthResponse,
  RegenerateSecretResponse,
  // Role types
  CreateRoleRequest,
  RoleResponse,
  AssignRoleRequest,
  // Permission types
  CreatePermissionRequest,
  PermissionResponse,
  AssignPermissionRequest,
  // App user types
  AppUser,
  AppUsersResponse,
  // Connected apps (OAuth)
  ConnectedApp,
  ConnectedAppsResponse,
  // Webhook types
  CreateWebhookRequest,
  UpdateWebhookRequest,
  WebhookResponse,
  WebhookWithSecretResponse,
  // API Key types
  CreateApiKeyRequest,
  UpdateApiKeyRequest,
  ApiKeyResponse,
  ApiKeyWithSecretResponse,
  // IP Rule types
  CreateIpRuleRequest,
  IpRuleResponse,
  IpCheckResponse,
  // Admin types
  AdminUserDetail,
  AdminUpdateUserRequest,
  AdminAppDetail,
  AdminUpdateAppRequest,
  UserRolesInfo,
  SearchUsersParams,
} from 'auth-server-sdk';

export { AuthServerClient, AuthServerError };

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const authClient = new AuthServerClient({
  baseUrl: API_URL,
  timeout: 30000,
});

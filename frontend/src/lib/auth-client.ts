import { AuthServerClient, AuthServerError } from 'auth-server-sdk';

export type {
  ApiError,
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
  UserProfile,
  UpdateProfileRequest,
  ChangePasswordRequest,
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
  PaginationParams,
  StartRegistrationRequest,
  RegistrationOptionsResponse,
  FinishRegistrationRequest,
  StartAuthenticationRequest,
  AuthenticationOptionsResponse,
  FinishAuthenticationRequest,
  PasskeyResponse,
  PasskeyAuthResponse,
  RenameCredentialRequest,
} from 'auth-server-sdk';

export { AuthServerClient, AuthServerError };

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const authClient = new AuthServerClient({
  baseUrl: API_URL,
  timeout: 30000,
});

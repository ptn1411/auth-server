import {
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
  CreateAppRequest,
  AppResponse,
  AppAuthRequest,
  AppAuthResponse,
  RegenerateSecretResponse,
  CreateRoleRequest,
  RoleResponse,
  AssignRoleRequest,
  CreatePermissionRequest,
  PermissionResponse,
  AssignPermissionRequest,
  LogoutRequest,
  SessionsResponse,
  RevokeSessionRequest,
  TotpSetupResponse,
  TotpVerifyRequest,
  MfaMethodsResponse,
  BackupCodesResponse,
  AuditLogsResponse,
  PaginationParams,
  PaginatedResponse,
  AdminUserDetail,
  AdminUpdateUserRequest,
  AdminAppDetail,
  AdminUpdateAppRequest,
  UserRolesInfo,
  SearchUsersParams,
  AppUsersResponse,
  ConnectedAppsResponse,
} from './types';

export interface AuthServerConfig {
  baseUrl: string;
  timeout?: number;
}

export class AuthServerError extends Error {
  constructor(
    public error: string,
    public statusCode: number,
    message: string
  ) {
    super(message);
    this.name = 'AuthServerError';
  }
}

export class AuthServerClient {
  private baseUrl: string;
  private timeout: number;
  private accessToken?: string;
  private refreshToken?: string;

  constructor(config: AuthServerConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, '');
    this.timeout = config.timeout || 30000;
  }

  // ============ Token Management ============

  setTokens(accessToken: string, refreshToken?: string): void {
    this.accessToken = accessToken;
    this.refreshToken = refreshToken;
  }

  getAccessToken(): string | undefined {
    return this.accessToken;
  }

  clearTokens(): void {
    this.accessToken = undefined;
    this.refreshToken = undefined;
  }

  // ============ HTTP Methods ============

  private async request<T>(
    method: string,
    path: string,
    options: {
      body?: unknown;
      query?: Record<string, string | number | boolean | undefined>;
      auth?: boolean;
      appAuth?: boolean;
    } = {}
  ): Promise<T> {
    const url = new URL(`${this.baseUrl}${path}`);
    
    if (options.query) {
      Object.entries(options.query).forEach(([key, value]) => {
        if (value !== undefined) {
          url.searchParams.append(key, String(value));
        }
      });
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (options.auth !== false && this.accessToken) {
      headers['Authorization'] = `Bearer ${this.accessToken}`;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url.toString(), {
        method,
        headers,
        body: options.body ? JSON.stringify(options.body) : undefined,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error = await response.json() as ApiError;
        throw new AuthServerError(error.error, response.status, error.message);
      }

      if (response.status === 204) {
        return undefined as T;
      }

      return await response.json() as T;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof AuthServerError) throw error;
      throw new AuthServerError('network_error', 0, String(error));
    }
  }

  private get<T>(path: string, query?: Record<string, string | number | boolean | undefined>): Promise<T> {
    return this.request<T>('GET', path, { query });
  }

  private post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('POST', path, { body });
  }

  private put<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PUT', path, { body });
  }

  private delete<T>(path: string): Promise<T> {
    return this.request<T>('DELETE', path);
  }

  // ============ Health Check ============

  async health(): Promise<{ status: string; version: string }> {
    return this.request('GET', '/health', { auth: false });
  }

  async ready(): Promise<{ status: string; version: string }> {
    return this.request('GET', '/ready', { auth: false });
  }

  // ============ Auth API ============

  async register(data: RegisterRequest): Promise<RegisterResponse> {
    return this.request('POST', '/auth/register', { body: data, auth: false });
  }

  async login(data: LoginRequest): Promise<LoginResponse | MfaRequiredResponse> {
    const response = await this.request<LoginResponse | MfaRequiredResponse>(
      'POST', '/auth/login', { body: data, auth: false }
    );
    
    if ('access_token' in response) {
      this.setTokens(response.access_token, response.refresh_token);
    }
    
    return response;
  }

  async completeMfaLogin(data: MfaVerifyRequest): Promise<LoginResponse> {
    const response = await this.request<LoginResponse>(
      'POST', '/auth/mfa/verify', { body: data, auth: false }
    );
    this.setTokens(response.access_token, response.refresh_token);
    return response;
  }

  async refresh(data?: RefreshRequest): Promise<RefreshResponse> {
    const token = data?.refresh_token || this.refreshToken;
    if (!token) throw new AuthServerError('no_refresh_token', 400, 'No refresh token available');
    
    const response = await this.request<RefreshResponse>(
      'POST', '/auth/refresh', { body: { refresh_token: token }, auth: false }
    );
    this.accessToken = response.access_token;
    return response;
  }

  async forgotPassword(data: ForgotPasswordRequest): Promise<{ message: string }> {
    return this.request('POST', '/auth/forgot-password', { body: data, auth: false });
  }

  async resetPassword(data: ResetPasswordRequest): Promise<{ message: string }> {
    return this.request('POST', '/auth/reset-password', { body: data, auth: false });
  }

  async verifyEmail(data: VerifyEmailRequest): Promise<{ message: string }> {
    return this.request('POST', '/auth/verify-email', { body: data, auth: false });
  }

  async resendVerification(): Promise<{ message: string }> {
    return this.post('/auth/resend-verification');
  }

  async logout(data?: LogoutRequest): Promise<{ message: string }> {
    const response = await this.post<{ message: string }>('/auth/logout', data);
    if (!data?.all_sessions) {
      this.clearTokens();
    }
    return response;
  }

  // ============ User Profile API ============

  async getProfile(): Promise<UserProfile> {
    return this.get('/users/me');
  }

  async updateProfile(data: UpdateProfileRequest): Promise<UserProfile> {
    return this.put('/users/me', data);
  }

  async changePassword(data: ChangePasswordRequest): Promise<{ message: string }> {
    return this.post('/users/me/change-password', data);
  }

  // ============ Session Management ============

  async getSessions(): Promise<SessionsResponse> {
    return this.get('/auth/sessions');
  }

  async revokeSession(data: RevokeSessionRequest): Promise<{ message: string }> {
    return this.post('/auth/sessions/revoke', data);
  }

  async revokeOtherSessions(): Promise<{ message: string }> {
    return this.delete('/auth/sessions');
  }

  // ============ MFA API ============

  async setupTotp(): Promise<TotpSetupResponse> {
    return this.post('/auth/mfa/totp/setup');
  }

  async verifyTotpSetup(data: TotpVerifyRequest): Promise<BackupCodesResponse> {
    return this.post('/auth/mfa/totp/verify', data);
  }

  async getMfaMethods(): Promise<MfaMethodsResponse> {
    return this.get('/auth/mfa/methods');
  }

  async disableMfa(): Promise<{ message: string }> {
    return this.delete('/auth/mfa');
  }

  async regenerateBackupCodes(): Promise<BackupCodesResponse> {
    return this.post('/auth/mfa/backup-codes/regenerate');
  }

  // ============ Audit Logs ============

  async getAuditLogs(params?: PaginationParams): Promise<AuditLogsResponse> {
    return this.get('/auth/audit-logs', params);
  }

  // ============ App Management API ============

  async createApp(data: CreateAppRequest): Promise<AppResponse> {
    return this.post('/apps', data);
  }

  async authenticateApp(data: AppAuthRequest): Promise<AppAuthResponse> {
    return this.request('POST', '/apps/auth', { body: data, auth: false });
  }

  async regenerateAppSecret(appId: string): Promise<RegenerateSecretResponse> {
    return this.post(`/apps/${appId}/secret/regenerate`);
  }

  // ============ Role Management API ============

  async createRole(appId: string, data: CreateRoleRequest): Promise<RoleResponse> {
    return this.post(`/apps/${appId}/roles`, data);
  }

  async assignRole(appId: string, userId: string, data: AssignRoleRequest): Promise<void> {
    return this.post(`/apps/${appId}/users/${userId}/roles`, data);
  }

  async getUserRolesInApp(appId: string, userId: string): Promise<RoleResponse[]> {
    return this.get(`/apps/${appId}/users/${userId}/roles`);
  }

  async removeRole(appId: string, userId: string, roleId: string): Promise<void> {
    return this.delete(`/apps/${appId}/users/${userId}/roles/${roleId}`);
  }

  // ============ Permission Management API ============

  async createPermission(appId: string, data: CreatePermissionRequest): Promise<PermissionResponse> {
    return this.post(`/apps/${appId}/permissions`, data);
  }

  // ============ App User Management API ============

  async registerToApp(appId: string): Promise<{ message: string }> {
    return this.post(`/apps/${appId}/register`);
  }

  async getAppUsers(appId: string, params?: PaginationParams): Promise<AppUsersResponse> {
    return this.get(`/apps/${appId}/users`, params);
  }

  async banUser(appId: string, userId: string): Promise<{ message: string }> {
    return this.post(`/apps/${appId}/users/${userId}/ban`);
  }

  async unbanUser(appId: string, userId: string): Promise<{ message: string }> {
    return this.post(`/apps/${appId}/users/${userId}/unban`);
  }

  async removeUserFromApp(appId: string, userId: string): Promise<void> {
    return this.delete(`/apps/${appId}/users/${userId}`);
  }

  // ============ OAuth Account Management ============

  async getConnectedApps(): Promise<ConnectedAppsResponse> {
    return this.get('/account/connected-apps');
  }

  async revokeAppConsent(clientId: string): Promise<void> {
    return this.delete(`/account/connected-apps/${clientId}`);
  }

  // ============ Admin API ============

  async adminListUsers(params?: PaginationParams): Promise<PaginatedResponse<AdminUserDetail>> {
    return this.get('/admin/users', params);
  }

  async adminSearchUsers(params?: SearchUsersParams): Promise<PaginatedResponse<AdminUserDetail>> {
    return this.get('/admin/users/search', params as Record<string, string | number | boolean | undefined>);
  }

  async adminGetUser(userId: string): Promise<AdminUserDetail> {
    return this.get(`/admin/users/${userId}`);
  }

  async adminUpdateUser(userId: string, data: AdminUpdateUserRequest): Promise<AdminUserDetail> {
    return this.put(`/admin/users/${userId}`, data);
  }

  async adminDeleteUser(userId: string): Promise<void> {
    return this.delete(`/admin/users/${userId}`);
  }

  async adminDeactivateUser(userId: string): Promise<void> {
    return this.post(`/admin/users/${userId}/deactivate`);
  }

  async adminActivateUser(userId: string): Promise<void> {
    return this.post(`/admin/users/${userId}/activate`);
  }

  async adminUnlockUser(userId: string): Promise<{ message: string }> {
    return this.post(`/admin/users/${userId}/unlock`);
  }

  async adminGetUserRoles(userId: string): Promise<UserRolesInfo> {
    return this.get(`/admin/users/${userId}/roles`);
  }

  async adminListApps(params?: PaginationParams): Promise<PaginatedResponse<AdminAppDetail>> {
    return this.get('/admin/apps', params);
  }

  async adminGetApp(appId: string): Promise<AdminAppDetail> {
    return this.get(`/admin/apps/${appId}`);
  }

  async adminUpdateApp(appId: string, data: AdminUpdateAppRequest): Promise<AdminAppDetail> {
    return this.put(`/admin/apps/${appId}`, data);
  }

  async adminDeleteApp(appId: string): Promise<void> {
    return this.delete(`/admin/apps/${appId}`);
  }

  async adminGetAuditLogs(params?: PaginationParams): Promise<AuditLogsResponse> {
    return this.get('/admin/audit-logs', params);
  }

  async adminExportUsers(): Promise<AdminUserDetail[]> {
    return this.get('/admin/users/export');
  }

  async adminImportUsers(users: Partial<AdminUserDetail>[]): Promise<{ imported: number }> {
    return this.post('/admin/users/import', { users });
  }

  async adminBulkAssignRole(userIds: string[], roleId: string): Promise<{ assigned: number }> {
    return this.post('/admin/users/bulk-assign-role', { user_ids: userIds, role_id: roleId });
  }

  // ============ Webhook API ============

  async createWebhook(appId: string, data: import('./types').CreateWebhookRequest): Promise<import('./types').WebhookWithSecretResponse> {
    return this.post(`/apps/${appId}/webhooks`, data);
  }

  async listWebhooks(appId: string): Promise<import('./types').WebhookResponse[]> {
    return this.get(`/apps/${appId}/webhooks`);
  }

  async getWebhook(appId: string, webhookId: string): Promise<import('./types').WebhookResponse> {
    return this.get(`/apps/${appId}/webhooks/${webhookId}`);
  }

  async updateWebhook(appId: string, webhookId: string, data: import('./types').UpdateWebhookRequest): Promise<import('./types').WebhookResponse> {
    return this.put(`/apps/${appId}/webhooks/${webhookId}`, data);
  }

  async deleteWebhook(appId: string, webhookId: string): Promise<void> {
    return this.delete(`/apps/${appId}/webhooks/${webhookId}`);
  }

  // ============ API Key API ============

  async createApiKey(appId: string, data: import('./types').CreateApiKeyRequest): Promise<import('./types').ApiKeyWithSecretResponse> {
    return this.post(`/apps/${appId}/api-keys`, data);
  }

  async listApiKeys(appId: string): Promise<import('./types').ApiKeyResponse[]> {
    return this.get(`/apps/${appId}/api-keys`);
  }

  async getApiKey(appId: string, keyId: string): Promise<import('./types').ApiKeyResponse> {
    return this.get(`/apps/${appId}/api-keys/${keyId}`);
  }

  async updateApiKey(appId: string, keyId: string, data: import('./types').UpdateApiKeyRequest): Promise<import('./types').ApiKeyResponse> {
    return this.put(`/apps/${appId}/api-keys/${keyId}`, data);
  }

  async deleteApiKey(appId: string, keyId: string): Promise<void> {
    return this.delete(`/apps/${appId}/api-keys/${keyId}`);
  }

  async revokeApiKey(appId: string, keyId: string): Promise<void> {
    return this.post(`/apps/${appId}/api-keys/${keyId}/revoke`);
  }

  // ============ IP Rules API ============

  async createAppIpRule(appId: string, data: import('./types').CreateIpRuleRequest): Promise<import('./types').IpRuleResponse> {
    return this.post(`/apps/${appId}/ip-rules`, data);
  }

  async listAppIpRules(appId: string): Promise<import('./types').IpRuleResponse[]> {
    return this.get(`/apps/${appId}/ip-rules`);
  }

  async adminCreateIpRule(data: import('./types').CreateIpRuleRequest): Promise<import('./types').IpRuleResponse> {
    return this.post('/admin/ip-rules', data);
  }

  async adminListIpRules(): Promise<import('./types').IpRuleResponse[]> {
    return this.get('/admin/ip-rules');
  }

  async adminCheckIp(ip: string, appId?: string): Promise<import('./types').IpCheckResponse> {
    return this.get('/admin/ip-rules/check', { ip, app_id: appId });
  }

  async adminDeleteIpRule(ruleId: string): Promise<void> {
    return this.delete(`/admin/ip-rules/${ruleId}`);
  }

  // ============ WebAuthn/Passkey API ============

  async startPasskeyRegistration(data?: import('./types').StartRegistrationRequest): Promise<import('./types').RegistrationOptionsResponse> {
    return this.post('/auth/webauthn/register/start', data || {});
  }

  async finishPasskeyRegistration(data: import('./types').FinishRegistrationRequest): Promise<import('./types').PasskeyResponse> {
    return this.post('/auth/webauthn/register/finish', data);
  }

  async startPasskeyAuthentication(data?: import('./types').StartAuthenticationRequest): Promise<import('./types').AuthenticationOptionsResponse> {
    return this.request('POST', '/auth/webauthn/authenticate/start', { body: data || {}, auth: false });
  }

  async finishPasskeyAuthentication(data: import('./types').FinishAuthenticationRequest): Promise<import('./types').PasskeyAuthResponse> {
    const response = await this.request<import('./types').PasskeyAuthResponse>(
      'POST', '/auth/webauthn/authenticate/finish', { body: data, auth: false }
    );
    this.setTokens(response.access_token, response.refresh_token);
    return response;
  }

  async listPasskeys(): Promise<import('./types').PasskeyResponse[]> {
    return this.get('/auth/webauthn/credentials');
  }

  async renamePasskey(credentialId: string, data: import('./types').RenameCredentialRequest): Promise<void> {
    return this.put(`/auth/webauthn/credentials/${credentialId}`, data);
  }

  async deletePasskey(credentialId: string): Promise<void> {
    return this.delete(`/auth/webauthn/credentials/${credentialId}`);
  }
}

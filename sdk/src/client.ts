import {
  AuthServerConfig,
  AuthServerError,
  TokenManager,
  AuthApi,
  MfaApi,
  UserApi,
  AppsApi,
  AppSelfApi,
  WebAuthnApi,
  OAuthApi,
  AdminApi,
} from "./api";

// Re-export for backward compatibility
export { AuthServerConfig, AuthServerError };

/**
 * Main client for Auth Server SDK
 * 
 * Provides access to all API modules:
 * - auth: Authentication (login, register, sessions, etc.)
 * - mfa: Multi-factor authentication
 * - user: User profile management
 * - apps: App management (CRUD, roles, permissions, webhooks, etc.)
 * - appSelf: App self-management API (for app tokens)
 * - webauthn: WebAuthn/Passkey authentication
 * - oauth: OAuth clients and tokens
 * - admin: Admin operations (users, apps, scopes, IP rules)
 */
export class AuthServerClient implements TokenManager {
  private accessToken?: string;
  private refreshToken?: string;
  private apiKey?: string;

  // API modules
  public readonly auth: AuthApi;
  public readonly mfa: MfaApi;
  public readonly user: UserApi;
  public readonly apps: AppsApi;
  public readonly appSelf: AppSelfApi;
  public readonly webauthn: WebAuthnApi;
  public readonly oauth: OAuthApi;
  public readonly admin: AdminApi;

  private baseUrl: string;
  private timeout: number;

  constructor(config: AuthServerConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.timeout = config.timeout || 30000;

    // Initialize API modules
    this.auth = new AuthApi(config, this);
    this.mfa = new MfaApi(config, this);
    this.user = new UserApi(config, this);
    this.apps = new AppsApi(config, this);
    this.appSelf = new AppSelfApi(config, this);
    this.webauthn = new WebAuthnApi(config, this);
    this.oauth = new OAuthApi(config, this);
    this.admin = new AdminApi(config, this);
  }

  // ============ Token Management (implements TokenManager) ============

  setTokens(accessToken: string, refreshToken?: string): void {
    this.accessToken = accessToken;
    if (refreshToken) {
      this.refreshToken = refreshToken;
    }
  }

  getAccessToken(): string | undefined {
    return this.accessToken;
  }

  getRefreshToken(): string | undefined {
    return this.refreshToken;
  }

  clearTokens(): void {
    this.accessToken = undefined;
    this.refreshToken = undefined;
  }

  // ============ API Key Authentication ============

  /**
   * Set API key for X-API-Key header authentication
   * Used for app-level API authentication
   */
  setAuthApiKey(apiKey: string): void {
    this.apiKey = apiKey;
  }

  /**
   * Get current API key for X-API-Key header authentication
   */
  getAuthApiKey(): string | undefined {
    return this.apiKey;
  }

  /**
   * Clear API key authentication
   */
  clearAuthApiKey(): void {
    this.apiKey = undefined;
  }

  // ============ Health Check ============

  async health(): Promise<{ status: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
  }

  async ready(): Promise<{ status: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/ready`);
    return response.json();
  }

  // ============ Backward Compatibility Methods ============
  // These methods delegate to the appropriate API module for backward compatibility

  // Auth
  register: AuthApi["register"] = (...args) => this.auth.register(...args);
  login: AuthApi["login"] = (...args) => this.auth.login(...args);
  completeMfaLogin: AuthApi["completeMfaLogin"] = (...args) => this.auth.completeMfaLogin(...args);
  refresh: AuthApi["refresh"] = (...args) => this.auth.refresh(...args);
  forgotPassword: AuthApi["forgotPassword"] = (...args) => this.auth.forgotPassword(...args);
  resetPassword: AuthApi["resetPassword"] = (...args) => this.auth.resetPassword(...args);
  verifyEmail: AuthApi["verifyEmail"] = (...args) => this.auth.verifyEmail(...args);
  resendVerification: AuthApi["resendVerification"] = (...args) => this.auth.resendVerification(...args);
  logout: AuthApi["logout"] = (...args) => this.auth.logout(...args);
  getSessions: AuthApi["getSessions"] = (...args) => this.auth.getSessions(...args);
  revokeSession: AuthApi["revokeSession"] = (...args) => this.auth.revokeSession(...args);
  revokeOtherSessions: AuthApi["revokeOtherSessions"] = (...args) => this.auth.revokeOtherSessions(...args);
  getAuditLogs: AuthApi["getAuditLogs"] = (...args) => this.auth.getAuditLogs(...args);

  // MFA
  setupTotp: MfaApi["setupTotp"] = (...args) => this.mfa.setupTotp(...args);
  verifyTotpSetup: MfaApi["verifyTotpSetup"] = (...args) => this.mfa.verifyTotpSetup(...args);
  getMfaMethods: MfaApi["getMethods"] = (...args) => this.mfa.getMethods(...args);
  disableMfa: MfaApi["disable"] = (...args) => this.mfa.disable(...args);
  regenerateBackupCodes: MfaApi["regenerateBackupCodes"] = (...args) => this.mfa.regenerateBackupCodes(...args);

  // User
  getProfile: UserApi["getProfile"] = (...args) => this.user.getProfile(...args);
  updateProfile: UserApi["updateProfile"] = (...args) => this.user.updateProfile(...args);
  changePassword: UserApi["changePassword"] = (...args) => this.user.changePassword(...args);
  getConnectedApps: UserApi["getConnectedApps"] = (...args) => this.user.getConnectedApps(...args);
  revokeAppConsent: UserApi["revokeAppConsent"] = (...args) => this.user.revokeAppConsent(...args);

  // Apps
  listMyApps: AppsApi["list"] = (...args) => this.apps.list(...args);
  createApp: AppsApi["create"] = (...args) => this.apps.create(...args);
  getApp: AppsApi["getById"] = (...args) => this.apps.getById(...args);
  authenticateApp: AppsApi["authenticate"] = (...args) => this.apps.authenticate(...args);
  regenerateAppSecret: AppsApi["regenerateSecret"] = (...args) => this.apps.regenerateSecret(...args);
  registerToApp: AppsApi["registerUser"] = (...args) => this.apps.registerUser(...args);
  getAppUsers: AppsApi["getUsers"] = (...args) => this.apps.getUsers(...args);
  banUser: AppsApi["banUser"] = (...args) => this.apps.banUser(...args);
  unbanUser: AppsApi["unbanUser"] = (...args) => this.apps.unbanUser(...args);
  removeUserFromApp: AppsApi["removeUser"] = (...args) => this.apps.removeUser(...args);
  createRole: AppsApi["createRole"] = (...args) => this.apps.createRole(...args);
  listRoles: AppsApi["listRoles"] = (...args) => this.apps.listRoles(...args);
  assignRole: AppsApi["assignRole"] = (...args) => this.apps.assignRole(...args);
  getUserRolesInApp: AppsApi["getUserRoles"] = (...args) => this.apps.getUserRoles(...args);
  removeRole: AppsApi["removeRole"] = (...args) => this.apps.removeRole(...args);
  createPermission: AppsApi["createPermission"] = (...args) => this.apps.createPermission(...args);
  assignPermissionToRole: AppsApi["assignPermissionToRole"] = (...args) => this.apps.assignPermissionToRole(...args);
  removePermissionFromRole: AppsApi["removePermissionFromRole"] = (...args) => this.apps.removePermissionFromRole(...args);
  getRolePermissions: AppsApi["getRolePermissions"] = (...args) => this.apps.getRolePermissions(...args);
  createWebhook: AppsApi["createWebhook"] = (...args) => this.apps.createWebhook(...args);
  listWebhooks: AppsApi["listWebhooks"] = (...args) => this.apps.listWebhooks(...args);
  getWebhook: AppsApi["getWebhook"] = (...args) => this.apps.getWebhook(...args);
  updateWebhook: AppsApi["updateWebhook"] = (...args) => this.apps.updateWebhook(...args);
  deleteWebhook: AppsApi["deleteWebhook"] = (...args) => this.apps.deleteWebhook(...args);
  createApiKey: AppsApi["createApiKey"] = (...args) => this.apps.createApiKey(...args);
  listApiKeys: AppsApi["listApiKeys"] = (...args) => this.apps.listApiKeys(...args);
  getApiKey: AppsApi["getApiKey"] = (...args) => this.apps.getApiKey(...args);
  updateApiKey: AppsApi["updateApiKey"] = (...args) => this.apps.updateApiKey(...args);
  deleteApiKey: AppsApi["deleteApiKey"] = (...args) => this.apps.deleteApiKey(...args);
  revokeApiKey: AppsApi["revokeApiKey"] = (...args) => this.apps.revokeApiKey(...args);
  createAppIpRule: AppsApi["createIpRule"] = (...args) => this.apps.createIpRule(...args);
  listAppIpRules: AppsApi["listIpRules"] = (...args) => this.apps.listIpRules(...args);

  // WebAuthn
  startPasskeyRegistration: WebAuthnApi["startRegistration"] = (...args) => this.webauthn.startRegistration(...args);
  finishPasskeyRegistration: WebAuthnApi["finishRegistration"] = (...args) => this.webauthn.finishRegistration(...args);
  startPasskeyAuthentication: WebAuthnApi["startAuthentication"] = (...args) => this.webauthn.startAuthentication(...args);
  finishPasskeyAuthentication: WebAuthnApi["finishAuthentication"] = (...args) => this.webauthn.finishAuthentication(...args);
  listPasskeys: WebAuthnApi["list"] = (...args) => this.webauthn.list(...args);
  renamePasskey: WebAuthnApi["rename"] = (...args) => this.webauthn.rename(...args);
  deletePasskey: WebAuthnApi["remove"] = (...args) => this.webauthn.remove(...args);

  // OAuth
  listOAuthClients: OAuthApi["listClients"] = (...args) => this.oauth.listClients(...args);
  createOAuthClient: OAuthApi["createClient"] = (...args) => this.oauth.createClient(...args);
  updateOAuthClient: OAuthApi["updateClient"] = (...args) => this.oauth.updateClient(...args);
  deleteOAuthClient: OAuthApi["deleteClient"] = (...args) => this.oauth.deleteClient(...args);
  regenerateOAuthClientSecret: OAuthApi["regenerateClientSecret"] = (...args) => this.oauth.regenerateClientSecret(...args);
  listPublicScopes: OAuthApi["listPublicScopes"] = (...args) => this.oauth.listPublicScopes(...args);
  revokeOAuthToken: OAuthApi["revokeToken"] = (...args) => this.oauth.revokeToken(...args);
  getOpenIdConfiguration: OAuthApi["getOpenIdConfiguration"] = (...args) => this.oauth.getOpenIdConfiguration(...args);
  getUserInfo: OAuthApi["getUserInfo"] = (...args) => this.oauth.getUserInfo(...args);

  // Admin
  adminListUsers: AdminApi["listUsers"] = (...args) => this.admin.listUsers(...args);
  adminSearchUsers: AdminApi["searchUsers"] = (...args) => this.admin.searchUsers(...args);
  adminGetUser: AdminApi["getUser"] = (...args) => this.admin.getUser(...args);
  adminUpdateUser: AdminApi["updateUser"] = (...args) => this.admin.updateUser(...args);
  adminDeleteUser: AdminApi["deleteUser"] = (...args) => this.admin.deleteUser(...args);
  adminDeactivateUser: AdminApi["deactivateUser"] = (...args) => this.admin.deactivateUser(...args);
  adminActivateUser: AdminApi["activateUser"] = (...args) => this.admin.activateUser(...args);
  adminUnlockUser: AdminApi["unlockUser"] = (...args) => this.admin.unlockUser(...args);
  adminGetUserRoles: AdminApi["getUserRoles"] = (...args) => this.admin.getUserRoles(...args);
  adminListApps: AdminApi["listApps"] = (...args) => this.admin.listApps(...args);
  adminGetApp: AdminApi["getApp"] = (...args) => this.admin.getApp(...args);
  adminUpdateApp: AdminApi["updateApp"] = (...args) => this.admin.updateApp(...args);
  adminDeleteApp: AdminApi["deleteApp"] = (...args) => this.admin.deleteApp(...args);
  adminGetAuditLogs: AdminApi["getAuditLogs"] = (...args) => this.admin.getAuditLogs(...args);
  adminExportUsers: AdminApi["exportUsers"] = (...args) => this.admin.exportUsers(...args);
  adminImportUsers: AdminApi["importUsers"] = (...args) => this.admin.importUsers(...args);
  adminBulkAssignRole: AdminApi["bulkAssignRole"] = (...args) => this.admin.bulkAssignRole(...args);
  adminCreateIpRule: AdminApi["createIpRule"] = (...args) => this.admin.createIpRule(...args);
  adminListIpRules: AdminApi["listIpRules"] = (...args) => this.admin.listIpRules(...args);
  adminCheckIp: AdminApi["checkIp"] = (...args) => this.admin.checkIp(...args);
  adminDeleteIpRule: AdminApi["deleteIpRule"] = (...args) => this.admin.deleteIpRule(...args);
  adminListScopes: AdminApi["listScopes"] = (...args) => this.admin.listScopes(...args);
  adminGetScope: AdminApi["getScope"] = (...args) => this.admin.getScope(...args);
  adminCreateScope: AdminApi["createScope"] = (...args) => this.admin.createScope(...args);
  adminUpdateScope: AdminApi["updateScope"] = (...args) => this.admin.updateScope(...args);
  adminActivateScope: AdminApi["activateScope"] = (...args) => this.admin.activateScope(...args);
  adminDeactivateScope: AdminApi["deactivateScope"] = (...args) => this.admin.deactivateScope(...args);
  adminDeleteScope: AdminApi["deleteScope"] = (...args) => this.admin.deleteScope(...args);

  // App Self API
  appCreateRole: AppSelfApi["createRole"] = (...args) => this.appSelf.createRole(...args);
  appListRoles: AppSelfApi["listRoles"] = (...args) => this.appSelf.listRoles(...args);
  appCreatePermission: AppSelfApi["createPermission"] = (...args) => this.appSelf.createPermission(...args);
  appListPermissions: AppSelfApi["listPermissions"] = (...args) => this.appSelf.listPermissions(...args);
  appAssignPermissionToRole: AppSelfApi["assignPermissionToRole"] = (...args) => this.appSelf.assignPermissionToRole(...args);
}

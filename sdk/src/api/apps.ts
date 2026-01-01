import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  AppResponse,
  CreateAppRequest,
  AppAuthRequest,
  AppAuthResponse,
  RegenerateSecretResponse,
  PaginatedResponse,
  PaginationParams,
  AppUsersResponse,
  RoleResponse,
  CreateRoleRequest,
  AssignRoleRequest,
  PermissionResponse,
  CreatePermissionRequest,
  AssignPermissionRequest,
  CreateWebhookRequest,
  WebhookWithSecretResponse,
  WebhookResponse,
  UpdateWebhookRequest,
  CreateApiKeyRequest,
  ApiKeyWithSecretResponse,
  ApiKeyResponse,
  UpdateApiKeyRequest,
  CreateIpRuleRequest,
  IpRuleResponse,
} from "../types";

export class AppsApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  // ============ App CRUD ============

  async list(params?: PaginationParams): Promise<PaginatedResponse<AppResponse>> {
    return this.get("/apps", params);
  }

  async create(data: CreateAppRequest): Promise<AppResponse> {
    return this.post("/apps", data);
  }

  async getById(appId: string): Promise<AppResponse> {
    return this.get(`/apps/${appId}`);
  }

  async authenticate(data: AppAuthRequest): Promise<AppAuthResponse> {
    const response = await this.request<AppAuthResponse>("POST", "/apps/auth", {
      body: data,
      auth: false,
    });
    this.tokenManager.setTokens(response.access_token);
    return response;
  }

  async regenerateSecret(appId: string): Promise<RegenerateSecretResponse> {
    return this.post(`/apps/${appId}/secret/regenerate`);
  }

  // ============ App Users ============

  async registerUser(appId: string): Promise<{ message: string }> {
    return this.post(`/apps/${appId}/register`);
  }

  async getUsers(appId: string, params?: PaginationParams): Promise<AppUsersResponse> {
    return this.get(`/apps/${appId}/users`, params);
  }

  async banUser(appId: string, userId: string): Promise<{ message: string }> {
    return this.post(`/apps/${appId}/users/${userId}/ban`);
  }

  async unbanUser(appId: string, userId: string): Promise<{ message: string }> {
    return this.post(`/apps/${appId}/users/${userId}/unban`);
  }

  async removeUser(appId: string, userId: string): Promise<void> {
    return this.delete(`/apps/${appId}/users/${userId}`);
  }

  // ============ Roles ============

  async createRole(appId: string, data: CreateRoleRequest): Promise<RoleResponse> {
    return this.post(`/apps/${appId}/roles`, data);
  }

  async listRoles(appId: string): Promise<RoleResponse[]> {
    return this.get(`/app-api/apps/${appId}/roles`);
  }

  async assignRole(appId: string, userId: string, data: AssignRoleRequest): Promise<void> {
    return this.post(`/apps/${appId}/users/${userId}/roles`, data);
  }

  async getUserRoles(appId: string, userId: string): Promise<RoleResponse[]> {
    return this.get(`/apps/${appId}/users/${userId}/roles`);
  }

  async removeRole(appId: string, userId: string, roleId: string): Promise<void> {
    return this.delete(`/apps/${appId}/users/${userId}/roles/${roleId}`);
  }

  // ============ Permissions ============

  async createPermission(appId: string, data: CreatePermissionRequest): Promise<PermissionResponse> {
    return this.post(`/apps/${appId}/permissions`, data);
  }

  async assignPermissionToRole(appId: string, roleId: string, data: AssignPermissionRequest): Promise<void> {
    return this.post(`/apps/${appId}/roles/${roleId}/permissions`, data);
  }

  async removePermissionFromRole(appId: string, roleId: string, permissionId: string): Promise<void> {
    return this.delete(`/apps/${appId}/roles/${roleId}/permissions/${permissionId}`);
  }

  async getRolePermissions(appId: string, roleId: string): Promise<PermissionResponse[]> {
    return this.get(`/apps/${appId}/roles/${roleId}/permissions`);
  }

  // ============ Webhooks ============

  async createWebhook(appId: string, data: CreateWebhookRequest): Promise<WebhookWithSecretResponse> {
    return this.post(`/apps/${appId}/webhooks`, data);
  }

  async listWebhooks(appId: string): Promise<WebhookResponse[]> {
    return this.get(`/apps/${appId}/webhooks`);
  }

  async getWebhook(appId: string, webhookId: string): Promise<WebhookResponse> {
    return this.get(`/apps/${appId}/webhooks/${webhookId}`);
  }

  async updateWebhook(appId: string, webhookId: string, data: UpdateWebhookRequest): Promise<WebhookResponse> {
    return this.put(`/apps/${appId}/webhooks/${webhookId}`, data);
  }

  async deleteWebhook(appId: string, webhookId: string): Promise<void> {
    return this.delete(`/apps/${appId}/webhooks/${webhookId}`);
  }

  // ============ API Keys ============

  async createApiKey(appId: string, data: CreateApiKeyRequest): Promise<ApiKeyWithSecretResponse> {
    return this.post(`/apps/${appId}/api-keys`, data);
  }

  async listApiKeys(appId: string): Promise<ApiKeyResponse[]> {
    return this.get(`/apps/${appId}/api-keys`);
  }

  async getApiKey(appId: string, keyId: string): Promise<ApiKeyResponse> {
    return this.get(`/apps/${appId}/api-keys/${keyId}`);
  }

  async updateApiKey(appId: string, keyId: string, data: UpdateApiKeyRequest): Promise<ApiKeyResponse> {
    return this.put(`/apps/${appId}/api-keys/${keyId}`, data);
  }

  async deleteApiKey(appId: string, keyId: string): Promise<void> {
    return this.delete(`/apps/${appId}/api-keys/${keyId}`);
  }

  async revokeApiKey(appId: string, keyId: string): Promise<void> {
    return this.post(`/apps/${appId}/api-keys/${keyId}/revoke`);
  }

  // ============ IP Rules ============

  async createIpRule(appId: string, data: CreateIpRuleRequest): Promise<IpRuleResponse> {
    return this.post(`/apps/${appId}/ip-rules`, data);
  }

  async listIpRules(appId: string): Promise<IpRuleResponse[]> {
    return this.get(`/apps/${appId}/ip-rules`);
  }
}

// ============ App Self-Management API (for app tokens) ============

/**
 * App Self-Management API
 * 
 * This API is designed for app-level operations using API key authentication.
 * Set the API key using `client.setAuthApiKey(apiKey)` before calling these methods.
 * 
 * @example
 * ```typescript
 * const client = new AuthServerClient({ baseUrl: 'https://auth.example.com' });
 * client.setAuthApiKey('your-api-key');
 * const roles = await client.appSelf.listRoles(appId);
 * ```
 */
export class AppSelfApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  async createRole(appId: string, data: CreateRoleRequest): Promise<RoleResponse> {
    return this.post(`/app-api/apps/${appId}/roles`, data, "apiKey");
  }

  async listRoles(appId: string): Promise<RoleResponse[]> {
    return this.get(`/app-api/apps/${appId}/roles`, undefined, "apiKey");
  }

  async createPermission(appId: string, data: CreatePermissionRequest): Promise<PermissionResponse> {
    return this.post(`/app-api/apps/${appId}/permissions`, data, "apiKey");
  }

  async listPermissions(appId: string): Promise<PermissionResponse[]> {
    return this.get(`/app-api/apps/${appId}/permissions`, undefined, "apiKey");
  }

  async assignPermissionToRole(appId: string, roleId: string, data: AssignPermissionRequest): Promise<void> {
    return this.post(`/app-api/apps/${appId}/roles/${roleId}/permissions`, data, "apiKey");
  }
}

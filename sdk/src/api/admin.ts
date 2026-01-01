import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  AdminUserDetail,
  AdminUpdateUserRequest,
  AdminAppDetail,
  AdminUpdateAppRequest,
  PaginatedResponse,
  PaginationParams,
  SearchUsersParams,
  UserRolesInfo,
  AuditLogsResponse,
  CreateIpRuleRequest,
  IpRuleResponse,
  IpCheckResponse,
  OAuthScope,
  CreateScopeRequest,
  UpdateScopeRequest,
  ListScopesResponse,
} from "../types";

export class AdminApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  // ============ Users ============

  async listUsers(params?: PaginationParams): Promise<PaginatedResponse<AdminUserDetail>> {
    return this.get("/admin/users", params);
  }

  async searchUsers(params?: SearchUsersParams): Promise<PaginatedResponse<AdminUserDetail>> {
    return this.get(
      "/admin/users/search",
      params as Record<string, string | number | boolean | undefined>
    );
  }

  async getUser(userId: string): Promise<AdminUserDetail> {
    return this.get(`/admin/users/${userId}`);
  }

  async updateUser(userId: string, data: AdminUpdateUserRequest): Promise<AdminUserDetail> {
    return this.put(`/admin/users/${userId}`, data);
  }

  async deleteUser(userId: string): Promise<void> {
    return this.delete(`/admin/users/${userId}`);
  }

  async deactivateUser(userId: string): Promise<void> {
    return this.post(`/admin/users/${userId}/deactivate`);
  }

  async activateUser(userId: string): Promise<void> {
    return this.post(`/admin/users/${userId}/activate`);
  }

  async unlockUser(userId: string): Promise<{ message: string }> {
    return this.post(`/admin/users/${userId}/unlock`);
  }

  async getUserRoles(userId: string): Promise<UserRolesInfo> {
    return this.get(`/admin/users/${userId}/roles`);
  }

  async exportUsers(): Promise<AdminUserDetail[]> {
    return this.get("/admin/users/export");
  }

  async importUsers(users: Partial<AdminUserDetail>[]): Promise<{ imported: number }> {
    return this.post("/admin/users/import", { users });
  }

  async bulkAssignRole(userIds: string[], roleId: string): Promise<{ assigned: number }> {
    return this.post("/admin/users/bulk-assign-role", {
      user_ids: userIds,
      role_id: roleId,
    });
  }

  // ============ Apps ============

  async listApps(params?: PaginationParams): Promise<PaginatedResponse<AdminAppDetail>> {
    return this.get("/admin/apps", params);
  }

  async getApp(appId: string): Promise<AdminAppDetail> {
    return this.get(`/admin/apps/${appId}`);
  }

  async updateApp(appId: string, data: AdminUpdateAppRequest): Promise<AdminAppDetail> {
    return this.put(`/admin/apps/${appId}`, data);
  }

  async deleteApp(appId: string): Promise<void> {
    return this.delete(`/admin/apps/${appId}`);
  }

  // ============ Audit Logs ============

  async getAuditLogs(params?: PaginationParams): Promise<AuditLogsResponse> {
    return this.get("/admin/audit-logs", params);
  }

  // ============ IP Rules ============

  async createIpRule(data: CreateIpRuleRequest): Promise<IpRuleResponse> {
    return this.post("/admin/ip-rules", data);
  }

  async listIpRules(): Promise<IpRuleResponse[]> {
    return this.get("/admin/ip-rules");
  }

  async checkIp(ip: string, appId?: string): Promise<IpCheckResponse> {
    return this.get("/admin/ip-rules/check", { ip, app_id: appId });
  }

  async deleteIpRule(ruleId: string): Promise<void> {
    return this.delete(`/admin/ip-rules/${ruleId}`);
  }

  // ============ OAuth Scopes ============

  async listScopes(params?: PaginationParams): Promise<ListScopesResponse> {
    return this.get("/admin/scopes", params);
  }

  async getScope(scopeId: string): Promise<OAuthScope> {
    return this.get(`/admin/scopes/${scopeId}`);
  }

  async createScope(data: CreateScopeRequest): Promise<OAuthScope> {
    return this.post("/admin/scopes", data);
  }

  async updateScope(scopeId: string, data: UpdateScopeRequest): Promise<OAuthScope> {
    return this.put(`/admin/scopes/${scopeId}`, data);
  }

  async activateScope(scopeId: string): Promise<{ message: string }> {
    return this.post(`/admin/scopes/${scopeId}/activate`);
  }

  async deactivateScope(scopeId: string): Promise<{ message: string }> {
    return this.post(`/admin/scopes/${scopeId}/deactivate`);
  }

  async deleteScope(scopeId: string): Promise<{ message: string }> {
    return this.delete(`/admin/scopes/${scopeId}`);
  }
}

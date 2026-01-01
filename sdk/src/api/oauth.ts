import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  ListOAuthClientsResponse,
  CreateOAuthClientRequest,
  OAuthClientWithSecret,
  UpdateOAuthClientRequest,
  OAuthClientInfo,
  ListPublicScopesResponse,
  RevokeTokenRequest,
  OpenIdConfiguration,
  UserInfo,
} from "../types";

export class OAuthApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  // ============ OAuth Clients ============

  async listClients(): Promise<ListOAuthClientsResponse> {
    return this.get("/oauth/clients");
  }

  async createClient(data: CreateOAuthClientRequest): Promise<OAuthClientWithSecret> {
    return this.post("/oauth/clients", data);
  }

  async updateClient(clientId: string, data: UpdateOAuthClientRequest): Promise<OAuthClientInfo> {
    return this.put(`/oauth/clients/${clientId}`, data);
  }

  async deleteClient(clientId: string): Promise<void> {
    return this.delete(`/oauth/clients/${clientId}`);
  }

  async regenerateClientSecret(clientId: string): Promise<{ client_secret: string }> {
    return this.post(`/oauth/clients/${clientId}/secret`);
  }

  // ============ Public Endpoints ============

  async listPublicScopes(): Promise<ListPublicScopesResponse> {
    return this.request("GET", "/oauth/scopes", { auth: false });
  }

  async revokeToken(data: RevokeTokenRequest): Promise<void> {
    return this.request("POST", "/oauth/revoke", { body: data, auth: false });
  }

  async getOpenIdConfiguration(): Promise<OpenIdConfiguration> {
    return this.request("GET", "/.well-known/openid-configuration", { auth: false });
  }

  async getUserInfo(): Promise<UserInfo> {
    return this.get("/oauth/userinfo");
  }
}

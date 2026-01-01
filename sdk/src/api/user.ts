import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  UserProfile,
  UpdateProfileRequest,
  ChangePasswordRequest,
  ConnectedAppsResponse,
} from "../types";

export class UserApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  async getProfile(): Promise<UserProfile> {
    return this.get("/users/me");
  }

  async updateProfile(data: UpdateProfileRequest): Promise<UserProfile> {
    return this.put("/users/me", data);
  }

  async changePassword(
    data: ChangePasswordRequest
  ): Promise<{ message: string }> {
    return this.post("/users/me/change-password", data);
  }

  async getConnectedApps(): Promise<ConnectedAppsResponse> {
    return this.get("/account/connected-apps");
  }

  async revokeAppConsent(clientId: string): Promise<void> {
    return this.delete(`/account/connected-apps/${clientId}`);
  }
}

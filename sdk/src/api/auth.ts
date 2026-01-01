import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  RegisterRequest,
  RegisterResponse,
  LoginRequest,
  LoginResponse,
  MfaRequiredResponse,
  MfaVerifyRequest,
  RefreshRequest,
  RefreshResponse,
  ForgotPasswordRequest,
  ResetPasswordRequest,
  VerifyEmailRequest,
  LogoutRequest,
  SessionsResponse,
  RevokeSessionRequest,
  AuditLogsResponse,
  PaginationParams,
} from "../types";

export class AuthApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  async register(data: RegisterRequest): Promise<RegisterResponse> {
    return this.request("POST", "/auth/register", { body: data, auth: false });
  }

  async login(
    data: LoginRequest
  ): Promise<LoginResponse | MfaRequiredResponse> {
    const response = await this.request<LoginResponse | MfaRequiredResponse>(
      "POST",
      "/auth/login",
      { body: data, auth: false }
    );

    if ("access_token" in response) {
      this.tokenManager.setTokens(response.access_token, response.refresh_token);
    }

    return response;
  }

  async completeMfaLogin(data: MfaVerifyRequest): Promise<LoginResponse> {
    const response = await this.request<LoginResponse>(
      "POST",
      "/auth/mfa/verify",
      { body: data, auth: false }
    );
    this.tokenManager.setTokens(response.access_token, response.refresh_token);
    return response;
  }

  async refresh(data?: RefreshRequest): Promise<RefreshResponse> {
    const token = data?.refresh_token || this.tokenManager.getRefreshToken();
    if (!token) {
      throw new Error("No refresh token available");
    }

    const response = await this.request<RefreshResponse>(
      "POST",
      "/auth/refresh",
      { body: { refresh_token: token }, auth: false }
    );
    this.tokenManager.setTokens(response.access_token);
    return response;
  }

  async forgotPassword(
    data: ForgotPasswordRequest
  ): Promise<{ message: string }> {
    return this.request("POST", "/auth/forgot-password", {
      body: data,
      auth: false,
    });
  }

  async resetPassword(
    data: ResetPasswordRequest
  ): Promise<{ message: string }> {
    return this.request("POST", "/auth/reset-password", {
      body: data,
      auth: false,
    });
  }

  async verifyEmail(data: VerifyEmailRequest): Promise<{ message: string }> {
    return this.request("POST", "/auth/verify-email", {
      body: data,
      auth: false,
    });
  }

  async resendVerification(): Promise<{ message: string }> {
    return this.post("/auth/resend-verification");
  }

  async logout(data?: LogoutRequest): Promise<{ message: string }> {
    const response = await this.post<{ message: string }>("/auth/logout", data);
    if (!data?.all_sessions) {
      this.tokenManager.clearTokens();
    }
    return response;
  }

  async getSessions(): Promise<SessionsResponse> {
    return this.get("/auth/sessions");
  }

  async revokeSession(
    data: RevokeSessionRequest
  ): Promise<{ message: string }> {
    return this.post("/auth/sessions/revoke", data);
  }

  async revokeOtherSessions(): Promise<{ message: string }> {
    return this.delete("/auth/sessions");
  }

  async getAuditLogs(params?: PaginationParams): Promise<AuditLogsResponse> {
    return this.get("/auth/audit-logs", params);
  }
}

import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  TotpSetupResponse,
  TotpVerifyRequest,
  BackupCodesResponse,
  MfaMethodsResponse,
} from "../types";

export class MfaApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  async setupTotp(): Promise<TotpSetupResponse> {
    return this.post("/auth/mfa/totp/setup");
  }

  async verifyTotpSetup(data: TotpVerifyRequest): Promise<BackupCodesResponse> {
    return this.post("/auth/mfa/totp/verify", data);
  }

  async getMethods(): Promise<MfaMethodsResponse> {
    return this.get("/auth/mfa/methods");
  }

  async disable(): Promise<{ message: string }> {
    return this.delete("/auth/mfa");
  }

  async regenerateBackupCodes(): Promise<BackupCodesResponse> {
    return this.post("/auth/mfa/backup-codes/regenerate");
  }
}

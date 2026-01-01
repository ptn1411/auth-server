import { BaseApi, TokenManager, AuthServerConfig } from "./base";
import {
  StartRegistrationRequest,
  RegistrationOptionsResponse,
  FinishRegistrationRequest,
  PasskeyResponse,
  StartAuthenticationRequest,
  AuthenticationOptionsResponse,
  FinishAuthenticationRequest,
  PasskeyAuthResponse,
  RenameCredentialRequest,
} from "../types";

export class WebAuthnApi extends BaseApi {
  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    super(config, tokenManager);
  }

  async startRegistration(
    data?: StartRegistrationRequest
  ): Promise<RegistrationOptionsResponse> {
    return this.post("/auth/webauthn/register/start", data || {});
  }

  async finishRegistration(
    data: FinishRegistrationRequest
  ): Promise<PasskeyResponse> {
    return this.post("/auth/webauthn/register/finish", data);
  }

  async startAuthentication(
    data?: StartAuthenticationRequest
  ): Promise<AuthenticationOptionsResponse> {
    return this.request("POST", "/auth/webauthn/authenticate/start", {
      body: data || {},
      auth: false,
    });
  }

  async finishAuthentication(
    data: FinishAuthenticationRequest
  ): Promise<PasskeyAuthResponse> {
    const response = await this.request<PasskeyAuthResponse>(
      "POST",
      "/auth/webauthn/authenticate/finish",
      { body: data, auth: false }
    );
    this.tokenManager.setTokens(response.access_token, response.refresh_token);
    return response;
  }

  async list(): Promise<PasskeyResponse[]> {
    return this.get("/auth/webauthn/credentials");
  }

  async rename(credentialId: string, data: RenameCredentialRequest): Promise<void> {
    return this.put(`/auth/webauthn/credentials/${credentialId}`, data);
  }

  async remove(credentialId: string): Promise<void> {
    return this.request("DELETE", `/auth/webauthn/credentials/${credentialId}`);
  }
}

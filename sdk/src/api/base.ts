import { ApiError } from "../types";

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
    this.name = "AuthServerError";
  }
}

export interface TokenManager {
  getAccessToken(): string | undefined;
  getRefreshToken(): string | undefined;
  setTokens(accessToken: string, refreshToken?: string): void;
  clearTokens(): void;
  // API Key authentication (X-API-Key header)
  getAuthApiKey?(): string | undefined;
  setAuthApiKey?(apiKey: string): void;
  clearAuthApiKey?(): void;
}

export type AuthMode = "bearer" | "apiKey" | "none";

export class BaseApi {
  protected baseUrl: string;
  protected timeout: number;
  protected tokenManager: TokenManager;
  private isRefreshing = false;
  private refreshPromise: Promise<boolean> | null = null;

  constructor(config: AuthServerConfig, tokenManager: TokenManager) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.timeout = config.timeout || 30000;
    this.tokenManager = tokenManager;
  }

  private async tryRefreshToken(): Promise<boolean> {
    const refreshToken = this.tokenManager.getRefreshToken();
    if (!refreshToken) return false;

    if (this.isRefreshing && this.refreshPromise) {
      return this.refreshPromise;
    }

    this.isRefreshing = true;
    this.refreshPromise = (async () => {
      try {
        const response = await this.request<{ access_token: string }>(
          "POST",
          "/auth/refresh",
          { body: { refresh_token: refreshToken }, auth: "none" }
        );
        this.tokenManager.setTokens(response.access_token);
        return true;
      } catch {
        this.tokenManager.clearTokens();
        return false;
      } finally {
        this.isRefreshing = false;
        this.refreshPromise = null;
      }
    })();

    return this.refreshPromise;
  }

  protected async request<T>(
    method: string,
    path: string,
    options: {
      body?: unknown;
      query?: Record<string, string | number | boolean | undefined>;
      auth?: AuthMode | boolean;
      _retry?: boolean;
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
      "Content-Type": "application/json",
    };

    // Determine auth mode
    let authMode: AuthMode = "bearer";
    if (options.auth === false || options.auth === "none") {
      authMode = "none";
    } else if (options.auth === "apiKey") {
      authMode = "apiKey";
    } else if (options.auth === true || options.auth === "bearer" || options.auth === undefined) {
      authMode = "bearer";
    }

    // Apply authentication headers
    if (authMode === "bearer") {
      const accessToken = this.tokenManager.getAccessToken();
      if (accessToken) {
        headers["Authorization"] = `Bearer ${accessToken}`;
      }
    } else if (authMode === "apiKey") {
      const apiKey = this.tokenManager.getAuthApiKey?.();
      if (apiKey) {
        headers["X-API-Key"] = apiKey;
      }
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
        const error = (await response.json()) as ApiError;

        // Auto-refresh on 401 for bearer auth
        if (
          response.status === 401 &&
          error.error === "invalid_token" &&
          authMode === "bearer" &&
          !options._retry &&
          this.tokenManager.getRefreshToken()
        ) {
          const refreshed = await this.tryRefreshToken();
          if (refreshed) {
            return this.request<T>(method, path, { ...options, _retry: true });
          }
        }

        throw new AuthServerError(error.error, response.status, error.message);
      }

      if (response.status === 204) {
        return undefined as T;
      }

      return (await response.json()) as T;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof AuthServerError) throw error;
      throw new AuthServerError("network_error", 0, String(error));
    }
  }

  protected get<T>(
    path: string,
    query?: Record<string, string | number | boolean | undefined>,
    auth?: AuthMode | boolean
  ): Promise<T> {
    return this.request<T>("GET", path, { query, auth });
  }

  protected post<T>(path: string, body?: unknown, auth?: AuthMode | boolean): Promise<T> {
    return this.request<T>("POST", path, { body, auth });
  }

  protected put<T>(path: string, body?: unknown, auth?: AuthMode | boolean): Promise<T> {
    return this.request<T>("PUT", path, { body, auth });
  }

  protected delete<T>(path: string, auth?: AuthMode | boolean): Promise<T> {
    return this.request<T>("DELETE", path, { auth });
  }
}

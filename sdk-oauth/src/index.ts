/**
 * Auth Server OAuth SDK
 *
 * Easy OAuth2 integration for any web application.
 * Supports Authorization Code Flow with PKCE.
 */

// ============================================================================
// Types
// ============================================================================

export interface AuthServerConfig {
  /**
   * Auth Server URL (e.g., 'https://auth.example.com')
   */
  serverUrl: string;

  /**
   * OAuth Client ID
   */
  clientId: string;

  /**
   * Redirect URI after authentication
   * Must be registered in OAuth client settings
   */
  redirectUri: string;

  /**
   * OAuth scopes to request
   * @default ['openid', 'profile', 'email']
   */
  scopes?: string[];

  /**
   * Storage for tokens and state
   * @default localStorage
   */
  storage?: Storage;

  /**
   * Storage key prefix
   * @default 'authserver'
   */
  storagePrefix?: string;

  /**
   * Auto refresh token before expiry
   * @default true
   */
  autoRefresh?: boolean;

  /**
   * Refresh token before this many seconds of expiry
   * @default 60
   */
  refreshThreshold?: number;

  /**
   * Popup window width
   * @default 500
   */
  popupWidth?: number;

  /**
   * Popup window height
   * @default 600
   */
  popupHeight?: number;

  /**
   * Callback when tokens are updated
   */
  onTokenUpdate?: (tokens: TokenResponse | null) => void;

  /**
   * Callback when user info is fetched
   */
  onUserUpdate?: (user: UserInfo | null) => void;
}

export interface TokenResponse {
  access_token: string;
  refresh_token?: string;
  token_type: string;
  expires_in: number;
  scope?: string;
  id_token?: string;
}

export interface UserInfo {
  sub: string;
  email?: string;
  email_verified?: boolean;
  name?: string;
  picture?: string;
  [key: string]: unknown;
}

export interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: UserInfo | null;
  tokens: TokenResponse | null;
  error: string | null;
}

export interface PKCEChallenge {
  codeVerifier: string;
  codeChallenge: string;
  codeChallengeMethod: 'S256';
}

export interface AuthorizationParams {
  state: string;
  codeChallenge: string;
  codeChallengeMethod: string;
  nonce?: string;
}

// ============================================================================
// PKCE Utilities
// ============================================================================

/**
 * Generate a cryptographically random string
 */
function generateRandomString(length: number = 64): string {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const randomValues = new Uint8Array(length);
  crypto.getRandomValues(randomValues);
  return Array.from(randomValues)
    .map((v) => charset[v % charset.length])
    .join('');
}

/**
 * Generate SHA-256 hash and base64url encode
 */
async function sha256Base64Url(input: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(input);
  const digest = await crypto.subtle.digest('SHA-256', data);

  const base64 = btoa(String.fromCharCode(...new Uint8Array(digest)));
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * Generate PKCE challenge pair
 */
export async function generatePKCE(): Promise<PKCEChallenge> {
  const codeVerifier = generateRandomString(64);
  const codeChallenge = await sha256Base64Url(codeVerifier);

  return {
    codeVerifier,
    codeChallenge,
    codeChallengeMethod: 'S256',
  };
}

// ============================================================================
// JWT Utilities
// ============================================================================

/**
 * Decode JWT payload without verification
 */
export function decodeJwt(token: string): Record<string, unknown> | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;

    const payload = parts[1];
    const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
    return JSON.parse(decoded);
  } catch {
    return null;
  }
}

/**
 * Check if JWT is expired
 */
export function isTokenExpired(token: string, thresholdSeconds: number = 0): boolean {
  const payload = decodeJwt(token);
  if (!payload || typeof payload.exp !== 'number') return true;

  const expiresAt = payload.exp * 1000;
  const now = Date.now();
  return now >= expiresAt - thresholdSeconds * 1000;
}

// ============================================================================
// Storage Utilities
// ============================================================================

class AuthStorage {
  private storage: Storage;
  private prefix: string;

  constructor(storage: Storage, prefix: string) {
    this.storage = storage;
    this.prefix = prefix;
  }

  private key(name: string): string {
    return `${this.prefix}_${name}`;
  }

  get<T>(name: string): T | null {
    try {
      const value = this.storage.getItem(this.key(name));
      return value ? JSON.parse(value) : null;
    } catch {
      return null;
    }
  }

  set<T>(name: string, value: T): void {
    try {
      this.storage.setItem(this.key(name), JSON.stringify(value));
    } catch {
      // Storage full or unavailable
    }
  }

  remove(name: string): void {
    try {
      this.storage.removeItem(this.key(name));
    } catch {
      // Ignore
    }
  }

  clear(): void {
    const keysToRemove: string[] = [];
    for (let i = 0; i < this.storage.length; i++) {
      const key = this.storage.key(i);
      if (key?.startsWith(this.prefix)) {
        keysToRemove.push(key);
      }
    }
    keysToRemove.forEach((key) => this.storage.removeItem(key));
  }
}

// ============================================================================
// Main Auth Client
// ============================================================================

export class AuthServerClient {
  private config: Required<
    Pick<AuthServerConfig, 'serverUrl' | 'clientId' | 'redirectUri' | 'scopes' | 'storagePrefix' | 'autoRefresh' | 'refreshThreshold' | 'popupWidth' | 'popupHeight'>
  > &
    Pick<AuthServerConfig, 'onTokenUpdate' | 'onUserUpdate'>;

  private storage: AuthStorage;
  private popup: Window | null = null;
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(config: AuthServerConfig) {
    this.config = {
      serverUrl: config.serverUrl.replace(/\/$/, ''),
      clientId: config.clientId,
      redirectUri: config.redirectUri,
      scopes: config.scopes || ['openid', 'profile', 'email'],
      storagePrefix: config.storagePrefix || 'authserver',
      autoRefresh: config.autoRefresh ?? true,
      refreshThreshold: config.refreshThreshold || 60,
      popupWidth: config.popupWidth || 500,
      popupHeight: config.popupHeight || 600,
      onTokenUpdate: config.onTokenUpdate,
      onUserUpdate: config.onUserUpdate,
    };

    const storage = config.storage || (typeof localStorage !== 'undefined' ? localStorage : null);
    if (!storage) {
      throw new Error('No storage available. Please provide a storage option.');
    }
    this.storage = new AuthStorage(storage, this.config.storagePrefix);

    // Setup auto refresh if enabled and we have tokens
    if (this.config.autoRefresh) {
      this.setupAutoRefresh();
    }
  }

  // ============================================================================
  // Public API
  // ============================================================================

  /**
   * Get current authentication state
   */
  getState(): AuthState {
    const tokens = this.getTokens();
    const user = this.getUser();

    return {
      isAuthenticated: !!tokens && !isTokenExpired(tokens.access_token),
      isLoading: false,
      user,
      tokens,
      error: null,
    };
  }

  /**
   * Get stored tokens
   */
  getTokens(): TokenResponse | null {
    return this.storage.get<TokenResponse>('tokens');
  }

  /**
   * Get stored user info
   */
  getUser(): UserInfo | null {
    return this.storage.get<UserInfo>('user');
  }

  /**
   * Get access token (refreshes if needed)
   */
  async getAccessToken(): Promise<string | null> {
    const tokens = this.getTokens();
    if (!tokens) return null;

    // Check if token needs refresh
    if (isTokenExpired(tokens.access_token, this.config.refreshThreshold)) {
      if (tokens.refresh_token) {
        try {
          const newTokens = await this.refreshToken();
          return newTokens.access_token;
        } catch {
          return null;
        }
      }
      return null;
    }

    return tokens.access_token;
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    const tokens = this.getTokens();
    return !!tokens && !isTokenExpired(tokens.access_token);
  }

  /**
   * Login using popup window
   */
  async loginWithPopup(): Promise<TokenResponse> {
    // Generate PKCE
    const pkce = await generatePKCE();
    const state = generateRandomString(32);
    const nonce = generateRandomString(32);

    // Store for verification
    this.storage.set('pkce', pkce);
    this.storage.set('state', state);
    this.storage.set('nonce', nonce);

    // Build authorization URL
    const authUrl = this.buildAuthorizationUrl({
      state,
      codeChallenge: pkce.codeChallenge,
      codeChallengeMethod: pkce.codeChallengeMethod,
      nonce,
    });

    return new Promise((resolve, reject) => {
      // Calculate popup position
      const left = Math.max(0, (window.screen.width - this.config.popupWidth) / 2);
      const top = Math.max(0, (window.screen.height - this.config.popupHeight) / 2);

      // Open popup
      this.popup = window.open(
        authUrl,
        'authserver_login',
        `width=${this.config.popupWidth},height=${this.config.popupHeight},left=${left},top=${top},scrollbars=yes,resizable=yes`
      );

      if (!this.popup) {
        reject(new Error('Failed to open popup. Please allow popups for this site.'));
        return;
      }

      // Listen for callback message
      const messageHandler = async (event: MessageEvent) => {
        // Handle different message formats
        let data = event.data;

        // Sveltia CMS format: 'authorization:provider:state:json'
        if (typeof data === 'string' && data.startsWith('authorization:')) {
          const parts = data.split(':');
          if (parts.length >= 4) {
            try {
              const jsonStr = parts.slice(3).join(':');
              const parsed = JSON.parse(jsonStr);
              data = {
                type: 'oauth_callback',
                ...parsed,
              };
            } catch {
              return;
            }
          }
        }

        // Check for our callback
        if (typeof data !== 'object' || data.type !== 'oauth_callback') {
          return;
        }

        window.removeEventListener('message', messageHandler);
        this.popup?.close();
        this.popup = null;

        if (data.error) {
          reject(new Error(data.error_description || data.error));
          return;
        }

        if (data.code) {
          try {
            const tokens = await this.exchangeCode(data.code, data.state);
            resolve(tokens);
          } catch (err) {
            reject(err);
          }
        } else if (data.token) {
          // Direct token (from proxy worker)
          const tokens: TokenResponse = {
            access_token: data.token,
            token_type: 'Bearer',
            expires_in: 3600,
          };
          this.setTokens(tokens);
          resolve(tokens);
        }
      };

      window.addEventListener('message', messageHandler);

      // Poll for popup close
      const pollTimer = setInterval(() => {
        if (this.popup?.closed) {
          clearInterval(pollTimer);
          window.removeEventListener('message', messageHandler);
          this.popup = null;
          reject(new Error('Login cancelled'));
        }
      }, 500);
    });
  }

  /**
   * Login using redirect (full page)
   */
  loginWithRedirect(): void {
    // Generate and store PKCE
    generatePKCE().then((pkce) => {
      const state = generateRandomString(32);
      const nonce = generateRandomString(32);

      this.storage.set('pkce', pkce);
      this.storage.set('state', state);
      this.storage.set('nonce', nonce);

      const authUrl = this.buildAuthorizationUrl({
        state,
        codeChallenge: pkce.codeChallenge,
        codeChallengeMethod: pkce.codeChallengeMethod,
        nonce,
      });

      window.location.href = authUrl;
    });
  }

  /**
   * Handle redirect callback
   * Call this on your redirect URI page
   */
  async handleRedirectCallback(): Promise<TokenResponse> {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');
    const error = params.get('error');
    const errorDescription = params.get('error_description');

    if (error) {
      throw new Error(errorDescription || error);
    }

    if (!code) {
      throw new Error('No authorization code received');
    }

    const tokens = await this.exchangeCode(code, state);

    // Clean URL
    window.history.replaceState({}, document.title, window.location.pathname);

    return tokens;
  }

  /**
   * Refresh access token
   */
  async refreshToken(): Promise<TokenResponse> {
    const tokens = this.getTokens();
    if (!tokens?.refresh_token) {
      throw new Error('No refresh token available');
    }

    const response = await fetch(`${this.config.serverUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: tokens.refresh_token,
        client_id: this.config.clientId,
      }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      this.clearTokens();
      throw new Error(error.error_description || error.error || 'Token refresh failed');
    }

    const newTokens = await response.json();
    this.setTokens(newTokens);
    return newTokens;
  }

  /**
   * Fetch user info from server
   */
  async fetchUserInfo(): Promise<UserInfo> {
    const accessToken = await this.getAccessToken();
    if (!accessToken) {
      throw new Error('Not authenticated');
    }

    const response = await fetch(`${this.config.serverUrl}/oauth/userinfo`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    });

    if (!response.ok) {
      throw new Error('Failed to fetch user info');
    }

    const user = await response.json();
    this.storage.set('user', user);
    this.config.onUserUpdate?.(user);
    return user;
  }

  /**
   * Logout - revoke tokens and clear storage
   */
  async logout(options?: { revokeToken?: boolean }): Promise<void> {
    const tokens = this.getTokens();

    if (options?.revokeToken && tokens?.access_token) {
      try {
        await fetch(`${this.config.serverUrl}/oauth/revoke`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
          body: new URLSearchParams({
            token: tokens.access_token,
            client_id: this.config.clientId,
          }),
        });
      } catch {
        // Ignore revocation errors
      }
    }

    this.clearTokens();
  }

  /**
   * Clear all stored data
   */
  clearTokens(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }

    this.storage.clear();
    this.config.onTokenUpdate?.(null);
    this.config.onUserUpdate?.(null);
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private buildAuthorizationUrl(params: AuthorizationParams): string {
    const searchParams = new URLSearchParams({
      client_id: this.config.clientId,
      response_type: 'code',
      redirect_uri: this.config.redirectUri,
      scope: this.config.scopes.join(' '),
      state: params.state,
      code_challenge: params.codeChallenge,
      code_challenge_method: params.codeChallengeMethod,
    });

    if (params.nonce) {
      searchParams.set('nonce', params.nonce);
    }

    return `${this.config.serverUrl}/oauth/authorize?${searchParams.toString()}`;
  }

  private async exchangeCode(code: string, state: string | null): Promise<TokenResponse> {
    // Verify state
    const storedState = this.storage.get<string>('state');
    if (state !== storedState) {
      throw new Error('Invalid state parameter');
    }

    // Get PKCE verifier
    const pkce = this.storage.get<PKCEChallenge>('pkce');
    if (!pkce) {
      throw new Error('PKCE data not found');
    }

    const response = await fetch(`${this.config.serverUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        redirect_uri: this.config.redirectUri,
        client_id: this.config.clientId,
        code_verifier: pkce.codeVerifier,
      }),
    });

    // Clean up PKCE data
    this.storage.remove('pkce');
    this.storage.remove('state');
    this.storage.remove('nonce');

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.error_description || error.error || 'Token exchange failed');
    }

    const tokens = await response.json();
    this.setTokens(tokens);

    // Fetch user info
    try {
      await this.fetchUserInfo();
    } catch {
      // User info fetch is optional
    }

    return tokens;
  }

  private setTokens(tokens: TokenResponse): void {
    this.storage.set('tokens', tokens);
    this.config.onTokenUpdate?.(tokens);

    if (this.config.autoRefresh) {
      this.setupAutoRefresh();
    }
  }

  private setupAutoRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }

    const tokens = this.getTokens();
    if (!tokens?.refresh_token) return;

    const payload = decodeJwt(tokens.access_token);
    if (!payload || typeof payload.exp !== 'number') return;

    const expiresAt = payload.exp * 1000;
    const refreshAt = expiresAt - this.config.refreshThreshold * 1000;
    const delay = Math.max(0, refreshAt - Date.now());

    this.refreshTimer = setTimeout(async () => {
      try {
        await this.refreshToken();
      } catch {
        // Token refresh failed, user needs to re-login
      }
    }, delay);
  }
}

// ============================================================================
// React Hook (optional)
// ============================================================================

// Type-only import for React (doesn't require React at runtime)
type ReactType = typeof import('react');

/**
 * Create a React hook for Auth Server
 * Usage: const { useAuth } = createAuthHook(authClient)
 */
export function createAuthHook(client: AuthServerClient) {
  // This is a factory function that returns a hook
  // The actual React import should be done by the consumer
  return {
    /**
     * React hook for authentication state
     * Requires React to be installed in the consuming project
     */
    useAuth: () => {
      // Dynamic import check for React
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      let React: ReactType;
      try {
        // eslint-disable-next-line @typescript-eslint/no-require-imports
        React = require('react') as ReactType;
      } catch {
        throw new Error('React is required to use useAuth hook. Please install react as a dependency.');
      }

      const [state, setState] = React.useState<AuthState>(client.getState());

      React.useEffect(() => {
        // Update state when tokens change
        const updateState = () => {
          setState(client.getState());
        };

        // Listen for storage events (cross-tab sync)
        window.addEventListener('storage', updateState);

        return () => {
          window.removeEventListener('storage', updateState);
        };
      }, []);

      const login = React.useCallback(async () => {
        setState((s) => ({ ...s, isLoading: true, error: null }));
        try {
          await client.loginWithPopup();
          setState(client.getState());
        } catch (err) {
          setState((s) => ({
            ...s,
            isLoading: false,
            error: err instanceof Error ? err.message : 'Login failed',
          }));
        }
      }, []);

      const logout = React.useCallback(async () => {
        await client.logout({ revokeToken: true });
        setState(client.getState());
      }, []);

      const getAccessToken = React.useCallback(() => {
        return client.getAccessToken();
      }, []);

      return {
        ...state,
        login,
        logout,
        getAccessToken,
        loginWithRedirect: () => client.loginWithRedirect(),
      };
    },
  };
}

// ============================================================================
// Exports
// ============================================================================

export default AuthServerClient;

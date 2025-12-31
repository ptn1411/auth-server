/**
 * OAuth2 Popup Flow SDK
 * 
 * Provides utilities for OAuth2 authorization using popup windows
 * with postMessage communication for seamless integration.
 */

export interface OAuthConfig {
  /** Base URL of the auth server */
  authServerUrl: string;
  /** OAuth client ID */
  clientId: string;
  /** Redirect URI registered with the OAuth server */
  redirectUri: string;
  /** Requested scopes (space-separated or array) */
  scopes?: string | string[];
  /** Popup window width */
  popupWidth?: number;
  /** Popup window height */
  popupHeight?: number;
}

export interface OAuthResult {
  /** Authorization code received from the server */
  code: string;
  /** State parameter (for CSRF protection) */
  state?: string;
}

export interface OAuthError {
  error: string;
  error_description?: string;
  state?: string;
}

export interface PKCEChallenge {
  codeVerifier: string;
  codeChallenge: string;
  codeChallengeMethod: 'S256';
}

/**
 * Generate a random string for state/code_verifier
 */
function generateRandomString(length: number = 43): string {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const randomValues = new Uint8Array(length);
  crypto.getRandomValues(randomValues);
  return Array.from(randomValues)
    .map((v) => charset[v % charset.length])
    .join('');
}

/**
 * Generate PKCE code challenge from code verifier
 */
async function generateCodeChallenge(codeVerifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(codeVerifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  
  // Base64URL encode
  const base64 = btoa(String.fromCharCode(...new Uint8Array(digest)));
  return base64
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

/**
 * Generate PKCE challenge pair
 */
export async function generatePKCE(): Promise<PKCEChallenge> {
  const codeVerifier = generateRandomString(64);
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  
  return {
    codeVerifier,
    codeChallenge,
    codeChallengeMethod: 'S256',
  };
}

/**
 * OAuth2 Client for popup-based authorization flow
 */
export class OAuthClient {
  private config: Required<OAuthConfig>;
  private popup: Window | null = null;
  private messageHandler: ((event: MessageEvent) => void) | null = null;

  constructor(config: OAuthConfig) {
    this.config = {
      authServerUrl: config.authServerUrl.replace(/\/$/, ''),
      clientId: config.clientId,
      redirectUri: config.redirectUri,
      scopes: config.scopes || 'openid profile email',
      popupWidth: config.popupWidth || 500,
      popupHeight: config.popupHeight || 600,
    };
  }

  /**
   * Build the authorization URL with all required parameters
   */
  buildAuthorizationUrl(options: {
    state: string;
    codeChallenge: string;
    codeChallengeMethod: string;
  }): string {
    const scopes = Array.isArray(this.config.scopes)
      ? this.config.scopes.join(' ')
      : this.config.scopes;

    const params = new URLSearchParams({
      client_id: this.config.clientId,
      response_type: 'code',
      redirect_uri: this.config.redirectUri,
      scope: scopes,
      state: options.state,
      code_challenge: options.codeChallenge,
      code_challenge_method: options.codeChallengeMethod,
    });

    return `${this.config.authServerUrl}/oauth/authorize?${params.toString()}`;
  }

  /**
   * Open authorization popup and wait for result
   * 
   * @returns Promise that resolves with authorization code or rejects with error
   */
  async authorize(): Promise<OAuthResult & { codeVerifier: string }> {
    // Generate PKCE challenge
    const pkce = await generatePKCE();
    const state = generateRandomString(32);

    // Build authorization URL
    const authUrl = this.buildAuthorizationUrl({
      state,
      codeChallenge: pkce.codeChallenge,
      codeChallengeMethod: pkce.codeChallengeMethod,
    });

    return new Promise((resolve, reject) => {
      // Calculate popup position (center of screen)
      const left = Math.max(0, (window.screen.width - this.config.popupWidth) / 2);
      const top = Math.max(0, (window.screen.height - this.config.popupHeight) / 2);

      // Open popup
      this.popup = window.open(
        authUrl,
        'oauth_popup',
        `width=${this.config.popupWidth},height=${this.config.popupHeight},left=${left},top=${top},scrollbars=yes,resizable=yes`
      );

      if (!this.popup) {
        reject(new Error('Failed to open popup window. Please allow popups for this site.'));
        return;
      }

      // Set up message listener
      this.messageHandler = (event: MessageEvent) => {
        // Validate origin
        if (event.origin !== this.config.authServerUrl && event.origin !== window.location.origin) {
          return;
        }

        const data = event.data;

        // Handle authorization response
        if (typeof data === 'object' && data !== null) {
          if (data.type === 'oauth_callback') {
            this.cleanup();

            if (data.error) {
              reject({
                error: data.error,
                error_description: data.error_description,
                state: data.state,
              } as OAuthError);
            } else if (data.code) {
              // Validate state
              if (data.state !== state) {
                reject({
                  error: 'invalid_state',
                  error_description: 'State parameter mismatch',
                } as OAuthError);
                return;
              }

              resolve({
                code: data.code,
                state: data.state,
                codeVerifier: pkce.codeVerifier,
              });
            }
          }
        }
      };

      window.addEventListener('message', this.messageHandler);

      // Poll for popup close (user cancelled)
      const pollTimer = setInterval(() => {
        if (this.popup?.closed) {
          clearInterval(pollTimer);
          this.cleanup();
          reject({
            error: 'access_denied',
            error_description: 'User closed the authorization window',
          } as OAuthError);
        }
      }, 500);
    });
  }

  /**
   * Exchange authorization code for tokens
   */
  async exchangeCodeForTokens(
    code: string,
    codeVerifier: string
  ): Promise<{
    access_token: string;
    refresh_token?: string;
    token_type: string;
    expires_in: number;
    scope: string;
  }> {
    const response = await fetch(`${this.config.authServerUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        redirect_uri: this.config.redirectUri,
        client_id: this.config.clientId,
        code_verifier: codeVerifier,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw {
        error: error.error || 'token_error',
        error_description: error.error_description || 'Failed to exchange code for tokens',
      } as OAuthError;
    }

    return response.json();
  }

  /**
   * Full authorization flow: open popup, get code, exchange for tokens
   */
  async login(): Promise<{
    access_token: string;
    refresh_token?: string;
    token_type: string;
    expires_in: number;
    scope: string;
  }> {
    const { code, codeVerifier } = await this.authorize();
    return this.exchangeCodeForTokens(code, codeVerifier);
  }

  /**
   * Clean up popup and event listeners
   */
  private cleanup(): void {
    if (this.messageHandler) {
      window.removeEventListener('message', this.messageHandler);
      this.messageHandler = null;
    }
    if (this.popup && !this.popup.closed) {
      this.popup.close();
    }
    this.popup = null;
  }

  /**
   * Cancel any ongoing authorization
   */
  cancel(): void {
    this.cleanup();
  }
}

/**
 * Script to be injected into the OAuth callback page
 * This sends the authorization result back to the opener window
 * 
 * Usage: Include this script in your redirect_uri page
 */
export function createCallbackScript(): string {
  return `
(function() {
  // Parse URL parameters
  const params = new URLSearchParams(window.location.search);
  const code = params.get('code');
  const state = params.get('state');
  const error = params.get('error');
  const errorDescription = params.get('error_description');

  // Build message
  const message = {
    type: 'oauth_callback',
    code: code,
    state: state,
    error: error,
    error_description: errorDescription,
  };

  // Send to opener
  if (window.opener) {
    window.opener.postMessage(message, '*');
    // Close popup after sending message
    setTimeout(() => window.close(), 100);
  } else {
    // If no opener, display result (for debugging)
    document.body.innerHTML = '<pre>' + JSON.stringify(message, null, 2) + '</pre>';
  }
})();
`;
}

/**
 * Helper to create a callback page HTML
 */
export function createCallbackPageHtml(): string {
  return `<!DOCTYPE html>
<html>
<head>
  <title>Authorization Callback</title>
</head>
<body>
  <p>Processing authorization...</p>
  <script>${createCallbackScript()}</script>
</body>
</html>`;
}

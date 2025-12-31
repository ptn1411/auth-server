/**
 * Auth Server OAuth Proxy for Cloudflare Workers
 * 
 * This worker acts as an OAuth proxy for applications like Sveltia CMS
 * to authenticate with your Auth Server using OAuth2 Authorization Code Flow with PKCE.
 */

/**
 * Escape the given string for safe use in a regular expression.
 * @param {string} str - Original string.
 * @returns {string} Escaped string.
 */
const escapeRegExp = (str) => str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/**
 * Generate a random string for PKCE code verifier
 * @param {number} length - Length of the string
 * @returns {string} Random string
 */
const generateRandomString = (length = 64) => {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const randomValues = new Uint8Array(length);
  crypto.getRandomValues(randomValues);
  return Array.from(randomValues)
    .map((v) => charset[v % charset.length])
    .join('');
};

/**
 * Generate PKCE code challenge from code verifier (S256)
 * @param {string} codeVerifier - The code verifier
 * @returns {Promise<string>} Base64URL encoded code challenge
 */
const generateCodeChallenge = async (codeVerifier) => {
  const encoder = new TextEncoder();
  const data = encoder.encode(codeVerifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  
  // Base64URL encode
  const base64 = btoa(String.fromCharCode(...new Uint8Array(digest)));
  return base64
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
};

/**
 * Output HTML response that communicates with the window opener.
 * @param {object} args - Options.
 * @param {string} [args.provider] - Backend name.
 * @param {string} [args.token] - OAuth token.
 * @param {string} [args.error] - Error message.
 * @param {string} [args.errorCode] - Error code.
 * @returns {Response} Response with HTML.
 */
const outputHTML = ({ provider = 'auth-server', token, error, errorCode }) => {
  const state = error ? 'error' : 'success';
  const content = error ? { provider, error, errorCode } : { provider, token };

  return new Response(
    `<!doctype html>
<html>
<head>
  <title>OAuth Callback</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      display: flex;
      justify-content: center;
      align-items: center;
      height: 100vh;
      margin: 0;
      background: #f5f5f5;
    }
    .container {
      text-align: center;
      padding: 2rem;
      background: white;
      border-radius: 8px;
      box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    }
    .success { color: #22c55e; }
    .error { color: #ef4444; }
    .spinner {
      border: 3px solid #f3f3f3;
      border-top: 3px solid #3498db;
      border-radius: 50%;
      width: 30px;
      height: 30px;
      animation: spin 1s linear infinite;
      margin: 0 auto 1rem;
    }
    @keyframes spin {
      0% { transform: rotate(0deg); }
      100% { transform: rotate(360deg); }
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="spinner"></div>
    <p class="${state}">${error ? 'Authentication failed' : 'Authentication successful'}</p>
    <p>${error || 'Redirecting...'}</p>
  </div>
  <script>
    (() => {
      const content = ${JSON.stringify(content)};
      const state = '${state}';
      const provider = '${provider}';
      
      // For Sveltia CMS / Netlify CMS compatibility
      window.addEventListener('message', ({ data, origin }) => {
        if (data === 'authorizing:' + provider) {
          window.opener?.postMessage(
            'authorization:' + provider + ':' + state + ':' + JSON.stringify(content),
            origin
          );
        }
      });
      
      // Send initial message
      window.opener?.postMessage('authorizing:' + provider, '*');
      
      // Also try generic OAuth callback for other integrations
      if (window.opener) {
        window.opener.postMessage({
          type: 'oauth_callback',
          provider: provider,
          state: state,
          ...content
        }, '*');
        
        // Close window after a short delay
        setTimeout(() => window.close(), 1000);
      }
    })();
  </script>
</body>
</html>`,
    {
      headers: {
        'Content-Type': 'text/html;charset=UTF-8',
        'Set-Cookie': `oauth-state=deleted; HttpOnly; Max-Age=0; Path=/; SameSite=Lax; Secure`,
      },
    },
  );
};

/**
 * Handle the `auth` method - initiates the OAuth flow
 * @param {Request} request - HTTP request.
 * @param {{ [key: string]: string }} env - Environment variables.
 * @returns {Promise<Response>} HTTP response.
 */
const handleAuth = async (request, env) => {
  const { url } = request;
  const { origin, searchParams } = new URL(url);
  const { provider = 'auth-server', site_id: domain, scope } = Object.fromEntries(searchParams);

  const {
    ALLOWED_DOMAINS,
    AUTH_SERVER_URL,
    OAUTH_CLIENT_ID,
    OAUTH_CLIENT_SECRET,
    OAUTH_REDIRECT_URI,
  } = env;

  // Validate configuration
  if (!AUTH_SERVER_URL || !OAUTH_CLIENT_ID) {
    return outputHTML({
      provider,
      error: 'OAuth is not configured. Please set AUTH_SERVER_URL and OAUTH_CLIENT_ID.',
      errorCode: 'MISCONFIGURED_CLIENT',
    });
  }

  // Check if the domain is whitelisted
  if (ALLOWED_DOMAINS && domain) {
    const isAllowed = ALLOWED_DOMAINS.split(/,/).some((str) =>
      (domain ?? '').match(new RegExp(`^${escapeRegExp(str.trim()).replace('\\*', '.+')}$`))
    );
    
    if (!isAllowed) {
      return outputHTML({
        provider,
        error: 'Your domain is not allowed to use the authenticator.',
        errorCode: 'UNSUPPORTED_DOMAIN',
      });
    }
  }

  // Generate PKCE code verifier and challenge
  const codeVerifier = generateRandomString(64);
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  
  // Generate state for CSRF protection
  const state = crypto.randomUUID().replaceAll('-', '');

  // Build redirect URI
  const redirectUri = OAUTH_REDIRECT_URI || `${origin}/callback`;

  // Build authorization URL
  const params = new URLSearchParams({
    client_id: OAUTH_CLIENT_ID,
    response_type: 'code',
    redirect_uri: redirectUri,
    scope: scope || 'openid profile email',
    state: state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });

  const authURL = `${AUTH_SERVER_URL}/oauth/authorize?${params.toString()}`;

  // Store state and code_verifier in cookie (encrypted in production)
  const cookieData = JSON.stringify({ state, codeVerifier, redirectUri });
  const encodedCookie = btoa(cookieData);

  return new Response('', {
    status: 302,
    headers: {
      Location: authURL,
      'Set-Cookie': `oauth-state=${encodedCookie}; HttpOnly; Path=/; Max-Age=600; SameSite=Lax; Secure`,
    },
  });
};

/**
 * Handle the `callback` method - exchanges code for token
 * @param {Request} request - HTTP request.
 * @param {{ [key: string]: string }} env - Environment variables.
 * @returns {Promise<Response>} HTTP response.
 */
const handleCallback = async (request, env) => {
  const { url, headers } = request;
  const { searchParams } = new URL(url);
  const { code, state, error, error_description } = Object.fromEntries(searchParams);

  const provider = 'auth-server';

  // Check for OAuth error response
  if (error) {
    return outputHTML({
      provider,
      error: error_description || error,
      errorCode: error.toUpperCase(),
    });
  }

  // Get stored state from cookie
  const cookieHeader = headers.get('Cookie') || '';
  const cookieMatch = cookieHeader.match(/oauth-state=([^;]+)/);
  
  if (!cookieMatch) {
    return outputHTML({
      provider,
      error: 'Session expired. Please try again.',
      errorCode: 'SESSION_EXPIRED',
    });
  }

  let storedData;
  try {
    storedData = JSON.parse(atob(cookieMatch[1]));
  } catch {
    return outputHTML({
      provider,
      error: 'Invalid session data. Please try again.',
      errorCode: 'INVALID_SESSION',
    });
  }

  const { state: storedState, codeVerifier, redirectUri } = storedData;

  // Validate state (CSRF protection)
  if (!state || state !== storedState) {
    return outputHTML({
      provider,
      error: 'Potential CSRF attack detected. Authentication flow aborted.',
      errorCode: 'CSRF_DETECTED',
    });
  }

  if (!code) {
    return outputHTML({
      provider,
      error: 'Failed to receive an authorization code. Please try again later.',
      errorCode: 'AUTH_CODE_REQUEST_FAILED',
    });
  }

  const { AUTH_SERVER_URL, OAUTH_CLIENT_ID, OAUTH_CLIENT_SECRET } = env;

  // Exchange code for token
  const tokenURL = `${AUTH_SERVER_URL}/oauth/token`;
  const tokenBody = new URLSearchParams({
    grant_type: 'authorization_code',
    code: code,
    redirect_uri: redirectUri,
    client_id: OAUTH_CLIENT_ID,
    code_verifier: codeVerifier,
  });

  // Add client_secret if configured (for confidential clients)
  if (OAUTH_CLIENT_SECRET) {
    tokenBody.append('client_secret', OAUTH_CLIENT_SECRET);
  }

  let response;
  try {
    response = await fetch(tokenURL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Accept': 'application/json',
      },
      body: tokenBody.toString(),
    });
  } catch (err) {
    return outputHTML({
      provider,
      error: 'Failed to connect to authentication server.',
      errorCode: 'CONNECTION_FAILED',
    });
  }

  if (!response.ok) {
    let errorData;
    try {
      errorData = await response.json();
    } catch {
      errorData = { error: 'unknown_error' };
    }
    
    return outputHTML({
      provider,
      error: errorData.error_description || errorData.error || 'Token exchange failed',
      errorCode: (errorData.error || 'TOKEN_EXCHANGE_FAILED').toUpperCase(),
    });
  }

  let tokenData;
  try {
    tokenData = await response.json();
  } catch {
    return outputHTML({
      provider,
      error: 'Server responded with malformed data.',
      errorCode: 'MALFORMED_RESPONSE',
    });
  }

  // Return the access token
  return outputHTML({
    provider,
    token: tokenData.access_token,
  });
};

/**
 * Handle userinfo request - proxy to auth server
 * @param {Request} request - HTTP request.
 * @param {{ [key: string]: string }} env - Environment variables.
 * @returns {Promise<Response>} HTTP response.
 */
const handleUserInfo = async (request, env) => {
  const authHeader = request.headers.get('Authorization');
  
  if (!authHeader) {
    return new Response(JSON.stringify({ error: 'unauthorized' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const { AUTH_SERVER_URL } = env;
  
  try {
    const response = await fetch(`${AUTH_SERVER_URL}/oauth/userinfo`, {
      headers: {
        'Authorization': authHeader,
        'Accept': 'application/json',
      },
    });
    
    const data = await response.json();
    return new Response(JSON.stringify(data), {
      status: response.status,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch {
    return new Response(JSON.stringify({ error: 'server_error' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }
};

export default {
  /**
   * Main request handler
   * @param {Request} request - HTTP request.
   * @param {{ [key: string]: string }} env - Environment variables.
   * @returns {Promise<Response>} HTTP response.
   */
  async fetch(request, env) {
    const { method, url } = request;
    const { pathname } = new URL(url);

    // Handle CORS preflight
    if (method === 'OPTIONS') {
      return new Response(null, {
        headers: {
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
          'Access-Control-Allow-Headers': 'Authorization, Content-Type',
          'Access-Control-Max-Age': '86400',
        },
      });
    }

    // Auth endpoint - initiates OAuth flow
    if (method === 'GET' && ['/auth', '/oauth/authorize'].includes(pathname)) {
      return handleAuth(request, env);
    }

    // Callback endpoint - handles OAuth callback
    if (method === 'GET' && ['/callback', '/oauth/callback', '/oauth/redirect'].includes(pathname)) {
      return handleCallback(request, env);
    }

    // UserInfo proxy endpoint
    if (method === 'GET' && pathname === '/userinfo') {
      return handleUserInfo(request, env);
    }

    // Health check
    if (method === 'GET' && pathname === '/health') {
      return new Response(JSON.stringify({ status: 'ok' }), {
        headers: { 'Content-Type': 'application/json' },
      });
    }

    // Not found
    return new Response('Not Found', { status: 404 });
  },
};

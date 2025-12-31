/**
 * OAuth2 Demo Application
 * Tests OAuth2 Authorization Code Flow with PKCE
 */

// State
let accessToken = null;
let refreshToken = null;
let codeVerifier = null;

// DOM Elements
const elements = {
  authServerUrl: document.getElementById('auth-server-url'),
  clientId: document.getElementById('client-id'),
  redirectUri: document.getElementById('redirect-uri'),
  scopes: document.getElementById('scopes'),
  loginBtn: document.getElementById('login-btn'),
  loginStatus: document.getElementById('login-status'),
  tokenSection: document.getElementById('token-section'),
  accessToken: document.getElementById('access-token'),
  refreshToken: document.getElementById('refresh-token'),
  tokenType: document.getElementById('token-type'),
  expiresIn: document.getElementById('expires-in'),
  tokenScopes: document.getElementById('token-scopes'),
  userinfoBtn: document.getElementById('userinfo-btn'),
  logoutBtn: document.getElementById('logout-btn'),
  userinfoSection: document.getElementById('userinfo-section'),
  userinfoData: document.getElementById('userinfo-data'),
  debugLog: document.getElementById('debug-log'),
  clearLogBtn: document.getElementById('clear-log-btn'),
};

// ============ Logging ============

function log(message, type = 'info') {
  const time = new Date().toLocaleTimeString();
  const entry = document.createElement('div');
  entry.className = 'log-entry';
  entry.innerHTML = `<span class="log-time">[${time}]</span><span class="log-${type}">${message}</span>`;
  elements.debugLog.appendChild(entry);
  elements.debugLog.scrollTop = elements.debugLog.scrollHeight;
  console.log(`[${type.toUpperCase()}]`, message);
}

// ============ PKCE Utilities ============

function generateRandomString(length = 64) {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const randomValues = new Uint8Array(length);
  crypto.getRandomValues(randomValues);
  return Array.from(randomValues)
    .map((v) => charset[v % charset.length])
    .join('');
}

async function generateCodeChallenge(verifier) {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  
  // Base64URL encode
  const base64 = btoa(String.fromCharCode(...new Uint8Array(digest)));
  return base64
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

// ============ OAuth Flow ============

async function startAuthorization() {
  const authServerUrl = elements.authServerUrl.value.trim();
  const clientId = elements.clientId.value.trim();
  const redirectUri = elements.redirectUri.value.trim();
  const scopes = elements.scopes.value.trim();

  // Validate inputs
  if (!authServerUrl || !clientId || !redirectUri) {
    showStatus('Please fill in all required fields', 'error');
    return;
  }

  log('Starting OAuth2 Authorization Code Flow with PKCE...', 'info');

  // Generate PKCE
  codeVerifier = generateRandomString(64);
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const state = generateRandomString(32);

  // Store state for validation
  sessionStorage.setItem('oauth_state', state);
  sessionStorage.setItem('oauth_code_verifier', codeVerifier);

  log(`Generated code_verifier: ${codeVerifier.substring(0, 20)}...`, 'info');
  log(`Generated code_challenge: ${codeChallenge.substring(0, 20)}...`, 'info');
  log(`Generated state: ${state}`, 'info');

  // Build authorization URL
  const params = new URLSearchParams({
    client_id: clientId,
    response_type: 'code',
    redirect_uri: redirectUri,
    scope: scopes,
    state: state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });

  const authUrl = `${authServerUrl}/oauth/authorize?${params.toString()}`;
  log(`Authorization URL: ${authUrl}`, 'info');

  // Open popup
  const width = 500;
  const height = 600;
  const left = (window.screen.width - width) / 2;
  const top = (window.screen.height - height) / 2;

  const popup = window.open(
    authUrl,
    'oauth_popup',
    `width=${width},height=${height},left=${left},top=${top},scrollbars=yes,resizable=yes`
  );

  if (!popup) {
    showStatus('Failed to open popup. Please allow popups for this site.', 'error');
    log('Popup blocked!', 'error');
    return;
  }

  log('Popup opened, waiting for authorization...', 'info');
  showStatus('Waiting for authorization...', 'info');
  elements.loginBtn.disabled = true;

  // Listen for callback message
  const messageHandler = async (event) => {
    // Accept messages from any origin for demo purposes
    const data = event.data;

    if (typeof data === 'object' && data.type === 'oauth_callback') {
      window.removeEventListener('message', messageHandler);
      elements.loginBtn.disabled = false;

      if (data.error) {
        log(`Authorization error: ${data.error} - ${data.error_description}`, 'error');
        showStatus(`Error: ${data.error_description || data.error}`, 'error');
        return;
      }

      if (data.code) {
        // Validate state
        const savedState = sessionStorage.getItem('oauth_state');
        if (data.state !== savedState) {
          log('State mismatch! Possible CSRF attack.', 'error');
          showStatus('Security error: State mismatch', 'error');
          return;
        }

        log(`Received authorization code: ${data.code.substring(0, 20)}...`, 'success');
        
        // Exchange code for tokens
        await exchangeCodeForTokens(data.code);
      }
    }
  };

  window.addEventListener('message', messageHandler);

  // Poll for popup close
  const pollTimer = setInterval(() => {
    if (popup.closed) {
      clearInterval(pollTimer);
      window.removeEventListener('message', messageHandler);
      elements.loginBtn.disabled = false;
      
      if (!accessToken) {
        log('Popup closed without completing authorization', 'warn');
        showStatus('Authorization cancelled', 'error');
      }
    }
  }, 500);
}

async function exchangeCodeForTokens(code) {
  const authServerUrl = elements.authServerUrl.value.trim();
  const clientId = elements.clientId.value.trim();
  const redirectUri = elements.redirectUri.value.trim();
  const savedCodeVerifier = sessionStorage.getItem('oauth_code_verifier');

  log('Exchanging authorization code for tokens...', 'info');

  try {
    const response = await fetch(`${authServerUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code: code,
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: savedCodeVerifier,
      }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error_description || data.error || 'Token exchange failed');
    }

    log('Token exchange successful!', 'success');
    
    // Store tokens
    accessToken = data.access_token;
    refreshToken = data.refresh_token;

    // Display tokens
    displayTokens(data);
    showStatus('Successfully logged in!', 'success');

    // Clean up
    sessionStorage.removeItem('oauth_state');
    sessionStorage.removeItem('oauth_code_verifier');

  } catch (error) {
    log(`Token exchange error: ${error.message}`, 'error');
    showStatus(`Error: ${error.message}`, 'error');
  }
}

function displayTokens(data) {
  elements.tokenSection.classList.remove('hidden');
  elements.accessToken.textContent = data.access_token ? `${data.access_token.substring(0, 50)}...` : '-';
  elements.refreshToken.textContent = data.refresh_token ? `${data.refresh_token.substring(0, 50)}...` : '-';
  elements.tokenType.textContent = data.token_type || '-';
  elements.expiresIn.textContent = data.expires_in ? `${data.expires_in} seconds` : '-';
  elements.tokenScopes.textContent = data.scope || '-';

  log(`Access Token: ${data.access_token?.substring(0, 30)}...`, 'info');
  log(`Refresh Token: ${data.refresh_token?.substring(0, 30)}...`, 'info');
  log(`Expires in: ${data.expires_in} seconds`, 'info');
}

async function getUserInfo() {
  if (!accessToken) {
    showStatus('No access token available', 'error');
    return;
  }

  const authServerUrl = elements.authServerUrl.value.trim();
  log('Fetching user info...', 'info');

  try {
    const response = await fetch(`${authServerUrl}/oauth/userinfo`, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
      },
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error_description || data.error || 'Failed to get user info');
    }

    log('User info retrieved successfully!', 'success');
    elements.userinfoSection.classList.remove('hidden');
    elements.userinfoData.textContent = JSON.stringify(data, null, 2);

  } catch (error) {
    log(`User info error: ${error.message}`, 'error');
    showStatus(`Error: ${error.message}`, 'error');
  }
}

function logout() {
  accessToken = null;
  refreshToken = null;
  codeVerifier = null;
  
  elements.tokenSection.classList.add('hidden');
  elements.userinfoSection.classList.add('hidden');
  elements.loginStatus.classList.add('hidden');
  
  sessionStorage.removeItem('oauth_state');
  sessionStorage.removeItem('oauth_code_verifier');
  
  log('Logged out', 'info');
  showStatus('Logged out successfully', 'success');
}

function showStatus(message, type) {
  elements.loginStatus.textContent = message;
  elements.loginStatus.className = `status ${type}`;
  elements.loginStatus.classList.remove('hidden');
}

function clearLog() {
  elements.debugLog.innerHTML = '';
  log('Log cleared', 'info');
}

// ============ Event Listeners ============

elements.loginBtn.addEventListener('click', startAuthorization);
elements.userinfoBtn.addEventListener('click', getUserInfo);
elements.logoutBtn.addEventListener('click', logout);
elements.clearLogBtn.addEventListener('click', clearLog);

// Fetch scopes button
document.getElementById('fetch-scopes-btn').addEventListener('click', async () => {
  const authServerUrl = elements.authServerUrl.value.trim();
  log('Fetching available scopes...', 'info');
  
  try {
    const response = await fetch(`${authServerUrl}/oauth/scopes`);
    if (!response.ok) {
      throw new Error('Failed to fetch scopes');
    }
    const data = await response.json();
    const scopeCodes = data.scopes.map(s => s.code).join(', ');
    log(`Available scopes: ${scopeCodes}`, 'success');
    
    // Update hint
    const hint = document.querySelector('.scope-hint');
    if (hint) {
      hint.textContent = `Available: ${scopeCodes}`;
    }
  } catch (error) {
    log(`Failed to fetch scopes: ${error.message}`, 'error');
  }
});

// ============ Initialize ============

log('OAuth2 Demo App initialized', 'info');
log('Please configure your OAuth client settings and click "Login with Auth Server"', 'info');

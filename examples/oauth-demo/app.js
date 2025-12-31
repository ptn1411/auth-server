/**
 * OAuth2 Demo Application
 * Tests OAuth2 Authorization Code Flow with PKCE
 *
 * Features:
 * - Authorization Code Flow with PKCE
 * - Token Exchange
 * - Token Refresh
 * - Token Revocation
 * - User Info
 * - OpenID Configuration
 * - JWT Decoding
 */

// State
let accessToken = null;
let refreshToken = null;
let idToken = null;
let codeVerifier = null;

// DOM Elements
const elements = {
  authServerUrl: document.getElementById("auth-server-url"),
  frontendUrl: document.getElementById("frontend-url"),
  clientId: document.getElementById("client-id"),
  redirectUri: document.getElementById("redirect-uri"),
  scopes: document.getElementById("scopes"),
  loginBtn: document.getElementById("login-btn"),
  loginStatus: document.getElementById("login-status"),
  tokenSection: document.getElementById("token-section"),
  accessToken: document.getElementById("access-token"),
  refreshToken: document.getElementById("refresh-token"),
  idTokenDisplay: document.getElementById("id-token"),
  tokenType: document.getElementById("token-type"),
  expiresIn: document.getElementById("expires-in"),
  tokenScopes: document.getElementById("token-scopes"),
  userinfoBtn: document.getElementById("userinfo-btn"),
  refreshBtn: document.getElementById("refresh-btn"),
  revokeBtn: document.getElementById("revoke-btn"),
  logoutBtn: document.getElementById("logout-btn"),
  decodeBtn: document.getElementById("decode-btn"),
  userinfoSection: document.getElementById("userinfo-section"),
  userinfoData: document.getElementById("userinfo-data"),
  decodedSection: document.getElementById("decoded-section"),
  decodedData: document.getElementById("decoded-data"),
  oidcSection: document.getElementById("oidc-section"),
  oidcData: document.getElementById("oidc-data"),
  debugLog: document.getElementById("debug-log"),
  clearLogBtn: document.getElementById("clear-log-btn"),
};

// ============ Logging ============

function log(message, type = "info") {
  const time = new Date().toLocaleTimeString();
  const entry = document.createElement("div");
  entry.className = "log-entry";
  entry.innerHTML = `<span class="log-time">[${time}]</span><span class="log-${type}">${message}</span>`;
  elements.debugLog.appendChild(entry);
  elements.debugLog.scrollTop = elements.debugLog.scrollHeight;
  console.log(`[${type.toUpperCase()}]`, message);
}

// ============ PKCE Utilities ============

function generateRandomString(length = 64) {
  const charset =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
  const randomValues = new Uint8Array(length);
  crypto.getRandomValues(randomValues);
  return Array.from(randomValues)
    .map((v) => charset[v % charset.length])
    .join("");
}

async function generateCodeChallenge(verifier) {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest("SHA-256", data);

  // Base64URL encode
  const base64 = btoa(String.fromCharCode(...new Uint8Array(digest)));
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

// ============ JWT Utilities ============

function decodeJwt(token) {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) {
      throw new Error("Invalid JWT format");
    }

    const header = JSON.parse(
      atob(parts[0].replace(/-/g, "+").replace(/_/g, "/"))
    );
    const payload = JSON.parse(
      atob(parts[1].replace(/-/g, "+").replace(/_/g, "/"))
    );

    return { header, payload };
  } catch (error) {
    return { error: error.message };
  }
}

// ============ OAuth Flow ============

async function startAuthorization() {
  const authServerUrl = elements.authServerUrl.value.trim();
  const clientId = elements.clientId.value.trim();
  const redirectUri = elements.redirectUri.value.trim();
  const scopes = elements.scopes.value.trim();

  // Validate inputs
  if (!authServerUrl || !clientId || !redirectUri) {
    showStatus("Please fill in all required fields", "error");
    return;
  }

  log("Starting OAuth2 Authorization Code Flow with PKCE...", "info");

  // Generate PKCE
  codeVerifier = generateRandomString(64);
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const state = generateRandomString(32);

  // Store state for validation
  sessionStorage.setItem("oauth_state", state);
  sessionStorage.setItem("oauth_code_verifier", codeVerifier);

  log(`Generated code_verifier: ${codeVerifier.substring(0, 20)}...`, "info");
  log(`Generated code_challenge: ${codeChallenge.substring(0, 20)}...`, "info");
  log(`Generated state: ${state}`, "info");

  // Build authorization URL
  const params = new URLSearchParams({
    client_id: clientId,
    response_type: "code",
    redirect_uri: redirectUri,
    scope: scopes,
    state: state,
    code_challenge: codeChallenge,
    code_challenge_method: "S256",
  });

  // Use frontend URL for the consent page (frontend handles auth + consent UI)
  // Backend is only used for API calls (token exchange, userinfo, etc.)
  const frontendUrl =
    elements.frontendUrl?.value?.trim() || "http://localhost:5173";
  const authUrl = `${frontendUrl}/oauth/authorize?${params.toString()}`;
  log(`Authorization URL (Frontend): ${authUrl}`, "info");

  // Open popup
  const width = 500;
  const height = 600;
  const left = (window.screen.width - width) / 2;
  const top = (window.screen.height - height) / 2;

  const popup = window.open(
    authUrl,
    "oauth_popup",
    `width=${width},height=${height},left=${left},top=${top},scrollbars=yes,resizable=yes`
  );

  if (!popup) {
    showStatus(
      "Failed to open popup. Please allow popups for this site.",
      "error"
    );
    log("Popup blocked!", "error");
    return;
  }

  log("Popup opened, waiting for authorization...", "info");
  showStatus("Waiting for authorization...", "info");
  elements.loginBtn.disabled = true;

  // Listen for callback message
  const messageHandler = async (event) => {
    // Accept messages from any origin for demo purposes
    const data = event.data;

    if (typeof data === "object" && data.type === "oauth_callback") {
      window.removeEventListener("message", messageHandler);
      elements.loginBtn.disabled = false;

      if (data.error) {
        log(
          `Authorization error: ${data.error} - ${data.error_description}`,
          "error"
        );
        showStatus(`Error: ${data.error_description || data.error}`, "error");
        return;
      }

      if (data.code) {
        // Validate state
        const savedState = sessionStorage.getItem("oauth_state");
        if (data.state !== savedState) {
          log("State mismatch! Possible CSRF attack.", "error");
          showStatus("Security error: State mismatch", "error");
          return;
        }

        log(
          `Received authorization code: ${data.code.substring(0, 20)}...`,
          "success"
        );

        // Exchange code for tokens
        await exchangeCodeForTokens(data.code);
      }
    }
  };

  window.addEventListener("message", messageHandler);

  // Poll for popup close
  const pollTimer = setInterval(() => {
    if (popup.closed) {
      clearInterval(pollTimer);
      window.removeEventListener("message", messageHandler);
      elements.loginBtn.disabled = false;

      if (!accessToken) {
        log("Popup closed without completing authorization", "warn");
        showStatus("Authorization cancelled", "error");
      }
    }
  }, 500);
}

async function exchangeCodeForTokens(code) {
  const authServerUrl = elements.authServerUrl.value.trim();
  const clientId = elements.clientId.value.trim();
  const redirectUri = elements.redirectUri.value.trim();
  const savedCodeVerifier = sessionStorage.getItem("oauth_code_verifier");

  log("Exchanging authorization code for tokens...", "info");

  try {
    const response = await fetch(`${authServerUrl}/oauth/token`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        grant_type: "authorization_code",
        code: code,
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: savedCodeVerifier,
      }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(
        data.error_description || data.error || "Token exchange failed"
      );
    }

    log("Token exchange successful!", "success");

    // Store tokens
    accessToken = data.access_token;
    refreshToken = data.refresh_token;
    idToken = data.id_token;

    // Display tokens
    displayTokens(data);
    showStatus("Successfully logged in!", "success");

    // Clean up
    sessionStorage.removeItem("oauth_state");
    sessionStorage.removeItem("oauth_code_verifier");
  } catch (error) {
    log(`Token exchange error: ${error.message}`, "error");
    showStatus(`Error: ${error.message}`, "error");
  }
}

function displayTokens(data) {
  elements.tokenSection.classList.remove("hidden");
  elements.accessToken.textContent = data.access_token
    ? `${data.access_token.substring(0, 50)}...`
    : "-";
  elements.refreshToken.textContent = data.refresh_token
    ? `${data.refresh_token.substring(0, 50)}...`
    : "-";
  if (elements.idTokenDisplay) {
    elements.idTokenDisplay.textContent = data.id_token
      ? `${data.id_token.substring(0, 50)}...`
      : "-";
  }
  elements.tokenType.textContent = data.token_type || "-";
  elements.expiresIn.textContent = data.expires_in
    ? `${data.expires_in} seconds`
    : "-";
  elements.tokenScopes.textContent = data.scope || "-";

  log(`Access Token: ${data.access_token?.substring(0, 30)}...`, "info");
  if (data.refresh_token) {
    log(`Refresh Token: ${data.refresh_token?.substring(0, 30)}...`, "info");
  }
  if (data.id_token) {
    log(`ID Token: ${data.id_token?.substring(0, 30)}...`, "info");
  }
  log(`Expires in: ${data.expires_in} seconds`, "info");
}

async function refreshAccessToken() {
  if (!refreshToken) {
    showStatus("No refresh token available", "error");
    log("Cannot refresh: No refresh token available", "error");
    return;
  }

  const authServerUrl = elements.authServerUrl.value.trim();
  const clientId = elements.clientId.value.trim();

  log("Refreshing access token...", "info");

  try {
    const response = await fetch(`${authServerUrl}/oauth/token`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        grant_type: "refresh_token",
        refresh_token: refreshToken,
        client_id: clientId,
      }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(
        data.error_description || data.error || "Token refresh failed"
      );
    }

    log("Token refresh successful!", "success");

    // Update tokens
    accessToken = data.access_token;
    if (data.refresh_token) {
      refreshToken = data.refresh_token;
    }
    if (data.id_token) {
      idToken = data.id_token;
    }

    // Update display
    displayTokens(data);
    showStatus("Token refreshed successfully!", "success");
  } catch (error) {
    log(`Token refresh error: ${error.message}`, "error");
    showStatus(`Refresh error: ${error.message}`, "error");
  }
}

async function revokeToken() {
  if (!accessToken) {
    showStatus("No token to revoke", "error");
    return;
  }

  const authServerUrl = elements.authServerUrl.value.trim();
  const clientId = elements.clientId.value.trim();

  log("Revoking token...", "info");

  try {
    const response = await fetch(`${authServerUrl}/oauth/revoke`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        token: accessToken,
        client_id: clientId,
        token_type_hint: "access_token",
      }),
    });

    if (response.ok || response.status === 200) {
      log("Token revoked successfully!", "success");
      showStatus("Token revoked!", "success");

      // Clear tokens
      accessToken = null;
      refreshToken = null;
      idToken = null;

      elements.tokenSection.classList.add("hidden");
      elements.userinfoSection.classList.add("hidden");
      elements.decodedSection?.classList.add("hidden");
    } else {
      const data = await response.json();
      throw new Error(data.error_description || data.error || "Revoke failed");
    }
  } catch (error) {
    log(`Token revoke error: ${error.message}`, "error");
    showStatus(`Revoke error: ${error.message}`, "error");
  }
}

async function getUserInfo() {
  if (!accessToken) {
    showStatus("No access token available", "error");
    return;
  }

  const authServerUrl = elements.authServerUrl.value.trim();
  log("Fetching user info...", "info");

  try {
    const response = await fetch(`${authServerUrl}/oauth/userinfo`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(
        data.error_description || data.error || "Failed to get user info"
      );
    }

    log("User info retrieved successfully!", "success");
    elements.userinfoSection.classList.remove("hidden");
    elements.userinfoData.textContent = JSON.stringify(data, null, 2);
  } catch (error) {
    log(`User info error: ${error.message}`, "error");
    showStatus(`Error: ${error.message}`, "error");
  }
}

function decodeTokens() {
  if (!accessToken && !idToken) {
    showStatus("No tokens to decode", "error");
    return;
  }

  log("Decoding tokens...", "info");

  const result = {};

  if (accessToken) {
    result.access_token = decodeJwt(accessToken);
  }

  if (idToken) {
    result.id_token = decodeJwt(idToken);
  }

  if (elements.decodedSection && elements.decodedData) {
    elements.decodedSection.classList.remove("hidden");
    elements.decodedData.textContent = JSON.stringify(result, null, 2);
  }

  log("Tokens decoded successfully!", "success");
}

async function fetchOpenIdConfiguration() {
  const authServerUrl = elements.authServerUrl.value.trim();
  log("Fetching OpenID Configuration...", "info");

  try {
    const response = await fetch(
      `${authServerUrl}/.well-known/openid-configuration`
    );

    if (!response.ok) {
      throw new Error("Failed to fetch OpenID configuration");
    }

    const data = await response.json();
    log("OpenID Configuration retrieved!", "success");

    if (elements.oidcSection && elements.oidcData) {
      elements.oidcSection.classList.remove("hidden");
      elements.oidcData.textContent = JSON.stringify(data, null, 2);
    }
  } catch (error) {
    log(`OpenID Config error: ${error.message}`, "error");
  }
}

function logout() {
  accessToken = null;
  refreshToken = null;
  idToken = null;
  codeVerifier = null;

  elements.tokenSection.classList.add("hidden");
  elements.userinfoSection.classList.add("hidden");
  elements.decodedSection?.classList.add("hidden");
  elements.loginStatus.classList.add("hidden");

  sessionStorage.removeItem("oauth_state");
  sessionStorage.removeItem("oauth_code_verifier");

  log("Logged out", "info");
  showStatus("Logged out successfully", "success");
}

function showStatus(message, type) {
  elements.loginStatus.textContent = message;
  elements.loginStatus.className = `status ${type}`;
  elements.loginStatus.classList.remove("hidden");
}

function clearLog() {
  elements.debugLog.innerHTML = "";
  log("Log cleared", "info");
}

// ============ Event Listeners ============

elements.loginBtn.addEventListener("click", startAuthorization);
elements.userinfoBtn.addEventListener("click", getUserInfo);
elements.logoutBtn.addEventListener("click", logout);
elements.clearLogBtn.addEventListener("click", clearLog);

// Optional buttons (may not exist in all versions)
if (elements.refreshBtn) {
  elements.refreshBtn.addEventListener("click", refreshAccessToken);
}
if (elements.revokeBtn) {
  elements.revokeBtn.addEventListener("click", revokeToken);
}
if (elements.decodeBtn) {
  elements.decodeBtn.addEventListener("click", decodeTokens);
}

// Fetch scopes button
document
  .getElementById("fetch-scopes-btn")
  ?.addEventListener("click", async () => {
    const authServerUrl = elements.authServerUrl.value.trim();
    log("Fetching available scopes...", "info");

    try {
      const response = await fetch(`${authServerUrl}/oauth/scopes`);
      if (!response.ok) {
        throw new Error("Failed to fetch scopes");
      }
      const data = await response.json();
      const scopeCodes = data.scopes.map((s) => s.code).join(", ");
      log(`Available scopes: ${scopeCodes}`, "success");

      // Update hint
      const hint = document.querySelector(".scope-hint");
      if (hint) {
        hint.textContent = `Available: ${scopeCodes}`;
      }
    } catch (error) {
      log(`Failed to fetch scopes: ${error.message}`, "error");
    }
  });

// Fetch OpenID config button
document
  .getElementById("fetch-oidc-btn")
  ?.addEventListener("click", fetchOpenIdConfiguration);

// ============ Initialize ============

log("OAuth2 Demo App initialized", "info");
log(
  "Features: Authorization, Token Refresh, Token Revocation, User Info, JWT Decode",
  "info"
);
log(
  'Please configure your OAuth client settings and click "Login with Auth Server"',
  "info"
);

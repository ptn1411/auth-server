/**
 * React Example - Auth Server OAuth SDK
 * 
 * This is a complete example of how to use the OAuth SDK with React.
 * 
 * Installation:
 * npm install @authserver/oauth-sdk
 */

import React, { useState, useEffect, useCallback, createContext, useContext } from 'react';
import { AuthServerClient, AuthState, TokenResponse, UserInfo } from '@authserver/oauth-sdk';

// ============================================================================
// Configuration
// ============================================================================

const AUTH_CONFIG = {
  serverUrl: import.meta.env.VITE_AUTH_SERVER_URL || 'http://localhost:3000',
  clientId: import.meta.env.VITE_AUTH_CLIENT_ID || 'your-client-id',
  redirectUri: import.meta.env.VITE_AUTH_REDIRECT_URI || window.location.origin + '/callback',
  scopes: ['openid', 'profile', 'email', 'offline_access'],
};

// ============================================================================
// Auth Context
// ============================================================================

interface AuthContextType extends AuthState {
  login: () => Promise<void>;
  loginWithRedirect: () => void;
  logout: () => Promise<void>;
  getAccessToken: () => Promise<string | null>;
}

const AuthContext = createContext<AuthContextType | null>(null);

// Create auth client instance
const authClient = new AuthServerClient(AUTH_CONFIG);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AuthState>({
    isAuthenticated: false,
    isLoading: true,
    user: null,
    tokens: null,
    error: null,
  });

  // Initialize state from storage
  useEffect(() => {
    const initialState = authClient.getState();
    setState({
      ...initialState,
      isLoading: false,
    });

    // Handle redirect callback if we have a code in URL
    if (window.location.search.includes('code=')) {
      authClient.handleRedirectCallback()
        .then(() => {
          setState({
            ...authClient.getState(),
            isLoading: false,
          });
        })
        .catch((error) => {
          setState((s) => ({
            ...s,
            isLoading: false,
            error: error.message,
          }));
        });
    }
  }, []);

  const login = useCallback(async () => {
    setState((s) => ({ ...s, isLoading: true, error: null }));
    try {
      await authClient.loginWithPopup();
      setState({
        ...authClient.getState(),
        isLoading: false,
      });
    } catch (error) {
      setState((s) => ({
        ...s,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Login failed',
      }));
    }
  }, []);

  const loginWithRedirect = useCallback(() => {
    authClient.loginWithRedirect();
  }, []);

  const logout = useCallback(async () => {
    await authClient.logout({ revokeToken: true });
    setState({
      isAuthenticated: false,
      isLoading: false,
      user: null,
      tokens: null,
      error: null,
    });
  }, []);

  const getAccessToken = useCallback(async () => {
    return authClient.getAccessToken();
  }, []);

  return (
    <AuthContext.Provider
      value={{
        ...state,
        login,
        loginWithRedirect,
        logout,
        getAccessToken,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

// ============================================================================
// Components
// ============================================================================

function LoginButton() {
  const { login, loginWithRedirect, isLoading, error } = useAuth();

  return (
    <div className="login-container">
      <h2>Welcome</h2>
      <p>Please login to continue</p>
      
      <div className="button-group">
        <button 
          onClick={login} 
          disabled={isLoading}
          className="btn btn-primary"
        >
          {isLoading ? 'Loading...' : 'Login with Popup'}
        </button>
        
        <button 
          onClick={loginWithRedirect}
          className="btn btn-secondary"
        >
          Login with Redirect
        </button>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
    </div>
  );
}

function UserProfile() {
  const { user, tokens, logout, getAccessToken } = useAuth();
  const [apiResponse, setApiResponse] = useState<string | null>(null);

  const callApi = async () => {
    const token = await getAccessToken();
    if (!token) {
      setApiResponse('No token available');
      return;
    }

    // Example API call
    try {
      const response = await fetch('https://api.example.com/protected', {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      const data = await response.json();
      setApiResponse(JSON.stringify(data, null, 2));
    } catch (error) {
      setApiResponse(`Error: ${error}`);
    }
  };

  return (
    <div className="profile-container">
      <div className="user-card">
        <div className="avatar">
          {user?.picture ? (
            <img src={user.picture} alt={user.name || 'User'} />
          ) : (
            <span>{user?.name?.[0]?.toUpperCase() || '👤'}</span>
          )}
        </div>
        
        <div className="user-info">
          <h2>{user?.name || 'User'}</h2>
          <p>{user?.email}</p>
        </div>
      </div>

      <div className="actions">
        <button onClick={callApi} className="btn btn-secondary">
          Call Protected API
        </button>
        <button onClick={logout} className="btn btn-danger">
          Logout
        </button>
      </div>

      {apiResponse && (
        <pre className="api-response">{apiResponse}</pre>
      )}

      <details className="token-details">
        <summary>Token Info</summary>
        <pre>{JSON.stringify(tokens, null, 2)}</pre>
      </details>
    </div>
  );
}

function LoadingSpinner() {
  return (
    <div className="loading-container">
      <div className="spinner"></div>
      <p>Loading...</p>
    </div>
  );
}

// ============================================================================
// Main App
// ============================================================================

function AppContent() {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return <LoadingSpinner />;
  }

  return isAuthenticated ? <UserProfile /> : <LoginButton />;
}

export default function App() {
  return (
    <AuthProvider>
      <div className="app">
        <header>
          <h1>🔐 Auth Server Demo</h1>
        </header>
        <main>
          <AppContent />
        </main>
      </div>
    </AuthProvider>
  );
}

// ============================================================================
// Styles (can be moved to CSS file)
// ============================================================================

const styles = `
  .app {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    max-width: 600px;
    margin: 0 auto;
    padding: 2rem;
  }

  header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .login-container, .profile-container {
    background: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  }

  .button-group {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .btn:hover {
    opacity: 0.9;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
  }

  .btn-secondary {
    background: #6b7280;
    color: white;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
  }

  .error-message {
    background: #fef2f2;
    color: #dc2626;
    padding: 1rem;
    border-radius: 6px;
    margin-top: 1rem;
  }

  .user-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .avatar {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: #e5e7eb;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    overflow: hidden;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .user-info h2 {
    margin: 0 0 0.25rem;
  }

  .user-info p {
    margin: 0;
    color: #6b7280;
  }

  .actions {
    display: flex;
    gap: 1rem;
  }

  .api-response {
    background: #f3f4f6;
    padding: 1rem;
    border-radius: 6px;
    margin-top: 1rem;
    overflow-x: auto;
  }

  .token-details {
    margin-top: 1rem;
  }

  .token-details pre {
    background: #f3f4f6;
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
    font-size: 0.75rem;
  }

  .loading-container {
    text-align: center;
    padding: 2rem;
  }

  .spinner {
    border: 3px solid #f3f3f3;
    border-top: 3px solid #3b82f6;
    border-radius: 50%;
    width: 40px;
    height: 40px;
    animation: spin 1s linear infinite;
    margin: 0 auto 1rem;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
`;

// Inject styles
if (typeof document !== 'undefined') {
  const styleSheet = document.createElement('style');
  styleSheet.textContent = styles;
  document.head.appendChild(styleSheet);
}

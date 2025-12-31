import { create } from 'zustand';
import { authClient } from '@/lib/auth-client';
import { setRefreshToken, getRefreshToken, clearRefreshToken } from '@/lib/cookies';
import type { UserProfile, MfaRequiredResponse } from '@/lib/auth-client';

interface AuthState {
  user: UserProfile | null;
  accessToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  mfaPending: MfaRequiredResponse | null;
}

interface AuthActions {
  login: (email: string, password: string) => Promise<void>;
  loginWithPasskey: (accessToken: string, refreshToken: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  completeMfa: (code: string) => Promise<void>;
  refreshUser: () => Promise<void>;
  refreshAccessToken: () => Promise<boolean>;
  setUser: (user: UserProfile | null) => void;
  setLoading: (loading: boolean) => void;
  clearMfaPending: () => void;
  initialize: () => Promise<void>;
}

type AuthStore = AuthState & AuthActions;

export const useAuthStore = create<AuthStore>()(
  (set, get) => ({
    // State
    user: null,
    accessToken: null,
    isAuthenticated: false,
    isLoading: true,
    mfaPending: null,

    // Actions
    login: async (email: string, password: string) => {
      set({ isLoading: true });
      try {
        const response = await authClient.login({ email, password });
        if ('mfa_required' in response) {
          set({ mfaPending: response, isLoading: false });
        } else {
          // Save refresh token to cookie, access token to memory
          setRefreshToken(response.refresh_token);
          authClient.setTokens(response.access_token, response.refresh_token);
          const user = await authClient.getProfile();
          set({ 
            user, 
            accessToken: response.access_token,
            isAuthenticated: true, 
            isLoading: false, 
            mfaPending: null 
          });
        }
      } catch (error) {
        set({ isLoading: false });
        throw error;
      }
    },

    loginWithPasskey: async (accessToken: string, refreshToken: string) => {
      set({ isLoading: true });
      try {
        // Tokens are already set by the SDK during finishPasskeyAuthentication
        // Save refresh token to cookie
        setRefreshToken(refreshToken);
        authClient.setTokens(accessToken, refreshToken);
        const user = await authClient.getProfile();
        set({ 
          user, 
          accessToken: accessToken,
          isAuthenticated: true, 
          isLoading: false, 
          mfaPending: null 
        });
      } catch (error) {
        set({ isLoading: false });
        throw error;
      }
    },

    register: async (email: string, password: string) => {
      set({ isLoading: true });
      try {
        await authClient.register({ email, password });
        set({ isLoading: false });
      } catch (error) {
        set({ isLoading: false });
        throw error;
      }
    },

    logout: async () => {
      try {
        await authClient.logout();
      } catch {
        // Ignore logout errors
      } finally {
        authClient.clearTokens();
        clearRefreshToken();
        set({ user: null, accessToken: null, isAuthenticated: false, mfaPending: null });
      }
    },

    completeMfa: async (code: string) => {
      const { mfaPending } = get();
      if (!mfaPending) throw new Error('No MFA pending');
      
      set({ isLoading: true });
      try {
        const response = await authClient.completeMfaLogin({
          mfa_token: mfaPending.mfa_token,
          code,
        });
        // Save refresh token to cookie, access token to memory
        setRefreshToken(response.refresh_token);
        authClient.setTokens(response.access_token, response.refresh_token);
        const user = await authClient.getProfile();
        set({ 
          user, 
          accessToken: response.access_token,
          isAuthenticated: true, 
          mfaPending: null, 
          isLoading: false 
        });
      } catch (error) {
        set({ isLoading: false });
        throw error;
      }
    },

    refreshUser: async () => {
      try {
        const user = await authClient.getProfile();
        set({ user });
      } catch {
        // If refresh fails, try to refresh token first
        const refreshed = await get().refreshAccessToken();
        if (refreshed) {
          try {
            const user = await authClient.getProfile();
            set({ user });
          } catch {
            set({ user: null, accessToken: null, isAuthenticated: false });
            clearRefreshToken();
          }
        }
      }
    },

    refreshAccessToken: async () => {
      const refreshToken = getRefreshToken();
      if (!refreshToken) {
        set({ user: null, accessToken: null, isAuthenticated: false });
        return false;
      }

      try {
        const response = await authClient.refresh({ refresh_token: refreshToken });
        authClient.setTokens(response.access_token, refreshToken);
        set({ accessToken: response.access_token });
        return true;
      } catch {
        // Refresh failed, clear auth state
        authClient.clearTokens();
        clearRefreshToken();
        set({ user: null, accessToken: null, isAuthenticated: false });
        return false;
      }
    },

    setUser: (user: UserProfile | null) => set({ user, isAuthenticated: !!user }),
    setLoading: (isLoading: boolean) => set({ isLoading }),
    clearMfaPending: () => set({ mfaPending: null }),

    initialize: async () => {
      const refreshToken = getRefreshToken();
      
      if (!refreshToken) {
        set({ isLoading: false, isAuthenticated: false });
        return;
      }

      // Set refresh token in SDK for auto-refresh capability
      authClient.setTokens('', refreshToken);

      // Try to refresh access token using stored refresh token
      try {
        const response = await authClient.refresh({ refresh_token: refreshToken });
        authClient.setTokens(response.access_token, refreshToken);
        
        const user = await authClient.getProfile();
        set({ 
          user, 
          accessToken: response.access_token,
          isAuthenticated: true, 
          isLoading: false 
        });
      } catch {
        // Refresh failed, clear auth state
        authClient.clearTokens();
        clearRefreshToken();
        set({ user: null, accessToken: null, isAuthenticated: false, isLoading: false });
      }
    },
  })
);

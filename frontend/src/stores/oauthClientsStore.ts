import { create } from 'zustand';
import { authClient } from '@/lib/auth-client';
import type {
  OAuthClientInfo,
  OAuthClientWithSecret,
  CreateOAuthClientRequest,
  UpdateOAuthClientRequest,
  PublicScopeInfo,
} from 'auth-server-sdk';

export type {
  OAuthClientInfo as OAuthClient,
  OAuthClientWithSecret,
  CreateOAuthClientRequest,
  UpdateOAuthClientRequest,
  PublicScopeInfo as OAuthScope,
};

// Authorization request params (from URL)
export interface AuthorizationParams {
  client_id: string;
  response_type: string;
  redirect_uri: string;
  scope: string;
  state?: string;
  code_challenge?: string;
  code_challenge_method?: string;
}

// Consent response from authorize endpoint
export interface ConsentRequiredResponse {
  status: 'consent_required';
  client_id: string;
  client_name: string;
  redirect_uri: string;
  scopes: string[];
  state?: string;
  code_challenge?: string;
  code_challenge_method?: string;
}

interface OAuthClientsState {
  clients: OAuthClientInfo[];
  scopes: PublicScopeInfo[];
  isLoading: boolean;
  error: string | null;
}

interface OAuthClientsActions {
  fetchClients: () => Promise<void>;
  createClient: (data: CreateOAuthClientRequest) => Promise<OAuthClientWithSecret>;
  updateClient: (id: string, data: UpdateOAuthClientRequest) => Promise<OAuthClientInfo>;
  deleteClient: (id: string) => Promise<void>;
  regenerateSecret: (id: string) => Promise<string>;
  fetchScopes: () => Promise<void>;
  initiateAuthorization: (params: AuthorizationParams) => Promise<ConsentRequiredResponse>;
  submitConsent: (params: {
    approved: boolean;
    client_id: string;
    user_id: string;
    redirect_uri: string;
    scopes: string;
    state?: string;
    code_challenge?: string;
    code_challenge_method?: string;
  }) => Promise<string>;
  clearError: () => void;
}

type OAuthClientsStore = OAuthClientsState & OAuthClientsActions;

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

const handleApiError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred';
};

export const useOAuthClientsStore = create<OAuthClientsStore>()((set) => ({
  clients: [],
  scopes: [],
  isLoading: false,
  error: null,

  fetchClients: async () => {
    set({ isLoading: true, error: null });
    try {
      const data = await authClient.listOAuthClients();
      set({ clients: data.clients || [], isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createClient: async (data: CreateOAuthClientRequest) => {
    set({ isLoading: true, error: null });
    try {
      const client = await authClient.createOAuthClient(data);
      set((state) => ({
        clients: [...state.clients, { ...client, client_secret: undefined } as OAuthClientInfo],
        isLoading: false,
      }));
      return client;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  updateClient: async (id: string, data: UpdateOAuthClientRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updatedClient = await authClient.updateOAuthClient(id, data);
      set((state) => ({
        clients: state.clients.map((c) => (c.id === id ? updatedClient : c)),
        isLoading: false,
      }));
      return updatedClient;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deleteClient: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.deleteOAuthClient(id);
      set((state) => ({
        clients: state.clients.filter((c) => c.id !== id),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  regenerateSecret: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const data = await authClient.regenerateOAuthClientSecret(id);
      set({ isLoading: false });
      return data.client_secret;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  fetchScopes: async () => {
    set({ isLoading: true, error: null });
    try {
      const data = await authClient.listPublicScopes();
      set({ scopes: data.scopes || [], isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // These methods still use fetch directly as they involve redirects
  initiateAuthorization: async (params: AuthorizationParams) => {
    set({ isLoading: true, error: null });
    try {
      const queryParams = new URLSearchParams({
        client_id: params.client_id,
        response_type: params.response_type,
        redirect_uri: params.redirect_uri,
        scope: params.scope,
      });
      if (params.state) queryParams.set('state', params.state);
      if (params.code_challenge) queryParams.set('code_challenge', params.code_challenge);
      if (params.code_challenge_method) queryParams.set('code_challenge_method', params.code_challenge_method);

      const response = await fetch(`${API_URL}/oauth/authorize?${queryParams}`);
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error_description || errorData.message || 'Authorization failed');
      }
      const data = await response.json();
      set({ isLoading: false });
      return data;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  submitConsent: async (params) => {
    set({ isLoading: true, error: null });
    try {
      const token = authClient.getAccessToken();
      const response = await fetch(`${API_URL}/oauth/authorize/callback`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify(params),
      });
      
      const data = await response.json();
      
      if (!response.ok || data.status === 'error') {
        throw new Error(data.error_description || data.message || 'Consent submission failed');
      }
      
      set({ isLoading: false });
      return data.redirect_url || '';
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));

import { create } from 'zustand';
import { authClient } from '@/lib/auth-client';
import type { OAuthScope, CreateScopeRequest, UpdateScopeRequest } from 'auth-server-sdk';

export type { OAuthScope, CreateScopeRequest, UpdateScopeRequest };

interface AdminScopesState {
  scopes: OAuthScope[];
  total: number;
  page: number;
  limit: number;
  isLoading: boolean;
  error: string | null;
}

interface AdminScopesActions {
  fetchScopes: (page?: number, limit?: number) => Promise<void>;
  createScope: (data: CreateScopeRequest) => Promise<OAuthScope>;
  updateScope: (id: string, data: UpdateScopeRequest) => Promise<OAuthScope>;
  activateScope: (id: string) => Promise<void>;
  deactivateScope: (id: string) => Promise<void>;
  deleteScope: (id: string) => Promise<void>;
  clearError: () => void;
}

type AdminScopesStore = AdminScopesState & AdminScopesActions;

const handleApiError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred';
};

export const useAdminScopesStore = create<AdminScopesStore>()((set) => ({
  scopes: [],
  total: 0,
  page: 1,
  limit: 20,
  isLoading: false,
  error: null,

  fetchScopes: async (page = 1, limit = 20) => {
    set({ isLoading: true, error: null });
    try {
      const data = await authClient.adminListScopes({ page, limit });
      set({
        scopes: data.scopes,
        total: data.total,
        page: data.page,
        limit: data.limit,
        isLoading: false,
      });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createScope: async (data: CreateScopeRequest) => {
    set({ isLoading: true, error: null });
    try {
      const scope = await authClient.adminCreateScope(data);
      set((state) => ({
        scopes: [...state.scopes, scope],
        total: state.total + 1,
        isLoading: false,
      }));
      return scope;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  updateScope: async (id: string, data: UpdateScopeRequest) => {
    set({ isLoading: true, error: null });
    try {
      const scope = await authClient.adminUpdateScope(id, data);
      set((state) => ({
        scopes: state.scopes.map((s) => (s.id === id ? scope : s)),
        isLoading: false,
      }));
      return scope;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  activateScope: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminActivateScope(id);
      set((state) => ({
        scopes: state.scopes.map((s) =>
          s.id === id ? { ...s, is_active: true } : s
        ),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deactivateScope: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminDeactivateScope(id);
      set((state) => ({
        scopes: state.scopes.map((s) =>
          s.id === id ? { ...s, is_active: false } : s
        ),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deleteScope: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminDeleteScope(id);
      set((state) => ({
        scopes: state.scopes.filter((s) => s.id !== id),
        total: state.total - 1,
        isLoading: false,
      }));
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));

import { create } from 'zustand';
import { authClient } from '@/lib/auth-client';
import type { ConnectedApp } from '@/lib/auth-client';

interface ConnectedAppsState {
  apps: ConnectedApp[];
  isLoading: boolean;
  error: string | null;
}

interface ConnectedAppsActions {
  fetchConnectedApps: () => Promise<void>;
  revokeConsent: (clientId: string) => Promise<void>;
  clearError: () => void;
}

type ConnectedAppsStore = ConnectedAppsState & ConnectedAppsActions;

const handleApiError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred';
};

export const useConnectedAppsStore = create<ConnectedAppsStore>()((set, get) => ({
  // State
  apps: [],
  isLoading: false,
  error: null,

  // Actions
  fetchConnectedApps: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await authClient.getConnectedApps();
      set({ apps: response.apps, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  revokeConsent: async (clientId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.revokeAppConsent(clientId);
      
      // Remove app from list after successful revocation
      const { apps } = get();
      const updatedApps = apps.filter((app) => app.client_id !== clientId);
      set({ apps: updatedApps, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));

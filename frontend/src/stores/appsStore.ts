import { create } from 'zustand';
import { authClient } from '@/lib/auth-client';
import type {
  AppResponse,
  RoleResponse,
  PermissionResponse,
  AppUsersResponse,
  WebhookResponse,
  WebhookWithSecretResponse,
  ApiKeyResponse,
  ApiKeyWithSecretResponse,
  IpRuleResponse,
  CreateAppRequest,
  CreateWebhookRequest,
  UpdateWebhookRequest,
  CreateApiKeyRequest,
  UpdateApiKeyRequest,
  CreateIpRuleRequest,
  PaginationParams,
} from '@/lib/auth-client';

// Extended AppResponse with secret for creation
export interface AppWithSecret extends AppResponse {
  secret: string;
}

// App detail state containing all related data
export interface AppDetailState {
  app: AppResponse;
  roles: RoleResponse[];
  permissions: PermissionResponse[];
  users: AppUsersResponse | null;
  webhooks: WebhookResponse[];
  apiKeys: ApiKeyResponse[];
  ipRules: IpRuleResponse[];
}

interface AppsState {
  apps: AppResponse[];
  currentApp: AppDetailState | null;
  isLoading: boolean;
  error: string | null;
}

interface AppsActions {
  // App management
  fetchApps: () => Promise<void>;
  createApp: (data: CreateAppRequest) => Promise<AppWithSecret>;
  regenerateSecret: (appId: string) => Promise<string>;
  fetchAppDetail: (appId: string) => Promise<void>;
  clearCurrentApp: () => void;
  
  // Role actions
  fetchRoles: (appId: string) => Promise<void>;
  createRole: (appId: string, name: string) => Promise<void>;
  assignRole: (appId: string, userId: string, roleId: string) => Promise<void>;
  removeRole: (appId: string, userId: string, roleId: string) => Promise<void>;
  
  // Permission actions
  fetchPermissions: (appId: string) => Promise<void>;
  createPermission: (appId: string, code: string) => Promise<void>;
  
  // User actions
  fetchAppUsers: (appId: string, params?: PaginationParams) => Promise<void>;
  banUser: (appId: string, userId: string) => Promise<void>;
  unbanUser: (appId: string, userId: string) => Promise<void>;
  removeUser: (appId: string, userId: string) => Promise<void>;
  
  // Webhook actions
  fetchWebhooks: (appId: string) => Promise<void>;
  createWebhook: (appId: string, data: CreateWebhookRequest) => Promise<WebhookWithSecretResponse>;
  updateWebhook: (appId: string, webhookId: string, data: UpdateWebhookRequest) => Promise<void>;
  deleteWebhook: (appId: string, webhookId: string) => Promise<void>;
  
  // API Key actions
  fetchApiKeys: (appId: string) => Promise<void>;
  createApiKey: (appId: string, data: CreateApiKeyRequest) => Promise<ApiKeyWithSecretResponse>;
  updateApiKey: (appId: string, keyId: string, data: UpdateApiKeyRequest) => Promise<void>;
  revokeApiKey: (appId: string, keyId: string) => Promise<void>;
  deleteApiKey: (appId: string, keyId: string) => Promise<void>;
  
  // IP Rule actions
  fetchIpRules: (appId: string) => Promise<void>;
  createIpRule: (appId: string, data: CreateIpRuleRequest) => Promise<void>;
  deleteIpRule: (appId: string, ruleId: string) => Promise<void>;
  
  // Error handling
  clearError: () => void;
}

type AppsStore = AppsState & AppsActions;

const handleApiError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred';
};

export const useAppsStore = create<AppsStore>()((set, get) => ({
  // State
  apps: [],
  currentApp: null,
  isLoading: false,
  error: null,

  // App management actions
  fetchApps: async () => {
    set({ isLoading: true, error: null });
    try {
      // Note: SDK currently doesn't have a method to list user's own apps
      // This will need to be added to the SDK and backend
      // For now, we'll use admin endpoint if user is admin, otherwise empty array
      // TODO: Add GET /apps endpoint to backend and SDK
      set({ apps: [], isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createApp: async (data: CreateAppRequest) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authClient.createApp(data);
      const appWithSecret = response as AppWithSecret;
      
      // Add to apps list (without secret)
      const appWithoutSecret: AppResponse = {
        id: appWithSecret.id,
        code: appWithSecret.code,
        name: appWithSecret.name,
      };
      set((state) => ({
        apps: [...state.apps, appWithoutSecret],
        isLoading: false,
      }));
      
      return appWithSecret;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  regenerateSecret: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authClient.regenerateAppSecret(appId);
      set({ isLoading: false });
      return response.secret;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  fetchAppDetail: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      // Fetch all app-related data in parallel
      const [webhooks, apiKeys, ipRules, usersResponse] = await Promise.all([
        authClient.listWebhooks(appId),
        authClient.listApiKeys(appId),
        authClient.listAppIpRules(appId),
        authClient.getAppUsers(appId),
      ]);

      // Note: Roles and permissions require app-auth, not user JWT
      // For now, we'll set them as empty arrays
      // TODO: Add user JWT endpoints for listing roles/permissions
      
      set({
        currentApp: {
          app: { id: appId, code: '', name: '' }, // Will be populated when we have GET /apps/:id
          roles: [],
          permissions: [],
          users: usersResponse,
          webhooks,
          apiKeys,
          ipRules,
        },
        isLoading: false,
      });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  clearCurrentApp: () => {
    set({ currentApp: null });
  },

  // Role actions
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  fetchRoles: async (_appId: string) => {
    // Note: This requires app-auth, not user JWT
    // TODO: Add user JWT endpoint for listing roles
    const { currentApp } = get();
    if (currentApp) {
      set({
        currentApp: { ...currentApp, roles: [] },
      });
    }
  },

  createRole: async (appId: string, name: string) => {
    set({ isLoading: true, error: null });
    try {
      const role = await authClient.createRole(appId, { name });
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: {
            ...currentApp,
            roles: [...currentApp.roles, role],
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  assignRole: async (appId: string, userId: string, roleId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.assignRole(appId, userId, { role_id: roleId });
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  removeRole: async (appId: string, userId: string, roleId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.removeRole(appId, userId, roleId);
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // Permission actions
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  fetchPermissions: async (_appId: string) => {
    // Note: This requires app-auth, not user JWT
    // TODO: Add user JWT endpoint for listing permissions
    const { currentApp } = get();
    if (currentApp) {
      set({
        currentApp: { ...currentApp, permissions: [] },
      });
    }
  },

  createPermission: async (appId: string, code: string) => {
    set({ isLoading: true, error: null });
    try {
      const permission = await authClient.createPermission(appId, { code });
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: {
            ...currentApp,
            permissions: [...currentApp.permissions, permission],
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // User actions
  fetchAppUsers: async (appId: string, params?: PaginationParams) => {
    set({ isLoading: true, error: null });
    try {
      const users = await authClient.getAppUsers(appId, params);
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: { ...currentApp, users },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  banUser: async (appId: string, userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.banUser(appId, userId);
      // Update user in list
      const { currentApp } = get();
      if (currentApp?.users) {
        const updatedUsers = currentApp.users.users.map((user) =>
          user.id === userId ? { ...user, is_banned: true } : user
        );
        set({
          currentApp: {
            ...currentApp,
            users: { ...currentApp.users, users: updatedUsers },
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  unbanUser: async (appId: string, userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.unbanUser(appId, userId);
      // Update user in list
      const { currentApp } = get();
      if (currentApp?.users) {
        const updatedUsers = currentApp.users.users.map((user) =>
          user.id === userId ? { ...user, is_banned: false } : user
        );
        set({
          currentApp: {
            ...currentApp,
            users: { ...currentApp.users, users: updatedUsers },
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  removeUser: async (appId: string, userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.removeUserFromApp(appId, userId);
      // Remove user from list
      const { currentApp } = get();
      if (currentApp?.users) {
        const updatedUsers = currentApp.users.users.filter(
          (user) => user.id !== userId
        );
        set({
          currentApp: {
            ...currentApp,
            users: {
              ...currentApp.users,
              users: updatedUsers,
              total: currentApp.users.total - 1,
            },
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // Webhook actions
  fetchWebhooks: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      const webhooks = await authClient.listWebhooks(appId);
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: { ...currentApp, webhooks },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createWebhook: async (appId: string, data: CreateWebhookRequest) => {
    set({ isLoading: true, error: null });
    try {
      const webhook = await authClient.createWebhook(appId, data);
      const { currentApp } = get();
      if (currentApp) {
        // Add webhook without secret to list
        const webhookWithoutSecret: WebhookResponse = {
          id: webhook.id,
          app_id: webhook.app_id,
          url: webhook.url,
          events: webhook.events,
          is_active: webhook.is_active,
          created_at: webhook.created_at,
        };
        set({
          currentApp: {
            ...currentApp,
            webhooks: [...currentApp.webhooks, webhookWithoutSecret],
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
      return webhook;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  updateWebhook: async (appId: string, webhookId: string, data: UpdateWebhookRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updatedWebhook = await authClient.updateWebhook(appId, webhookId, data);
      const { currentApp } = get();
      if (currentApp) {
        const updatedWebhooks = currentApp.webhooks.map((webhook) =>
          webhook.id === webhookId ? updatedWebhook : webhook
        );
        set({
          currentApp: { ...currentApp, webhooks: updatedWebhooks },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deleteWebhook: async (appId: string, webhookId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.deleteWebhook(appId, webhookId);
      const { currentApp } = get();
      if (currentApp) {
        const updatedWebhooks = currentApp.webhooks.filter(
          (webhook) => webhook.id !== webhookId
        );
        set({
          currentApp: { ...currentApp, webhooks: updatedWebhooks },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // API Key actions
  fetchApiKeys: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      const apiKeys = await authClient.listApiKeys(appId);
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: { ...currentApp, apiKeys },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createApiKey: async (appId: string, data: CreateApiKeyRequest) => {
    set({ isLoading: true, error: null });
    try {
      const apiKey = await authClient.createApiKey(appId, data);
      const { currentApp } = get();
      if (currentApp) {
        // Add API key without full key to list
        const apiKeyWithoutKey: ApiKeyResponse = {
          id: apiKey.id,
          app_id: apiKey.app_id,
          name: apiKey.name,
          key_prefix: apiKey.key_prefix,
          scopes: apiKey.scopes,
          expires_at: apiKey.expires_at,
          last_used_at: apiKey.last_used_at,
          is_active: apiKey.is_active,
          created_at: apiKey.created_at,
        };
        set({
          currentApp: {
            ...currentApp,
            apiKeys: [...currentApp.apiKeys, apiKeyWithoutKey],
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
      return apiKey;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  updateApiKey: async (appId: string, keyId: string, data: UpdateApiKeyRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updatedKey = await authClient.updateApiKey(appId, keyId, data);
      const { currentApp } = get();
      if (currentApp) {
        const updatedKeys = currentApp.apiKeys.map((key) =>
          key.id === keyId ? updatedKey : key
        );
        set({
          currentApp: { ...currentApp, apiKeys: updatedKeys },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  revokeApiKey: async (appId: string, keyId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.revokeApiKey(appId, keyId);
      const { currentApp } = get();
      if (currentApp) {
        const updatedKeys = currentApp.apiKeys.map((key) =>
          key.id === keyId ? { ...key, is_active: false } : key
        );
        set({
          currentApp: { ...currentApp, apiKeys: updatedKeys },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deleteApiKey: async (appId: string, keyId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.deleteApiKey(appId, keyId);
      const { currentApp } = get();
      if (currentApp) {
        const updatedKeys = currentApp.apiKeys.filter((key) => key.id !== keyId);
        set({
          currentApp: { ...currentApp, apiKeys: updatedKeys },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // IP Rule actions
  fetchIpRules: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      const ipRules = await authClient.listAppIpRules(appId);
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: { ...currentApp, ipRules },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createIpRule: async (appId: string, data: CreateIpRuleRequest) => {
    set({ isLoading: true, error: null });
    try {
      const ipRule = await authClient.createAppIpRule(appId, data);
      const { currentApp } = get();
      if (currentApp) {
        set({
          currentApp: {
            ...currentApp,
            ipRules: [...currentApp.ipRules, ipRule],
          },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deleteIpRule: async (_appId: string, ruleId: string) => {
    set({ isLoading: true, error: null });
    try {
      // Note: SDK doesn't have deleteAppIpRule method yet
      // TODO: Add DELETE /apps/:app_id/ip-rules/:rule_id endpoint
      const { currentApp } = get();
      if (currentApp) {
        const updatedRules = currentApp.ipRules.filter(
          (rule) => rule.id !== ruleId
        );
        set({
          currentApp: { ...currentApp, ipRules: updatedRules },
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // Error handling
  clearError: () => {
    set({ error: null });
  },
}));

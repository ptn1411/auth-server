import { create } from 'zustand';
import { authClient } from '@/lib/auth-client';
import type {
  AdminUserDetail,
  AdminUpdateUserRequest,
  AdminAppDetail,
  AdminUpdateAppRequest,
  UserRolesInfo,
  SearchUsersParams,
  PaginatedResponse,
  PaginationParams,
  AuditLogsResponse,
  IpRuleResponse,
  CreateIpRuleRequest,
  IpCheckResponse,
} from '@/lib/auth-client';

interface AdminState {
  // User management
  users: PaginatedResponse<AdminUserDetail> | null;
  currentUser: AdminUserDetail | null;
  currentUserRoles: UserRolesInfo | null;
  
  // App management
  apps: PaginatedResponse<AdminAppDetail> | null;
  currentApp: AdminAppDetail | null;
  
  // Audit logs
  auditLogs: AuditLogsResponse | null;
  
  // IP rules
  ipRules: IpRuleResponse[];
  
  // Bulk selection
  selectedUserIds: Set<string>;
  
  // Loading and error states
  isLoading: boolean;
  error: string | null;
}

interface AdminActions {
  // User management
  fetchUsers: (params?: PaginationParams) => Promise<void>;
  searchUsers: (params: SearchUsersParams) => Promise<void>;
  fetchUser: (userId: string) => Promise<void>;
  updateUser: (userId: string, data: AdminUpdateUserRequest) => Promise<void>;
  deleteUser: (userId: string) => Promise<void>;
  deactivateUser: (userId: string) => Promise<void>;
  activateUser: (userId: string) => Promise<void>;
  unlockUser: (userId: string) => Promise<void>;
  getUserRoles: (userId: string) => Promise<UserRolesInfo>;
  clearCurrentUser: () => void;
  
  // App management
  fetchApps: (params?: PaginationParams) => Promise<void>;
  fetchApp: (appId: string) => Promise<void>;
  updateApp: (appId: string, data: AdminUpdateAppRequest) => Promise<void>;
  deleteApp: (appId: string) => Promise<void>;
  clearCurrentApp: () => void;
  
  // Audit logs
  fetchAuditLogs: (params?: PaginationParams) => Promise<void>;
  
  // IP Rules
  fetchIpRules: () => Promise<void>;
  createIpRule: (data: CreateIpRuleRequest) => Promise<void>;
  checkIp: (ip: string, appId?: string) => Promise<IpCheckResponse>;
  deleteIpRule: (ruleId: string) => Promise<void>;
  
  // Bulk operations
  exportUsers: () => Promise<AdminUserDetail[]>;
  importUsers: (users: Partial<AdminUserDetail>[]) => Promise<number>;
  bulkAssignRole: (userIds: string[], roleId: string) => Promise<number>;
  
  // Selection management
  selectUser: (userId: string) => void;
  deselectUser: (userId: string) => void;
  selectAllUsers: () => void;
  clearSelection: () => void;
  toggleUserSelection: (userId: string) => void;
  
  // Error handling
  clearError: () => void;
}

type AdminStore = AdminState & AdminActions;

const handleApiError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred';
};

export const useAdminStore = create<AdminStore>()((set, get) => ({
  // State
  users: null,
  currentUser: null,
  currentUserRoles: null,
  apps: null,
  currentApp: null,
  auditLogs: null,
  ipRules: [],
  selectedUserIds: new Set(),
  isLoading: false,
  error: null,

  // User management actions
  fetchUsers: async (params?: PaginationParams) => {
    set({ isLoading: true, error: null });
    try {
      const users = await authClient.adminListUsers(params);
      set({ users, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  searchUsers: async (params: SearchUsersParams) => {
    set({ isLoading: true, error: null });
    try {
      const users = await authClient.adminSearchUsers(params);
      set({ users, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  fetchUser: async (userId: string) => {
    set({ isLoading: true, error: null });
    try {
      const [user, roles] = await Promise.all([
        authClient.adminGetUser(userId),
        authClient.adminGetUserRoles(userId),
      ]);
      set({ currentUser: user, currentUserRoles: roles, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  updateUser: async (userId: string, data: AdminUpdateUserRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updatedUser = await authClient.adminUpdateUser(userId, data);
      
      // Update current user if it's the same
      const { currentUser, users } = get();
      if (currentUser?.id === userId) {
        set({ currentUser: updatedUser });
      }
      
      // Update user in list if present
      if (users) {
        const updatedData = users.data.map((user) =>
          user.id === userId ? updatedUser : user
        );
        set({
          users: { ...users, data: updatedData },
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

  deleteUser: async (userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminDeleteUser(userId);
      
      // Remove from list
      const { users, currentUser } = get();
      if (users) {
        const updatedData = users.data.filter((user) => user.id !== userId);
        set({
          users: { ...users, data: updatedData, total: users.total - 1 },
        });
      }
      
      // Clear current user if deleted
      if (currentUser?.id === userId) {
        set({ currentUser: null, currentUserRoles: null });
      }
      
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deactivateUser: async (userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminDeactivateUser(userId);
      
      // Update user in state
      const { currentUser, users } = get();
      if (currentUser?.id === userId) {
        set({ currentUser: { ...currentUser, is_active: false } });
      }
      
      if (users) {
        const updatedData = users.data.map((user) =>
          user.id === userId ? { ...user, is_active: false } : user
        );
        set({ users: { ...users, data: updatedData } });
      }
      
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  activateUser: async (userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminActivateUser(userId);
      
      // Update user in state
      const { currentUser, users } = get();
      if (currentUser?.id === userId) {
        set({ currentUser: { ...currentUser, is_active: true } });
      }
      
      if (users) {
        const updatedData = users.data.map((user) =>
          user.id === userId ? { ...user, is_active: true } : user
        );
        set({ users: { ...users, data: updatedData } });
      }
      
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  unlockUser: async (userId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminUnlockUser(userId);
      
      // Update user in state
      const { currentUser, users } = get();
      if (currentUser?.id === userId) {
        set({
          currentUser: {
            ...currentUser,
            failed_login_attempts: 0,
            locked_until: undefined,
          },
        });
      }
      
      if (users) {
        const updatedData = users.data.map((user) =>
          user.id === userId
            ? { ...user, failed_login_attempts: 0, locked_until: undefined }
            : user
        );
        set({ users: { ...users, data: updatedData } });
      }
      
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  getUserRoles: async (userId: string) => {
    set({ isLoading: true, error: null });
    try {
      const roles = await authClient.adminGetUserRoles(userId);
      set({ currentUserRoles: roles, isLoading: false });
      return roles;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  clearCurrentUser: () => {
    set({ currentUser: null, currentUserRoles: null });
  },

  // App management actions
  fetchApps: async (params?: PaginationParams) => {
    set({ isLoading: true, error: null });
    try {
      const apps = await authClient.adminListApps(params);
      set({ apps, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  fetchApp: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      const app = await authClient.adminGetApp(appId);
      set({ currentApp: app, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  updateApp: async (appId: string, data: AdminUpdateAppRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updatedApp = await authClient.adminUpdateApp(appId, data);
      
      // Update current app if it's the same
      const { currentApp, apps } = get();
      if (currentApp?.id === appId) {
        set({ currentApp: updatedApp });
      }
      
      // Update app in list if present
      if (apps) {
        const updatedData = apps.data.map((app) =>
          app.id === appId ? updatedApp : app
        );
        set({
          apps: { ...apps, data: updatedData },
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

  deleteApp: async (appId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminDeleteApp(appId);
      
      // Remove from list
      const { apps, currentApp } = get();
      if (apps) {
        const updatedData = apps.data.filter((app) => app.id !== appId);
        set({
          apps: { ...apps, data: updatedData, total: apps.total - 1 },
        });
      }
      
      // Clear current app if deleted
      if (currentApp?.id === appId) {
        set({ currentApp: null });
      }
      
      set({ isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  clearCurrentApp: () => {
    set({ currentApp: null });
  },

  // Audit logs actions
  fetchAuditLogs: async (params?: PaginationParams) => {
    set({ isLoading: true, error: null });
    try {
      const auditLogs = await authClient.adminGetAuditLogs(params);
      set({ auditLogs, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // IP Rules actions
  fetchIpRules: async () => {
    set({ isLoading: true, error: null });
    try {
      const ipRules = await authClient.adminListIpRules();
      set({ ipRules, isLoading: false });
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  createIpRule: async (data: CreateIpRuleRequest) => {
    set({ isLoading: true, error: null });
    try {
      const ipRule = await authClient.adminCreateIpRule(data);
      set((state) => ({
        ipRules: [...state.ipRules, ipRule],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  checkIp: async (ip: string, appId?: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await authClient.adminCheckIp(ip, appId);
      set({ isLoading: false });
      return result;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  deleteIpRule: async (ruleId: string) => {
    set({ isLoading: true, error: null });
    try {
      await authClient.adminDeleteIpRule(ruleId);
      set((state) => ({
        ipRules: state.ipRules.filter((rule) => rule.id !== ruleId),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // Bulk operations
  exportUsers: async () => {
    set({ isLoading: true, error: null });
    try {
      const users = await authClient.adminExportUsers();
      set({ isLoading: false });
      return users;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  importUsers: async (users: Partial<AdminUserDetail>[]) => {
    set({ isLoading: true, error: null });
    try {
      const result = await authClient.adminImportUsers(users);
      set({ isLoading: false });
      // Refresh user list after import
      await get().fetchUsers();
      return result.imported;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  bulkAssignRole: async (userIds: string[], roleId: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await authClient.adminBulkAssignRole(userIds, roleId);
      set({ isLoading: false });
      return result.assigned;
    } catch (error) {
      set({ error: handleApiError(error), isLoading: false });
      throw error;
    }
  },

  // Selection management
  selectUser: (userId: string) => {
    set((state) => {
      const newSelection = new Set(state.selectedUserIds);
      newSelection.add(userId);
      return { selectedUserIds: newSelection };
    });
  },

  deselectUser: (userId: string) => {
    set((state) => {
      const newSelection = new Set(state.selectedUserIds);
      newSelection.delete(userId);
      return { selectedUserIds: newSelection };
    });
  },

  selectAllUsers: () => {
    const { users } = get();
    if (users) {
      const allIds = new Set(users.data.map((user) => user.id));
      set({ selectedUserIds: allIds });
    }
  },

  clearSelection: () => {
    set({ selectedUserIds: new Set() });
  },

  toggleUserSelection: (userId: string) => {
    const { selectedUserIds } = get();
    if (selectedUserIds.has(userId)) {
      get().deselectUser(userId);
    } else {
      get().selectUser(userId);
    }
  },

  // Error handling
  clearError: () => {
    set({ error: null });
  },
}));

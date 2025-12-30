import { useEffect, useState } from 'react';
import { useAdminStore } from '@/stores/adminStore';
import {
  UserTable,
  UserSearchForm,
  BulkActionsBar,
} from '@/components/admin/users';
import type { SearchUsersParams } from '@/lib/auth-client';

export function AdminUsersPage() {
  const [page, setPage] = useState(1);
  const [limit, setLimit] = useState(10);
  const [searchParams, setSearchParams] = useState<SearchUsersParams | null>(null);

  const {
    users,
    isLoading,
    selectedUserIds,
    fetchUsers,
    searchUsers,
    clearSelection,
  } = useAdminStore();

  useEffect(() => {
    if (searchParams) {
      searchUsers({ ...searchParams, page, limit });
    } else {
      fetchUsers({ page, limit });
    }
  }, [page, limit, searchParams, fetchUsers, searchUsers]);

  const handleSearch = (params: SearchUsersParams) => {
    setSearchParams(params);
    setPage(1); // Reset to first page on new search
  };

  const handleClearSearch = () => {
    setSearchParams(null);
    setPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
  };

  const handleLimitChange = (newLimit: number) => {
    setLimit(newLimit);
    setPage(1); // Reset to first page when changing limit
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">User Management</h1>
        <p className="text-muted-foreground">
          View and manage all users in the system
        </p>
      </div>

      <UserSearchForm
        onSearch={handleSearch}
        onClear={handleClearSearch}
        isLoading={isLoading}
      />

      <BulkActionsBar
        selectedCount={selectedUserIds.size}
        onClearSelection={clearSelection}
      />

      <UserTable
        users={users}
        isLoading={isLoading}
        onPageChange={handlePageChange}
        onLimitChange={handleLimitChange}
      />
    </div>
  );
}

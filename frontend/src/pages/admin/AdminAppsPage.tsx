import { useEffect, useState } from 'react';
import { useAdminStore } from '@/stores/adminStore';
import { AdminAppTable } from '@/components/admin/apps';

export function AdminAppsPage() {
  const [page, setPage] = useState(1);
  const [limit, setLimit] = useState(10);

  const {
    apps,
    isLoading,
    fetchApps,
  } = useAdminStore();

  useEffect(() => {
    fetchApps({ page, limit });
  }, [page, limit, fetchApps]);

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
        <h1 className="text-2xl font-bold tracking-tight">Application Management</h1>
        <p className="text-muted-foreground">
          View and manage all applications in the system
        </p>
      </div>

      <AdminAppTable
        apps={apps}
        isLoading={isLoading}
        onPageChange={handlePageChange}
        onLimitChange={handleLimitChange}
      />
    </div>
  );
}

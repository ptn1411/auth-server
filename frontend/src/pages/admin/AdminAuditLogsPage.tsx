import { useEffect, useState } from 'react';
import { useAdminStore } from '@/stores/adminStore';
import {
  AuditLogTable,
  AuditLogFilters,
  type AuditLogFilterParams,
} from '@/components/admin/audit';
import type { PaginationParams } from '@/lib/auth-client';

export function AdminAuditLogsPage() {
  const [page, setPage] = useState(1);
  const [limit, setLimit] = useState(10);
  const [filterParams, setFilterParams] = useState<AuditLogFilterParams | null>(null);

  const {
    auditLogs,
    isLoading,
    fetchAuditLogs,
  } = useAdminStore();

  useEffect(() => {
    const params: PaginationParams = { page, limit };
    
    // Add filter params if present
    if (filterParams?.user_id) {
      params.user_id = filterParams.user_id;
    }
    if (filterParams?.action) {
      params.action = filterParams.action;
    }
    
    fetchAuditLogs(params);
  }, [page, limit, filterParams, fetchAuditLogs]);

  const handleFilter = (params: AuditLogFilterParams) => {
    setFilterParams(params);
    setPage(1); // Reset to first page on new filter
  };

  const handleClearFilter = () => {
    setFilterParams(null);
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
        <h1 className="text-2xl font-bold tracking-tight">Audit Logs</h1>
        <p className="text-muted-foreground">
          View system-wide activity history and security events
        </p>
      </div>

      <AuditLogFilters
        onFilter={handleFilter}
        onClear={handleClearFilter}
        isLoading={isLoading}
      />

      <AuditLogTable
        auditLogs={auditLogs}
        isLoading={isLoading}
        onPageChange={handlePageChange}
        onLimitChange={handleLimitChange}
      />
    </div>
  );
}

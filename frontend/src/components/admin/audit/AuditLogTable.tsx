import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Pagination } from '@/components/shared/Pagination';
import {
  FileText,
  Globe,
  Clock,
  Monitor,
  Loader2,
  User,
} from 'lucide-react';
import type { AuditLogsResponse } from '@/lib/auth-client';

interface AuditLogTableProps {
  auditLogs: AuditLogsResponse | null;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
  onLimitChange?: (limit: number) => void;
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString();
};

const formatAction = (action: string) => {
  return action
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
};

const getActionBadgeVariant = (action: string): 'default' | 'secondary' | 'destructive' | 'outline' => {
  const actionLower = action.toLowerCase();
  if (actionLower.includes('login') || actionLower.includes('register')) {
    return 'default';
  }
  if (actionLower.includes('logout') || actionLower.includes('revoke')) {
    return 'secondary';
  }
  if (actionLower.includes('delete') || actionLower.includes('fail')) {
    return 'destructive';
  }
  return 'outline';
};

export function AuditLogTable({
  auditLogs,
  isLoading = false,
  onPageChange,
  onLimitChange,
}: AuditLogTableProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5" />
          Audit Logs
        </CardTitle>
        <CardDescription>
          System-wide activity history and security events
        </CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : !auditLogs || auditLogs.logs.length === 0 ? (
          <div className="text-center py-8">
            <FileText className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No audit logs found</h3>
            <p className="text-muted-foreground">
              Try adjusting your filters or check back later.
            </p>
          </div>
        ) : (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>User</TableHead>
                  <TableHead>Action</TableHead>
                  <TableHead>IP Address</TableHead>
                  <TableHead>User Agent</TableHead>
                  <TableHead>Timestamp</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {auditLogs.logs.map((log) => (
                  <TableRow key={log.id}>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <User className="h-4 w-4 text-muted-foreground" />
                        <span className="font-mono text-xs truncate max-w-[120px]" title={log.id}>
                          {log.id.substring(0, 8)}...
                        </span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant={getActionBadgeVariant(log.action)}>
                        {formatAction(log.action)}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Globe className="h-4 w-4 text-muted-foreground" />
                        <span>{log.ip_address || 'Unknown'}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Monitor className="h-4 w-4 text-muted-foreground" />
                        <span 
                          className="truncate max-w-[200px]" 
                          title={log.user_agent || 'Unknown'}
                        >
                          {log.user_agent || 'Unknown'}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Clock className="h-4 w-4 text-muted-foreground" />
                        <span>{formatDate(log.created_at)}</span>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            <Pagination
              page={auditLogs.page}
              limit={auditLogs.limit}
              total={auditLogs.total}
              onPageChange={onPageChange}
              onLimitChange={onLimitChange}
              showPageSize
            />
          </>
        )}
      </CardContent>
    </Card>
  );
}

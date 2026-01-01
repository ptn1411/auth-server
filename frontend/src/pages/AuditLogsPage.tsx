import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { authClient, type AuditLog } from '@/lib/auth-client';
import { toast } from 'sonner';
import { 
  FileText, 
  Globe, 
  Clock, 
  Monitor, 
  Loader2,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';

export function AuditLogsPage() {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const limit = 10;

  const fetchAuditLogs = async (pageNum: number) => {
    try {
      setIsLoading(true);
      const response = await authClient.getAuditLogs({ page: pageNum, limit });
      setLogs(response.logs);
      setTotal(response.total);
      setPage(response.page);
    } catch (error) {
      toast.error('Failed to load audit logs');
      console.error('Failed to fetch audit logs:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchAuditLogs(1);
  }, []);

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const formatAction = (action: string) => {
    return action
      .split('_')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
      .join(' ');
  };

  const totalPages = Math.ceil(total / limit);

  const handlePreviousPage = () => {
    if (page > 1) {
      fetchAuditLogs(page - 1);
    }
  };

  const handleNextPage = () => {
    if (page < totalPages) {
      fetchAuditLogs(page + 1);
    }
  };

  // Mobile card view for audit log
  const MobileLogCard = ({ log }: { log: AuditLog }) => (
    <div className="border rounded-lg p-4 space-y-2">
      <div className="font-medium text-sm">{formatAction(log.action)}</div>
      <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
        <div className="flex items-center gap-1">
          <Globe className="h-3 w-3" />
          <span className="truncate">{log.ip_address || 'Unknown'}</span>
        </div>
        <div className="flex items-center gap-1">
          <Clock className="h-3 w-3" />
          <span className="truncate">{formatDate(log.created_at)}</span>
        </div>
      </div>
      <div className="flex items-center gap-1 text-xs text-muted-foreground">
        <Monitor className="h-3 w-3 flex-shrink-0" />
        <span className="truncate">{log.user_agent || 'Unknown'}</span>
      </div>
    </div>
  );

  return (
    <div className="space-y-4 sm:space-y-6">
      <div>
        <h1 className="text-2xl sm:text-3xl font-bold">Audit Logs</h1>
        <p className="text-sm sm:text-base text-muted-foreground">
          View your account activity history
        </p>
      </div>

      <Card>
        <CardHeader className="pb-3 sm:pb-6">
          <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
            <FileText className="h-4 w-4 sm:h-5 sm:w-5" />
            Activity History
          </CardTitle>
          <CardDescription className="text-xs sm:text-sm">
            A record of all actions performed on your account
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : logs.length === 0 ? (
            <p className="text-center text-muted-foreground py-8">
              No audit logs found
            </p>
          ) : (
            <>
              {/* Mobile view - Card list */}
              <div className="space-y-3 md:hidden">
                {logs.map((log) => (
                  <MobileLogCard key={log.id} log={log} />
                ))}
              </div>

              {/* Desktop view - Table */}
              <div className="hidden md:block">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Action</TableHead>
                      <TableHead>IP Address</TableHead>
                      <TableHead>User Agent</TableHead>
                      <TableHead>Timestamp</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {logs.map((log) => (
                      <TableRow key={log.id}>
                        <TableCell>
                          <div className="font-medium">
                            {formatAction(log.action)}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <Globe className="h-4 w-4 text-muted-foreground" />
                            {log.ip_address || 'Unknown'}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <Monitor className="h-4 w-4 text-muted-foreground" />
                            <span className="truncate max-w-[200px]" title={log.user_agent || 'Unknown'}>
                              {log.user_agent || 'Unknown'}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <Clock className="h-4 w-4 text-muted-foreground" />
                            {formatDate(log.created_at)}
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              {/* Pagination */}
              <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mt-4 pt-4 border-t">
                <p className="text-xs sm:text-sm text-muted-foreground text-center sm:text-left">
                  Showing {((page - 1) * limit) + 1} to {Math.min(page * limit, total)} of {total} entries
                </p>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handlePreviousPage}
                    disabled={page <= 1 || isLoading}
                  >
                    <ChevronLeft className="h-4 w-4" />
                    <span className="hidden sm:inline ml-1">Previous</span>
                  </Button>
                  <span className="text-xs sm:text-sm text-muted-foreground px-2">
                    {page} / {totalPages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleNextPage}
                    disabled={page >= totalPages || isLoading}
                  >
                    <span className="hidden sm:inline mr-1">Next</span>
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

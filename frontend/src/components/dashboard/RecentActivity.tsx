import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { authClient, type AuditLog } from '@/lib/auth-client';
import { Activity, Loader2 } from 'lucide-react';

export function RecentActivity() {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchLogs = async () => {
      try {
        setIsLoading(true);
        const response = await authClient.getAuditLogs({ limit: 5 });
        setLogs(response.logs);
        setError(null);
      } catch {
        setError('Failed to load recent activity');
      } finally {
        setIsLoading(false);
      }
    };

    fetchLogs();
  }, []);

  const formatAction = (action: string) => {
    return action
      .replace(/_/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase());
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString();
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          Recent Activity
        </CardTitle>
        <CardDescription>Your latest account activity</CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <p className="text-sm text-destructive py-4">{error}</p>
        ) : logs.length === 0 ? (
          <p className="text-sm text-muted-foreground py-4">No recent activity</p>
        ) : (
          <div className="space-y-4">
            {logs.map((log) => (
              <div
                key={log.id}
                className="flex items-start justify-between border-b pb-3 last:border-0 last:pb-0"
              >
                <div className="space-y-1">
                  <p className="text-sm font-medium">{formatAction(log.action)}</p>
                  <p className="text-xs text-muted-foreground">
                    {log.ip_address && `IP: ${log.ip_address}`}
                  </p>
                </div>
                <p className="text-xs text-muted-foreground">
                  {formatDate(log.created_at)}
                </p>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { authClient, type Session } from '@/lib/auth-client';
import { toast } from 'sonner';
import { Monitor, Smartphone, Globe, Clock, Trash2, LogOut, Loader2 } from 'lucide-react';

export function SessionList() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [revokeDialogOpen, setRevokeDialogOpen] = useState(false);
  const [revokeAllDialogOpen, setRevokeAllDialogOpen] = useState(false);
  const [sessionToRevoke, setSessionToRevoke] = useState<Session | null>(null);
  const [isRevoking, setIsRevoking] = useState(false);

  const fetchSessions = async () => {
    try {
      setIsLoading(true);
      const response = await authClient.getSessions();
      setSessions(response.sessions);
    } catch (error) {
      toast.error('Failed to load sessions');
      console.error('Failed to fetch sessions:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSessions();
  }, []);

  const parseUserAgent = (userAgent?: string) => {
    if (!userAgent) return { device: 'Unknown', browser: 'Unknown' };
    
    let device = 'Desktop';
    let browser = 'Unknown';

    // Detect device
    if (/mobile/i.test(userAgent)) {
      device = 'Mobile';
    } else if (/tablet/i.test(userAgent)) {
      device = 'Tablet';
    }

    // Detect browser
    if (/firefox/i.test(userAgent)) {
      browser = 'Firefox';
    } else if (/edg/i.test(userAgent)) {
      browser = 'Edge';
    } else if (/chrome/i.test(userAgent)) {
      browser = 'Chrome';
    } else if (/safari/i.test(userAgent)) {
      browser = 'Safari';
    } else if (/opera|opr/i.test(userAgent)) {
      browser = 'Opera';
    }

    return { device, browser };
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const handleRevokeSession = async () => {
    if (!sessionToRevoke) return;

    setIsRevoking(true);
    try {
      await authClient.revokeSession({ session_id: sessionToRevoke.id });
      toast.success('Session revoked successfully');
      setSessions(sessions.filter(s => s.id !== sessionToRevoke.id));
      setRevokeDialogOpen(false);
      setSessionToRevoke(null);
    } catch (error) {
      toast.error('Failed to revoke session');
      console.error('Failed to revoke session:', error);
    } finally {
      setIsRevoking(false);
    }
  };

  const handleRevokeAllOtherSessions = async () => {
    setIsRevoking(true);
    try {
      await authClient.revokeOtherSessions();
      toast.success('All other sessions revoked successfully');
      // Keep only the current session (first one is usually current)
      await fetchSessions();
      setRevokeAllDialogOpen(false);
    } catch (error) {
      toast.error('Failed to revoke sessions');
      console.error('Failed to revoke all sessions:', error);
    } finally {
      setIsRevoking(false);
    }
  };

  const openRevokeDialog = (session: Session) => {
    setSessionToRevoke(session);
    setRevokeDialogOpen(true);
  };

  const DeviceIcon = ({ userAgent }: { userAgent?: string }) => {
    const { device } = parseUserAgent(userAgent);
    if (device === 'Mobile') {
      return <Smartphone className="h-4 w-4" />;
    }
    return <Monitor className="h-4 w-4" />;
  };

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Monitor className="h-5 w-5" />
            Active Sessions
          </CardTitle>
          <CardDescription>Manage your active sessions</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Monitor className="h-5 w-5" />
                Active Sessions
              </CardTitle>
              <CardDescription>
                Manage your active sessions across devices
              </CardDescription>
            </div>
            {sessions.length > 1 && (
              <Button
                variant="destructive"
                size="sm"
                onClick={() => setRevokeAllDialogOpen(true)}
              >
                <LogOut className="h-4 w-4 mr-2" />
                Revoke All Other
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {sessions.length === 0 ? (
            <p className="text-center text-muted-foreground py-4">
              No active sessions found
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Device</TableHead>
                  <TableHead>IP Address</TableHead>
                  <TableHead>Last Activity</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sessions.map((session, index) => {
                  const { device, browser } = parseUserAgent(session.user_agent);
                  const isCurrentSession = index === 0;
                  
                  return (
                    <TableRow key={session.id}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <DeviceIcon userAgent={session.user_agent} />
                          <div>
                            <div className="font-medium flex items-center gap-2">
                              {device} - {browser}
                              {isCurrentSession && (
                                <Badge variant="secondary" className="text-xs">
                                  Current
                                </Badge>
                              )}
                            </div>
                            <div className="text-xs text-muted-foreground truncate max-w-[200px]">
                              {session.user_agent || 'Unknown'}
                            </div>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Globe className="h-4 w-4 text-muted-foreground" />
                          {session.ip_address || 'Unknown'}
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Clock className="h-4 w-4 text-muted-foreground" />
                          {formatDate(session.last_used_at)}
                        </div>
                      </TableCell>
                      <TableCell>
                        {formatDate(session.created_at)}
                      </TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          onClick={() => openRevokeDialog(session)}
                          disabled={isCurrentSession}
                          title={isCurrentSession ? 'Cannot revoke current session' : 'Revoke session'}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Revoke Single Session Dialog */}
      <Dialog open={revokeDialogOpen} onOpenChange={setRevokeDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Revoke Session</DialogTitle>
            <DialogDescription>
              Are you sure you want to revoke this session? The device will be logged out immediately.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setRevokeDialogOpen(false)}
              disabled={isRevoking}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleRevokeSession}
              disabled={isRevoking}
            >
              {isRevoking ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Revoking...
                </>
              ) : (
                'Revoke Session'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Revoke All Other Sessions Dialog */}
      <Dialog open={revokeAllDialogOpen} onOpenChange={setRevokeAllDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Revoke All Other Sessions</DialogTitle>
            <DialogDescription>
              Are you sure you want to revoke all other sessions? All devices except this one will be logged out immediately.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setRevokeAllDialogOpen(false)}
              disabled={isRevoking}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleRevokeAllOtherSessions}
              disabled={isRevoking}
            >
              {isRevoking ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Revoking...
                </>
              ) : (
                'Revoke All Other Sessions'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardAction } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Unlink, Calendar, Shield } from 'lucide-react';
import type { ConnectedApp } from '@/lib/auth-client';

interface ConnectedAppCardProps {
  app: ConnectedApp;
  onRevoke: (app: ConnectedApp) => void;
  isRevoking?: boolean;
}

export function ConnectedAppCard({ app, onRevoke, isRevoking }: ConnectedAppCardProps) {
  const authorizedDate = new Date(app.authorized_at).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });

  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-muted-foreground" />
              {app.client_name}
            </CardTitle>
            <CardDescription className="flex items-center gap-1 text-xs">
              <Calendar className="h-3 w-3" />
              Authorized on {authorizedDate}
            </CardDescription>
          </div>
          <CardAction>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onRevoke(app)}
              disabled={isRevoking}
              className="text-destructive hover:text-destructive"
            >
              <Unlink className="h-4 w-4 mr-1" />
              {isRevoking ? 'Revoking...' : 'Revoke'}
            </Button>
          </CardAction>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          <p className="text-sm text-muted-foreground">Granted permissions:</p>
          <div className="flex flex-wrap gap-1">
            {app.scopes.length > 0 ? (
              app.scopes.map((scope) => (
                <Badge key={scope} variant="secondary" className="text-xs">
                  {scope}
                </Badge>
              ))
            ) : (
              <span className="text-xs text-muted-foreground">No specific scopes</span>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardAction } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { MoreVertical, Key, Settings, ExternalLink } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import type { AppResponse } from '@/lib/auth-client';

interface AppCardProps {
  app: AppResponse;
  onViewDetails: (app: AppResponse) => void;
  onRegenerateSecret: (app: AppResponse) => void;
}

export function AppCard({ app, onViewDetails, onRegenerateSecret }: AppCardProps) {
  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2">
              {app.name}
              <Badge variant="secondary" className="text-xs font-mono">
                {app.code}
              </Badge>
            </CardTitle>
            <CardDescription className="font-mono text-xs">
              ID: {app.id}
            </CardDescription>
          </div>
          <CardAction>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon-sm">
                  <MoreVertical className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => onViewDetails(app)}>
                  <Settings className="h-4 w-4 mr-2" />
                  View Details
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => onRegenerateSecret(app)}>
                  <Key className="h-4 w-4 mr-2" />
                  Regenerate Secret
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </CardAction>
        </div>
      </CardHeader>
      <CardContent>
        <Button
          variant="outline"
          className="w-full"
          onClick={() => onViewDetails(app)}
        >
          <ExternalLink className="h-4 w-4 mr-2" />
          Manage App
        </Button>
      </CardContent>
    </Card>
  );
}

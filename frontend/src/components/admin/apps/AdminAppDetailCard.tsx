import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  AppWindow,
  Calendar,
  Code,
  Key,
  User,
  Edit,
  Trash2,
  Loader2,
} from 'lucide-react';
import type { AdminAppDetail } from '@/lib/auth-client';

interface AdminAppDetailCardProps {
  app: AdminAppDetail;
  isLoading?: boolean;
  onEdit: () => void;
  onDelete: () => void;
}

export function AdminAppDetailCard({
  app,
  isLoading = false,
  onEdit,
  onDelete,
}: AdminAppDetailCardProps) {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <AppWindow className="h-5 w-5" />
              Application Details
            </CardTitle>
            <CardDescription>
              View and manage application information
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={onEdit}>
              <Edit className="h-4 w-4 mr-2" />
              Edit
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Basic Info */}
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <AppWindow className="h-4 w-4" />
              Name
            </div>
            <p className="font-medium">{app.name}</p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Code className="h-4 w-4" />
              Code
            </div>
            <code className="text-sm bg-muted px-2 py-1 rounded font-medium">
              {app.code}
            </code>
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              Owner ID
            </div>
            <p className="font-mono text-sm">{app.owner_id}</p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Key className="h-4 w-4" />
              Secret Status
            </div>
            {app.has_secret ? (
              <Badge variant="outline" className="flex items-center gap-1 w-fit">
                <Key className="h-3 w-3" />
                Configured
              </Badge>
            ) : (
              <Badge variant="secondary" className="w-fit">Not set</Badge>
            )}
          </div>
        </div>

        {/* Timestamps */}
        <div className="space-y-2">
          <h4 className="text-sm font-medium text-muted-foreground">Timestamps</h4>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Calendar className="h-4 w-4" />
                Created
              </div>
              <p className="font-medium">{formatDate(app.created_at)}</p>
            </div>
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Calendar className="h-4 w-4" />
                Last Updated
              </div>
              <p className="font-medium">{formatDate(app.updated_at)}</p>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex flex-wrap gap-2 pt-4 border-t">
          <Button variant="destructive" onClick={onDelete}>
            <Trash2 className="h-4 w-4 mr-2" />
            Delete Application
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Shield, Package, Loader2 } from 'lucide-react';
import type { UserRolesInfo } from '@/lib/auth-client';

interface UserRolesCardProps {
  roles: UserRolesInfo | null;
  isLoading?: boolean;
}

export function UserRolesCard({ roles, isLoading = false }: UserRolesCardProps) {
  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  const hasRoles = roles && roles.apps.length > 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          Roles Across Apps
        </CardTitle>
        <CardDescription>
          User's roles in different applications
        </CardDescription>
      </CardHeader>
      <CardContent>
        {!hasRoles ? (
          <div className="text-center py-8">
            <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No roles assigned</h3>
            <p className="text-muted-foreground">
              This user doesn't have any roles in any application.
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {roles.apps.map((app) => (
              <div key={app.app_id} className="border rounded-lg p-4">
                <div className="flex items-center gap-2 mb-3">
                  <Package className="h-4 w-4 text-muted-foreground" />
                  <span className="font-medium">{app.app_name}</span>
                  <span className="text-sm text-muted-foreground">({app.app_code})</span>
                </div>
                <div className="flex flex-wrap gap-2">
                  {app.roles.length > 0 ? (
                    app.roles.map((role) => (
                      <Badge key={role.id} variant="secondary">
                        {role.name}
                      </Badge>
                    ))
                  ) : (
                    <span className="text-sm text-muted-foreground">No roles in this app</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

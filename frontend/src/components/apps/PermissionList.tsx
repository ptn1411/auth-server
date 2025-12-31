import { useState } from 'react';
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
import { CreatePermissionDialog } from './CreatePermissionDialog';
import { Plus, Lock, Loader2 } from 'lucide-react';
import type { PermissionResponse } from '@/lib/auth-client';

interface PermissionListProps {
  appId: string;
  permissions?: PermissionResponse[];
  isLoading?: boolean;
  onPermissionCreated?: () => void;
}

export function PermissionList({
  appId,
  permissions = [],
  isLoading = false,
  onPermissionCreated,
}: PermissionListProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);

  const handlePermissionCreated = () => {
    setCreateDialogOpen(false);
    onPermissionCreated?.();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Lock className="h-5 w-5" />
                Permissions
              </CardTitle>
              <CardDescription>
                Manage permissions for this application
              </CardDescription>
            </div>
            <Button onClick={() => setCreateDialogOpen(true)} size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create Permission
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : permissions.length === 0 ? (
            <div className="text-center py-8">
              <Lock className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No permissions yet</h3>
              <p className="text-muted-foreground mb-4">
                Create permissions to define granular access controls.
              </p>
              <Button onClick={() => setCreateDialogOpen(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create First Permission
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Code</TableHead>
                  <TableHead>ID</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {permissions.map((permission) => (
                  <TableRow key={permission.id}>
                    <TableCell>
                      <Badge variant="outline" className="font-mono">
                        {permission.code}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground">
                      {permission.id}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <CreatePermissionDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        appId={appId}
        onPermissionCreated={handlePermissionCreated}
      />
    </>
  );
}

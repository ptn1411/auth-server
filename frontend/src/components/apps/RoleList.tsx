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
import { CreateRoleDialog } from './CreateRoleDialog';
import { RolePermissionsDialog } from './RolePermissionsDialog';
import { Plus, Shield, Loader2, Settings } from 'lucide-react';
import type { RoleResponse, PermissionResponse } from '@/lib/auth-client';

interface RoleListProps {
  appId: string;
  roles?: RoleResponse[];
  permissions?: PermissionResponse[];
  isLoading?: boolean;
  onRoleCreated?: () => void;
  onPermissionsChanged?: () => void;
}

export function RoleList({ 
  appId, 
  roles = [], 
  permissions = [],
  isLoading = false, 
  onRoleCreated,
  onPermissionsChanged,
}: RoleListProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [permissionsDialogOpen, setPermissionsDialogOpen] = useState(false);
  const [selectedRole, setSelectedRole] = useState<RoleResponse | null>(null);

  const handleRoleCreated = () => {
    setCreateDialogOpen(false);
    onRoleCreated?.();
  };

  const handleManagePermissions = (role: RoleResponse) => {
    setSelectedRole(role);
    setPermissionsDialogOpen(true);
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Roles
              </CardTitle>
              <CardDescription>
                Manage roles and their permissions for this application
              </CardDescription>
            </div>
            <Button onClick={() => setCreateDialogOpen(true)} size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create Role
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : roles.length === 0 ? (
            <div className="text-center py-8">
              <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No roles yet</h3>
              <p className="text-muted-foreground mb-4">
                Create roles to manage user permissions within this app.
              </p>
              <Button onClick={() => setCreateDialogOpen(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create First Role
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>ID</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {roles.map((role) => (
                  <TableRow key={role.id}>
                    <TableCell>
                      <Badge variant="secondary">{role.name}</Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground">
                      {role.id}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleManagePermissions(role)}
                      >
                        <Settings className="h-4 w-4 mr-2" />
                        Permissions
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <CreateRoleDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        appId={appId}
        onRoleCreated={handleRoleCreated}
      />

      <RolePermissionsDialog
        open={permissionsDialogOpen}
        onOpenChange={setPermissionsDialogOpen}
        appId={appId}
        role={selectedRole}
        allPermissions={permissions}
        onPermissionsChanged={onPermissionsChanged}
      />
    </>
  );
}

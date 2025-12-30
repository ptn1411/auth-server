import { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Loader2, Plus, X } from 'lucide-react';
import type { AppUser, RoleResponse } from '@/lib/auth-client';

interface UserRoleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appId: string;
  user: AppUser;
  roles: RoleResponse[];
  userRoles?: RoleResponse[];
}

export function UserRoleDialog({
  open,
  onOpenChange,
  appId,
  user,
  roles,
  userRoles = [],
}: UserRoleDialogProps) {
  const [assignedRoles, setAssignedRoles] = useState<Set<string>>(
    new Set(userRoles.map((r) => r.id))
  );
  const [isProcessing, setIsProcessing] = useState(false);
  const [processingRoleId, setProcessingRoleId] = useState<string | null>(null);

  const { assignRole, removeRole } = useAppsStore();

  const handleAssignRole = async (roleId: string) => {
    setIsProcessing(true);
    setProcessingRoleId(roleId);
    try {
      await assignRole(appId, user.id, roleId);
      setAssignedRoles((prev) => new Set([...prev, roleId]));
      toast.success('Role assigned successfully');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to assign role');
    } finally {
      setIsProcessing(false);
      setProcessingRoleId(null);
    }
  };

  const handleRemoveRole = async (roleId: string) => {
    setIsProcessing(true);
    setProcessingRoleId(roleId);
    try {
      await removeRole(appId, user.id, roleId);
      setAssignedRoles((prev) => {
        const newSet = new Set(prev);
        newSet.delete(roleId);
        return newSet;
      });
      toast.success('Role removed successfully');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to remove role');
    } finally {
      setIsProcessing(false);
      setProcessingRoleId(null);
    }
  };

  const assignedRolesList = roles.filter((r) => assignedRoles.has(r.id));
  const availableRoles = roles.filter((r) => !assignedRoles.has(r.id));

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Manage User Roles</DialogTitle>
          <DialogDescription>
            Assign or remove roles for {user.email}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Assigned Roles */}
          <div>
            <h4 className="text-sm font-medium mb-2">Assigned Roles</h4>
            {assignedRolesList.length === 0 ? (
              <p className="text-sm text-muted-foreground">No roles assigned</p>
            ) : (
              <div className="flex flex-wrap gap-2">
                {assignedRolesList.map((role) => (
                  <Badge
                    key={role.id}
                    variant="secondary"
                    className="flex items-center gap-1"
                  >
                    {role.name}
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      className="h-4 w-4 p-0 hover:bg-transparent"
                      onClick={() => handleRemoveRole(role.id)}
                      disabled={isProcessing}
                    >
                      {processingRoleId === role.id ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : (
                        <X className="h-3 w-3" />
                      )}
                    </Button>
                  </Badge>
                ))}
              </div>
            )}
          </div>

          {/* Available Roles */}
          {availableRoles.length > 0 && (
            <div>
              <h4 className="text-sm font-medium mb-2">Available Roles</h4>
              <div className="flex flex-wrap gap-2">
                {availableRoles.map((role) => (
                  <Button
                    key={role.id}
                    variant="outline"
                    size="sm"
                    onClick={() => handleAssignRole(role.id)}
                    disabled={isProcessing}
                  >
                    {processingRoleId === role.id ? (
                      <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                    ) : (
                      <Plus className="h-4 w-4 mr-1" />
                    )}
                    {role.name}
                  </Button>
                ))}
              </div>
            </div>
          )}

          {roles.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-4">
              No roles defined for this app. Create roles first to assign them to users.
            </p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { authClient, type RoleResponse, type PermissionResponse } from '@/lib/auth-client';
import { toast } from 'sonner';
import { Loader2, Lock, Shield, X, Plus } from 'lucide-react';

interface RolePermissionsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appId: string;
  role: RoleResponse | null;
  allPermissions: PermissionResponse[];
  onPermissionsChanged?: () => void;
}

export function RolePermissionsDialog({
  open,
  onOpenChange,
  appId,
  role,
  allPermissions,
  onPermissionsChanged,
}: RolePermissionsDialogProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [assignedPermissions, setAssignedPermissions] = useState<PermissionResponse[]>([]);
  const [selectedPermissionId, setSelectedPermissionId] = useState<string>('');

  useEffect(() => {
    if (open && role) {
      fetchRolePermissions();
      setSelectedPermissionId('');
    }
  }, [open, role]);

  const fetchRolePermissions = async () => {
    if (!role) return;
    setIsLoading(true);
    try {
      const permissions = await authClient.getRolePermissions(appId, role.id);
      setAssignedPermissions(permissions);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to load role permissions');
  
    } finally {
      setIsLoading(false);
    }
  };

  // Get available permissions (not yet assigned)
  const availablePermissions = allPermissions.filter(
    p => !assignedPermissions.some(ap => ap.id === p.id)
  );

  const handleAddPermission = async () => {
    if (!role || !selectedPermissionId) return;
    
    setIsSaving(true);
    try {
      await authClient.assignPermissionToRole(appId, role.id, { permission_id: selectedPermissionId });
      
      // Add to local state
      const addedPermission = allPermissions.find(p => p.id === selectedPermissionId);
      if (addedPermission) {
        setAssignedPermissions(prev => [...prev, addedPermission]);
      }
      
      setSelectedPermissionId('');
      toast.success('Permission added');
      onPermissionsChanged?.();
    } catch (error) {
       toast.error(error instanceof Error ? error.message : 'Failed to add permission');
    
    } finally {
      setIsSaving(false);
    }
  };

  const handleRemovePermission = async (permissionId: string) => {
    if (!role) return;
    
    setIsSaving(true);
    try {
      await authClient.removePermissionFromRole(appId, role.id, permissionId);
      
      // Remove from local state
      setAssignedPermissions(prev => prev.filter(p => p.id !== permissionId));
      
      toast.success('Permission removed');
      onPermissionsChanged?.();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to remove permission');
     
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Manage Permissions
          </DialogTitle>
          <DialogDescription>
            {role && (
              <>
                Assign permissions to role{' '}
                <Badge variant="secondary">{role.name}</Badge>
              </>
            )}
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="space-y-4">
            {/* Add Permission Section */}
            <div className="flex gap-2">
              <Select
                value={selectedPermissionId}
                onValueChange={setSelectedPermissionId}
                disabled={isSaving || availablePermissions.length === 0}
              >
                <SelectTrigger className="flex-1">
                  <SelectValue placeholder={
                    availablePermissions.length === 0 
                      ? "No more permissions available" 
                      : "Select permission to add..."
                  } />
                </SelectTrigger>
                <SelectContent>
                  {availablePermissions.map((permission) => (
                    <SelectItem key={permission.id} value={permission.id}>
                      <span className="font-mono">{permission.code}</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                onClick={handleAddPermission}
                disabled={!selectedPermissionId || isSaving}
                size="icon"
              >
                {isSaving ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Plus className="h-4 w-4" />
                )}
              </Button>
            </div>

            {/* Assigned Permissions */}
            <div>
              <label className="text-sm font-medium text-muted-foreground mb-2 block">
                Assigned Permissions ({assignedPermissions.length})
              </label>
              
              {assignedPermissions.length === 0 ? (
                <div className="text-center py-6 border rounded-lg border-dashed">
                  <Lock className="h-8 w-8 mx-auto text-muted-foreground mb-2" />
                  <p className="text-sm text-muted-foreground">
                    No permissions assigned yet
                  </p>
                </div>
              ) : (
                <div className="flex flex-wrap gap-2 p-3 border rounded-lg min-h-[80px]">
                  {assignedPermissions.map((permission) => (
                    <Badge
                      key={permission.id}
                      variant="secondary"
                      className="flex items-center gap-1 pr-1 font-mono"
                    >
                      {permission.code}
                      <button
                        onClick={() => handleRemovePermission(permission.id)}
                        disabled={isSaving}
                        className="ml-1 hover:bg-destructive hover:text-destructive-foreground rounded-full p-0.5 transition-colors"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        <div className="flex justify-end pt-4">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

import { useEffect, useState } from 'react';
import { useParams, useNavigate, useSearchParams } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import {
  UserDetailCard,
  EditUserDialog,
  UserRolesCard,
} from '@/components/admin/users';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import { ArrowLeft, Loader2 } from 'lucide-react';

export function AdminUserDetailPage() {
  const { userId } = useParams<{ userId: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  
  const [editDialogOpen, setEditDialogOpen] = useState(searchParams.get('edit') === 'true');
  const [deactivateDialogOpen, setDeactivateDialogOpen] = useState(false);
  const [activateDialogOpen, setActivateDialogOpen] = useState(false);
  const [unlockDialogOpen, setUnlockDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const {
    currentUser,
    currentUserRoles,
    isLoading,
    fetchUser,
    deactivateUser,
    activateUser,
    unlockUser,
    deleteUser,
    clearCurrentUser,
  } = useAdminStore();

  useEffect(() => {
    if (userId) {
      fetchUser(userId);
    }
    
    return () => {
      clearCurrentUser();
    };
  }, [userId, fetchUser, clearCurrentUser]);

  // Handle edit query param
  useEffect(() => {
    if (searchParams.get('edit') === 'true' && currentUser) {
      setEditDialogOpen(true);
      // Remove the query param after opening dialog
      setSearchParams({});
    }
  }, [searchParams, currentUser, setSearchParams]);

  const handleDeactivate = async () => {
    if (!currentUser) return;
    setIsProcessing(true);
    try {
      await deactivateUser(currentUser.id);
      toast.success(`User ${currentUser.email} has been deactivated`);
      setDeactivateDialogOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to deactivate user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleActivate = async () => {
    if (!currentUser) return;
    setIsProcessing(true);
    try {
      await activateUser(currentUser.id);
      toast.success(`User ${currentUser.email} has been activated`);
      setActivateDialogOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to activate user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleUnlock = async () => {
    if (!currentUser) return;
    setIsProcessing(true);
    try {
      await unlockUser(currentUser.id);
      toast.success(`User ${currentUser.email} has been unlocked`);
      setUnlockDialogOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to unlock user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleDelete = async () => {
    if (!currentUser) return;
    setIsProcessing(true);
    try {
      await deleteUser(currentUser.id);
      toast.success(`User ${currentUser.email} has been deleted`);
      setDeleteDialogOpen(false);
      navigate('/admin/users');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete user');
    } finally {
      setIsProcessing(false);
    }
  };

  if (isLoading && !currentUser) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!currentUser) {
    return (
      <div className="text-center py-12">
        <h2 className="text-lg font-medium mb-2">User not found</h2>
        <p className="text-muted-foreground mb-4">
          The user you're looking for doesn't exist or has been deleted.
        </p>
        <Button onClick={() => navigate('/admin/users')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Users
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="sm" onClick={() => navigate('/admin/users')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back
        </Button>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{currentUser.email}</h1>
          <p className="text-muted-foreground">User ID: {currentUser.id}</p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <UserDetailCard
          user={currentUser}
          isLoading={isLoading}
          onEdit={() => setEditDialogOpen(true)}
          onActivate={() => setActivateDialogOpen(true)}
          onDeactivate={() => setDeactivateDialogOpen(true)}
          onUnlock={() => setUnlockDialogOpen(true)}
          onDelete={() => setDeleteDialogOpen(true)}
        />

        <UserRolesCard
          roles={currentUserRoles}
          isLoading={isLoading}
        />
      </div>

      {/* Edit User Dialog */}
      <EditUserDialog
        open={editDialogOpen}
        onOpenChange={setEditDialogOpen}
        user={currentUser}
        onSuccess={() => {
          if (userId) {
            fetchUser(userId);
          }
        }}
      />

      {/* Deactivate User Dialog */}
      <ConfirmDialog
        open={deactivateDialogOpen}
        onOpenChange={setDeactivateDialogOpen}
        title="Deactivate User"
        description={`Are you sure you want to deactivate ${currentUser.email}? They will no longer be able to log in.`}
        confirmText="Deactivate"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleDeactivate}
      />

      {/* Activate User Dialog */}
      <ConfirmDialog
        open={activateDialogOpen}
        onOpenChange={setActivateDialogOpen}
        title="Activate User"
        description={`Are you sure you want to activate ${currentUser.email}? They will be able to log in again.`}
        confirmText="Activate"
        isLoading={isProcessing}
        onConfirm={handleActivate}
      />

      {/* Unlock User Dialog */}
      <ConfirmDialog
        open={unlockDialogOpen}
        onOpenChange={setUnlockDialogOpen}
        title="Unlock User"
        description={`Are you sure you want to unlock ${currentUser.email}? Their failed login attempts will be reset.`}
        confirmText="Unlock"
        isLoading={isProcessing}
        onConfirm={handleUnlock}
      />

      {/* Delete User Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete User"
        description={`Are you sure you want to permanently delete ${currentUser.email}? This action cannot be undone and all user data will be lost.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleDelete}
      />
    </div>
  );
}

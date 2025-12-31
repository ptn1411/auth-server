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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Pagination } from '@/components/shared/Pagination';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { UserRoleDialog } from './UserRoleDialog';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Users, MoreVertical, Shield, Ban, UserMinus, Loader2 } from 'lucide-react';
import type { AppUser, AppUsersResponse, RoleResponse } from '@/lib/auth-client';

interface AppUserListProps {
  appId: string;
  users: AppUsersResponse | null;
  roles?: RoleResponse[];
  isLoading?: boolean;
  onPageChange: (page: number) => void;
}

export function AppUserList({
  appId,
  users,
  roles = [],
  isLoading = false,
  onPageChange,
}: AppUserListProps) {
  const [selectedUser, setSelectedUser] = useState<AppUser | null>(null);
  const [roleDialogOpen, setRoleDialogOpen] = useState(false);
  const [banDialogOpen, setBanDialogOpen] = useState(false);
  const [unbanDialogOpen, setUnbanDialogOpen] = useState(false);
  const [removeDialogOpen, setRemoveDialogOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const { banUser, unbanUser, removeUser } = useAppsStore();

  const handleBanUser = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await banUser(appId, selectedUser.id);
      toast.success(`User ${selectedUser.email} has been banned`);
      setBanDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to ban user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleUnbanUser = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await unbanUser(appId, selectedUser.id);
      toast.success(`User ${selectedUser.email} has been unbanned`);
      setUnbanDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to unban user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleRemoveUser = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await removeUser(appId, selectedUser.id);
      toast.success(`User ${selectedUser.email} has been removed from the app`);
      setRemoveDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to remove user');
    } finally {
      setIsProcessing(false);
    }
  };

  const openRoleDialog = (user: AppUser) => {
    setSelectedUser(user);
    setRoleDialogOpen(true);
  };

  const openBanDialog = (user: AppUser) => {
    setSelectedUser(user);
    setBanDialogOpen(true);
  };

  const openUnbanDialog = (user: AppUser) => {
    setSelectedUser(user);
    setUnbanDialogOpen(true);
  };

  const openRemoveDialog = (user: AppUser) => {
    setSelectedUser(user);
    setRemoveDialogOpen(true);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            App Users
          </CardTitle>
          <CardDescription>
            Users registered to this application
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : !users || !users.users || users.users.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No users yet</h3>
              <p className="text-muted-foreground">
                Users will appear here once they register with this app.
              </p>
            </div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Email</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Joined</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {users.users.map((user) => (
                    <TableRow key={user.id}>
                      <TableCell className="font-medium">{user.email}</TableCell>
                      <TableCell>
                        {user.is_banned ? (
                          <Badge variant="destructive">Banned</Badge>
                        ) : (
                          <Badge variant="secondary">Active</Badge>
                        )}
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {formatDate(user.joined_at)}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon-sm">
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => openRoleDialog(user)}>
                              <Shield className="h-4 w-4 mr-2" />
                              Manage Roles
                            </DropdownMenuItem>
                            {user.is_banned ? (
                              <DropdownMenuItem onClick={() => openUnbanDialog(user)}>
                                <Ban className="h-4 w-4 mr-2" />
                                Unban User
                              </DropdownMenuItem>
                            ) : (
                              <DropdownMenuItem
                                onClick={() => openBanDialog(user)}
                                className="text-destructive"
                              >
                                <Ban className="h-4 w-4 mr-2" />
                                Ban User
                              </DropdownMenuItem>
                            )}
                            <DropdownMenuItem
                              onClick={() => openRemoveDialog(user)}
                              className="text-destructive"
                            >
                              <UserMinus className="h-4 w-4 mr-2" />
                              Remove from App
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              <Pagination
                page={users.page}
                limit={users.limit}
                total={users.total}
                onPageChange={onPageChange}
              />
            </>
          )}
        </CardContent>
      </Card>

      {/* Role Management Dialog */}
      {selectedUser && (
        <UserRoleDialog
          open={roleDialogOpen}
          onOpenChange={setRoleDialogOpen}
          appId={appId}
          user={selectedUser}
          roles={roles}
        />
      )}

      {/* Ban User Dialog */}
      <ConfirmDialog
        open={banDialogOpen}
        onOpenChange={setBanDialogOpen}
        title="Ban User"
        description={`Are you sure you want to ban ${selectedUser?.email}? They will no longer be able to access this app.`}
        confirmText="Ban User"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleBanUser}
      />

      {/* Unban User Dialog */}
      <ConfirmDialog
        open={unbanDialogOpen}
        onOpenChange={setUnbanDialogOpen}
        title="Unban User"
        description={`Are you sure you want to unban ${selectedUser?.email}? They will be able to access this app again.`}
        confirmText="Unban User"
        isLoading={isProcessing}
        onConfirm={handleUnbanUser}
      />

      {/* Remove User Dialog */}
      <ConfirmDialog
        open={removeDialogOpen}
        onOpenChange={setRemoveDialogOpen}
        title="Remove User"
        description={`Are you sure you want to remove ${selectedUser?.email} from this app? This action cannot be undone.`}
        confirmText="Remove User"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleRemoveUser}
      />
    </>
  );
}

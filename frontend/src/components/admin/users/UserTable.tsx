import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
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
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Pagination } from '@/components/shared/Pagination';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import {
  Users,
  MoreVertical,
  Eye,
  Edit,
  UserX,
  UserCheck,
  Unlock,
  Trash2,
  Loader2,
  Shield,
  Lock,
} from 'lucide-react';
import type { AdminUserDetail, PaginatedResponse } from '@/lib/auth-client';

interface UserTableProps {
  users: PaginatedResponse<AdminUserDetail> | null;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
  onLimitChange?: (limit: number) => void;
}

export function UserTable({
  users,
  isLoading = false,
  onPageChange,
  onLimitChange,
}: UserTableProps) {
  const navigate = useNavigate();
  const [selectedUser, setSelectedUser] = useState<AdminUserDetail | null>(null);
  const [deactivateDialogOpen, setDeactivateDialogOpen] = useState(false);
  const [activateDialogOpen, setActivateDialogOpen] = useState(false);
  const [unlockDialogOpen, setUnlockDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const {
    selectedUserIds,
    toggleUserSelection,
    selectAllUsers,
    clearSelection,
    deactivateUser,
    activateUser,
    unlockUser,
    deleteUser,
  } = useAdminStore();

  const handleDeactivate = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await deactivateUser(selectedUser.id);
      toast.success(`User ${selectedUser.email} has been deactivated`);
      setDeactivateDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to deactivate user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleActivate = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await activateUser(selectedUser.id);
      toast.success(`User ${selectedUser.email} has been activated`);
      setActivateDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to activate user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleUnlock = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await unlockUser(selectedUser.id);
      toast.success(`User ${selectedUser.email} has been unlocked`);
      setUnlockDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to unlock user');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedUser) return;
    setIsProcessing(true);
    try {
      await deleteUser(selectedUser.id);
      toast.success(`User ${selectedUser.email} has been deleted`);
      setDeleteDialogOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete user');
    } finally {
      setIsProcessing(false);
    }
  };

  const isUserLocked = (user: AdminUserDetail) => {
    return user.locked_until && new Date(user.locked_until) > new Date();
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const allSelected = users?.data.length ? users.data.every((u) => selectedUserIds.has(u.id)) : false;
  const someSelected = users?.data.some((u) => selectedUserIds.has(u.id)) && !allSelected;

  const handleSelectAll = () => {
    if (allSelected) {
      clearSelection();
    } else {
      selectAllUsers();
    }
  };

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Users
          </CardTitle>
          <CardDescription>
            Manage all users in the system
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : !users || users.data.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No users found</h3>
              <p className="text-muted-foreground">
                Try adjusting your search filters.
              </p>
            </div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-12">
                      <Checkbox
                        checked={allSelected}
                        ref={(el) => {
                          if (el) {
                            (el as HTMLButtonElement & { indeterminate: boolean }).indeterminate = someSelected ?? false;
                          }
                        }}
                        onCheckedChange={handleSelectAll}
                        aria-label="Select all users"
                      />
                    </TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Role</TableHead>
                    <TableHead>MFA</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {users.data.map((user) => (
                    <TableRow
                      key={user.id}
                      data-state={selectedUserIds.has(user.id) ? 'selected' : undefined}
                    >
                      <TableCell>
                        <Checkbox
                          checked={selectedUserIds.has(user.id)}
                          onCheckedChange={() => toggleUserSelection(user.id)}
                          aria-label={`Select ${user.email}`}
                        />
                      </TableCell>
                      <TableCell className="font-medium">
                        <div className="flex flex-col">
                          <span>{user.email}</span>
                          {!user.email_verified && (
                            <span className="text-xs text-muted-foreground">Unverified</span>
                          )}
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-col gap-1">
                          {!user.is_active ? (
                            <Badge variant="secondary">Inactive</Badge>
                          ) : isUserLocked(user) ? (
                            <Badge variant="destructive" className="flex items-center gap-1 w-fit">
                              <Lock className="h-3 w-3" />
                              Locked
                            </Badge>
                          ) : (
                            <Badge variant="default">Active</Badge>
                          )}
                        </div>
                      </TableCell>
                      <TableCell>
                        {user.is_system_admin ? (
                          <Badge variant="outline" className="flex items-center gap-1 w-fit">
                            <Shield className="h-3 w-3" />
                            Admin
                          </Badge>
                        ) : (
                          <span className="text-muted-foreground">User</span>
                        )}
                      </TableCell>
                      <TableCell>
                        {user.mfa_enabled ? (
                          <Badge variant="outline">Enabled</Badge>
                        ) : (
                          <span className="text-muted-foreground">Disabled</span>
                        )}
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {formatDate(user.created_at)}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon-sm">
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => navigate(`/admin/users/${user.id}`)}>
                              <Eye className="h-4 w-4 mr-2" />
                              View Details
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => navigate(`/admin/users/${user.id}?edit=true`)}>
                              <Edit className="h-4 w-4 mr-2" />
                              Edit User
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            {user.is_active ? (
                              <DropdownMenuItem
                                onClick={() => {
                                  setSelectedUser(user);
                                  setDeactivateDialogOpen(true);
                                }}
                              >
                                <UserX className="h-4 w-4 mr-2" />
                                Deactivate
                              </DropdownMenuItem>
                            ) : (
                              <DropdownMenuItem
                                onClick={() => {
                                  setSelectedUser(user);
                                  setActivateDialogOpen(true);
                                }}
                              >
                                <UserCheck className="h-4 w-4 mr-2" />
                                Activate
                              </DropdownMenuItem>
                            )}
                            {isUserLocked(user) && (
                              <DropdownMenuItem
                                onClick={() => {
                                  setSelectedUser(user);
                                  setUnlockDialogOpen(true);
                                }}
                              >
                                <Unlock className="h-4 w-4 mr-2" />
                                Unlock Account
                              </DropdownMenuItem>
                            )}
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedUser(user);
                                setDeleteDialogOpen(true);
                              }}
                              className="text-destructive"
                            >
                              <Trash2 className="h-4 w-4 mr-2" />
                              Delete User
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
                onLimitChange={onLimitChange}
                showPageSize
              />
            </>
          )}
        </CardContent>
      </Card>

      {/* Deactivate User Dialog */}
      <ConfirmDialog
        open={deactivateDialogOpen}
        onOpenChange={setDeactivateDialogOpen}
        title="Deactivate User"
        description={`Are you sure you want to deactivate ${selectedUser?.email}? They will no longer be able to log in.`}
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
        description={`Are you sure you want to activate ${selectedUser?.email}? They will be able to log in again.`}
        confirmText="Activate"
        isLoading={isProcessing}
        onConfirm={handleActivate}
      />

      {/* Unlock User Dialog */}
      <ConfirmDialog
        open={unlockDialogOpen}
        onOpenChange={setUnlockDialogOpen}
        title="Unlock User"
        description={`Are you sure you want to unlock ${selectedUser?.email}? Their failed login attempts will be reset.`}
        confirmText="Unlock"
        isLoading={isProcessing}
        onConfirm={handleUnlock}
      />

      {/* Delete User Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete User"
        description={`Are you sure you want to permanently delete ${selectedUser?.email}? This action cannot be undone and all user data will be lost.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleDelete}
      />
    </>
  );
}

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
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
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Pagination } from '@/components/shared/Pagination';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import {
  AppWindow,
  MoreVertical,
  Eye,
  Edit,
  Trash2,
  Loader2,
  Key,
} from 'lucide-react';
import type { AdminAppDetail, PaginatedResponse } from '@/lib/auth-client';

interface AdminAppTableProps {
  apps: PaginatedResponse<AdminAppDetail> | null;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
  onLimitChange?: (limit: number) => void;
}

export function AdminAppTable({
  apps,
  isLoading = false,
  onPageChange,
  onLimitChange,
}: AdminAppTableProps) {
  const navigate = useNavigate();
  const [selectedApp, setSelectedApp] = useState<AdminAppDetail | null>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const { deleteApp } = useAdminStore();

  const handleDelete = async () => {
    if (!selectedApp) return;
    setIsProcessing(true);
    try {
      await deleteApp(selectedApp.id);
      toast.success(`App "${selectedApp.name}" has been deleted`);
      setDeleteDialogOpen(false);
      setSelectedApp(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete app');
    } finally {
      setIsProcessing(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AppWindow className="h-5 w-5" />
            Applications
          </CardTitle>
          <CardDescription>
            Manage all applications in the system
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : !apps || apps.data.length === 0 ? (
            <div className="text-center py-8">
              <AppWindow className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No applications found</h3>
              <p className="text-muted-foreground">
                There are no applications registered in the system.
              </p>
            </div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Code</TableHead>
                    <TableHead>Secret</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {apps.data.map((app) => (
                    <TableRow key={app.id}>
                      <TableCell className="font-medium">
                        {app.name}
                      </TableCell>
                      <TableCell>
                        <code className="text-sm bg-muted px-2 py-1 rounded">
                          {app.code}
                        </code>
                      </TableCell>
                      <TableCell>
                        {app.has_secret ? (
                          <Badge variant="outline" className="flex items-center gap-1 w-fit">
                            <Key className="h-3 w-3" />
                            Configured
                          </Badge>
                        ) : (
                          <span className="text-muted-foreground">Not set</span>
                        )}
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {formatDate(app.created_at)}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon-sm">
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => navigate(`/admin/apps/${app.id}`)}>
                              <Eye className="h-4 w-4 mr-2" />
                              View Details
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => navigate(`/admin/apps/${app.id}?edit=true`)}>
                              <Edit className="h-4 w-4 mr-2" />
                              Edit App
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedApp(app);
                                setDeleteDialogOpen(true);
                              }}
                              className="text-destructive"
                            >
                              <Trash2 className="h-4 w-4 mr-2" />
                              Delete App
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              <Pagination
                page={apps.page}
                limit={apps.limit}
                total={apps.total}
                onPageChange={onPageChange}
                onLimitChange={onLimitChange}
                showPageSize
              />
            </>
          )}
        </CardContent>
      </Card>

      {/* Delete App Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete Application"
        description={`Are you sure you want to permanently delete "${selectedApp?.name}"? This action cannot be undone and all app data including roles, permissions, webhooks, and API keys will be lost.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleDelete}
      />
    </>
  );
}

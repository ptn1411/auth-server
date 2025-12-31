import { useEffect, useState } from 'react';
import { useAdminScopesStore, type OAuthScope } from '@/stores/adminScopesStore';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { toast } from 'sonner';
import { Plus, Pencil, Trash2, Power, PowerOff, Loader2, Key } from 'lucide-react';

export function AdminScopesPage() {
  const {
    scopes,
    total,
    isLoading,
    fetchScopes,
    createScope,
    updateScope,
    activateScope,
    deactivateScope,
    deleteScope,
  } = useAdminScopesStore();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedScope, setSelectedScope] = useState<OAuthScope | null>(null);
  const [formData, setFormData] = useState({ code: '', description: '' });
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    fetchScopes();
  }, [fetchScopes]);

  const handleCreate = async () => {
    if (!formData.code.trim() || !formData.description.trim()) {
      toast.error('Please fill in all fields');
      return;
    }

    setIsSubmitting(true);
    try {
      await createScope(formData);
      toast.success('Scope created successfully');
      setIsCreateDialogOpen(false);
      setFormData({ code: '', description: '' });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create scope');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleUpdate = async () => {
    if (!selectedScope || !formData.description.trim()) {
      toast.error('Please fill in all fields');
      return;
    }

    setIsSubmitting(true);
    try {
      await updateScope(selectedScope.id, { description: formData.description });
      toast.success('Scope updated successfully');
      setIsEditDialogOpen(false);
      setSelectedScope(null);
      setFormData({ code: '', description: '' });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to update scope');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedScope) return;

    setIsSubmitting(true);
    try {
      await deleteScope(selectedScope.id);
      toast.success('Scope deleted successfully');
      setIsDeleteDialogOpen(false);
      setSelectedScope(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete scope');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleToggleActive = async (scope: OAuthScope) => {
    try {
      if (scope.is_active) {
        await deactivateScope(scope.id);
        toast.success('Scope deactivated');
      } else {
        await activateScope(scope.id);
        toast.success('Scope activated');
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to toggle scope status');
    }
  };

  const openEditDialog = (scope: OAuthScope) => {
    setSelectedScope(scope);
    setFormData({ code: scope.code, description: scope.description });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (scope: OAuthScope) => {
    setSelectedScope(scope);
    setIsDeleteDialogOpen(true);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">OAuth Scopes</h1>
          <p className="text-muted-foreground">
            Manage OAuth scopes for authorization
          </p>
        </div>
        <Button onClick={() => setIsCreateDialogOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Scope
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            Scopes ({total})
          </CardTitle>
          <CardDescription>
            OAuth scopes define what permissions an application can request
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading && scopes.length === 0 ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : scopes.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No scopes found. Create your first scope to get started.
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Code</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {scopes.map((scope) => (
                  <TableRow key={scope.id}>
                    <TableCell className="font-mono font-medium">
                      {scope.code}
                    </TableCell>
                    <TableCell className="max-w-md truncate">
                      {scope.description}
                    </TableCell>
                    <TableCell>
                      <Badge variant={scope.is_active ? 'default' : 'secondary'}>
                        {scope.is_active ? 'Active' : 'Inactive'}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {new Date(scope.created_at).toLocaleDateString()}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleToggleActive(scope)}
                          title={scope.is_active ? 'Deactivate' : 'Activate'}
                        >
                          {scope.is_active ? (
                            <PowerOff className="h-4 w-4" />
                          ) : (
                            <Power className="h-4 w-4" />
                          )}
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => openEditDialog(scope)}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => openDeleteDialog(scope)}
                          className="text-destructive hover:text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Create Dialog */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create New Scope</DialogTitle>
            <DialogDescription>
              Add a new OAuth scope for authorization
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="code">Scope Code</Label>
              <Input
                id="code"
                placeholder="e.g., profile.read"
                value={formData.code}
                onChange={(e) => setFormData({ ...formData, code: e.target.value })}
              />
              <p className="text-xs text-muted-foreground">
                Use lowercase with dots for namespacing (e.g., profile.read, email.write)
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Input
                id="description"
                placeholder="e.g., Read user profile information"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              />
              <p className="text-xs text-muted-foreground">
                This will be shown to users on the consent screen
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={isSubmitting}>
              {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit Scope</DialogTitle>
            <DialogDescription>
              Update the scope description
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>Scope Code</Label>
              <Input value={formData.code} disabled className="bg-muted" />
              <p className="text-xs text-muted-foreground">
                Scope code cannot be changed
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-description">Description</Label>
              <Input
                id="edit-description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsEditDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleUpdate} disabled={isSubmitting}>
              {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Save Changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <Dialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Scope</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete the scope "{selectedScope?.code}"? 
              This action cannot be undone and may affect existing OAuth clients using this scope.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDeleteDialogOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={isSubmitting}
            >
              {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

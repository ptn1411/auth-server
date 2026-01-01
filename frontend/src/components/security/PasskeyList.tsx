import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
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
import { useWebAuthn } from '@/hooks/useWebAuthn';
import type { PasskeyResponse } from '@/lib/auth-client';
import { toast } from 'sonner';
import { Key, Trash2, Pencil, Loader2, Plus, AlertCircle } from 'lucide-react';
import { PasskeyRegister } from './PasskeyRegister';

export function PasskeyList() {
  const { isSupported, listPasskeys, deletePasskey, renamePasskey } = useWebAuthn();
  const [passkeys, setPasskeys] = useState<PasskeyResponse[]>([]);
  const [isFetching, setIsFetching] = useState(() => isSupported);
  const [showRegisterDialog, setShowRegisterDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [showRenameDialog, setShowRenameDialog] = useState(false);
  const [selectedPasskey, setSelectedPasskey] = useState<PasskeyResponse | null>(null);
  const [newName, setNewName] = useState('');
  const [isDeleting, setIsDeleting] = useState(false);
  const [isRenaming, setIsRenaming] = useState(false);

  useEffect(() => {
    if (!isSupported) return;

    let cancelled = false;
    
    const fetchPasskeys = async () => {
      const result = await listPasskeys();
      if (!cancelled) {
        setPasskeys(result);
        setIsFetching(false);
      }
    };

    fetchPasskeys();

    return () => {
      cancelled = true;
    };
  }, [isSupported, listPasskeys]);

  const handleDelete = async () => {
    if (!selectedPasskey) return;

    setIsDeleting(true);
    const success = await deletePasskey(selectedPasskey.id);
    if (success) {
      toast.success('Passkey deleted successfully');
      setPasskeys(passkeys.filter(p => p.id !== selectedPasskey.id));
      setShowDeleteDialog(false);
      setSelectedPasskey(null);
    } else {
      toast.error('Failed to delete passkey');
    }
    setIsDeleting(false);
  };

  const handleRename = async () => {
    if (!selectedPasskey || !newName.trim()) return;

    setIsRenaming(true);
    const success = await renamePasskey(selectedPasskey.id, newName.trim());
    if (success) {
      toast.success('Passkey renamed successfully');
      setPasskeys(passkeys.map(p => 
        p.id === selectedPasskey.id 
          ? { ...p, device_name: newName.trim() } 
          : p
      ));
      setShowRenameDialog(false);
      setSelectedPasskey(null);
      setNewName('');
    } else {
      toast.error('Failed to rename passkey');
    }
    setIsRenaming(false);
  };

  const openDeleteDialog = (passkey: PasskeyResponse) => {
    setSelectedPasskey(passkey);
    setShowDeleteDialog(true);
  };

  const openRenameDialog = (passkey: PasskeyResponse) => {
    setSelectedPasskey(passkey);
    setNewName(passkey.device_name || '');
    setShowRenameDialog(true);
  };

  const handleRegisterSuccess = (passkey: PasskeyResponse) => {
    setPasskeys([...passkeys, passkey]);
    setShowRegisterDialog(false);
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'Never';
    return new Date(dateString).toLocaleString();
  };

  // Mobile card view for passkey
  const MobilePasskeyCard = ({ passkey }: { passkey: PasskeyResponse }) => (
    <div className="border rounded-lg p-4 space-y-3">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          <Key className="h-4 w-4 text-muted-foreground" />
          <span className="font-medium text-sm">
            {passkey.device_name || 'Unnamed Passkey'}
          </span>
        </div>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={() => openRenameDialog(passkey)}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={() => openDeleteDialog(passkey)}
          >
            <Trash2 className="h-4 w-4 text-destructive" />
          </Button>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
        <div>
          <span className="block text-muted-foreground/70">Created</span>
          <span>{formatDate(passkey.created_at)}</span>
        </div>
        <div>
          <span className="block text-muted-foreground/70">Last used</span>
          <span>{formatDate(passkey.last_used_at)}</span>
        </div>
      </div>
    </div>
  );

  if (!isSupported) {
    return (
      <Card>
        <CardHeader className="pb-3 sm:pb-6">
          <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
            <Key className="h-4 w-4 sm:h-5 sm:w-5" />
            Passkeys
          </CardTitle>
          <CardDescription className="text-xs sm:text-sm">
            Passwordless authentication using biometrics or security keys
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-start gap-2 text-amber-600 dark:text-amber-500">
            <AlertCircle className="h-5 w-5 shrink-0 mt-0.5" />
            <p className="text-sm">WebAuthn is not supported in this browser. Please use a modern browser to manage passkeys.</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
      <Card>
        <CardHeader className="pb-3 sm:pb-6">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div>
              <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
                <Key className="h-4 w-4 sm:h-5 sm:w-5" />
                Passkeys
              </CardTitle>
              <CardDescription className="text-xs sm:text-sm">
                Passwordless authentication using biometrics or security keys
              </CardDescription>
            </div>
            <Button onClick={() => setShowRegisterDialog(true)} className="w-full sm:w-auto">
              <Plus className="h-4 w-4 mr-2" />
              Add Passkey
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isFetching ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : passkeys.length === 0 ? (
            <p className="text-center text-sm text-muted-foreground py-4">
              No passkeys registered. Add a passkey for passwordless login.
            </p>
          ) : (
            <>
              {/* Mobile view - Card list */}
              <div className="space-y-3 md:hidden">
                {passkeys.map((passkey) => (
                  <MobilePasskeyCard key={passkey.id} passkey={passkey} />
                ))}
              </div>

              {/* Desktop view - Table */}
              <div className="hidden md:block">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Name</TableHead>
                      <TableHead>Created</TableHead>
                      <TableHead>Last Used</TableHead>
                      <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {passkeys.map((passkey) => (
                      <TableRow key={passkey.id}>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <Key className="h-4 w-4 text-muted-foreground" />
                            <span className="font-medium">
                              {passkey.device_name || 'Unnamed Passkey'}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell>{formatDate(passkey.created_at)}</TableCell>
                        <TableCell>{formatDate(passkey.last_used_at)}</TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-1">
                            <Button
                              variant="ghost"
                              size="icon-sm"
                              onClick={() => openRenameDialog(passkey)}
                              title="Rename passkey"
                            >
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon-sm"
                              onClick={() => openDeleteDialog(passkey)}
                              title="Delete passkey"
                            >
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* Register Passkey Dialog */}
      <Dialog open={showRegisterDialog} onOpenChange={setShowRegisterDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Register New Passkey</DialogTitle>
            <DialogDescription>
              Add a new passkey for passwordless authentication.
            </DialogDescription>
          </DialogHeader>
          <PasskeyRegister 
            onSuccess={handleRegisterSuccess}
            onCancel={() => setShowRegisterDialog(false)}
          />
        </DialogContent>
      </Dialog>

      {/* Delete Passkey Dialog */}
      <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Passkey</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this passkey? You won't be able to use it for authentication anymore.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="flex-col sm:flex-row gap-2">
            <Button
              variant="outline"
              onClick={() => setShowDeleteDialog(false)}
              disabled={isDeleting}
              className="w-full sm:w-auto"
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={isDeleting}
              className="w-full sm:w-auto"
            >
              {isDeleting ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Deleting...
                </>
              ) : (
                'Delete Passkey'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Rename Passkey Dialog */}
      <Dialog open={showRenameDialog} onOpenChange={setShowRenameDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename Passkey</DialogTitle>
            <DialogDescription>
              Give your passkey a memorable name.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="passkey-name">Name</Label>
              <Input
                id="passkey-name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="e.g., MacBook Pro, iPhone"
              />
            </div>
          </div>
          <DialogFooter className="flex-col sm:flex-row gap-2">
            <Button
              variant="outline"
              onClick={() => setShowRenameDialog(false)}
              disabled={isRenaming}
              className="w-full sm:w-auto"
            >
              Cancel
            </Button>
            <Button
              onClick={handleRename}
              disabled={isRenaming || !newName.trim()}
              className="w-full sm:w-auto"
            >
              {isRenaming ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Saving...
                </>
              ) : (
                'Save'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

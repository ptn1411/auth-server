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
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { ApiKeyDialog } from './ApiKeyDialog';
import { AppSecretDialog } from './AppSecretDialog';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Plus, Key, MoreVertical, Pencil, Trash2, Ban, Loader2 } from 'lucide-react';
import type { ApiKeyResponse, ApiKeyWithSecretResponse } from '@/lib/auth-client';

interface ApiKeyListProps {
  appId: string;
  apiKeys: ApiKeyResponse[];
  isLoading?: boolean;
  onApiKeyCreated?: () => void;
}

export function ApiKeyList({
  appId,
  apiKeys,
  isLoading = false,
  onApiKeyCreated,
}: ApiKeyListProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [revokeDialogOpen, setRevokeDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [secretDialogOpen, setSecretDialogOpen] = useState(false);
  const [selectedApiKey, setSelectedApiKey] = useState<ApiKeyResponse | null>(null);
  const [newApiKeySecret, setNewApiKeySecret] = useState<string>('');
  const [isProcessing, setIsProcessing] = useState(false);

  const { revokeApiKey, deleteApiKey } = useAppsStore();

  const handleApiKeyCreated = (apiKey: ApiKeyWithSecretResponse) => {
    setCreateDialogOpen(false);
    setNewApiKeySecret(apiKey.key);
    setSecretDialogOpen(true);
    onApiKeyCreated?.();
  };

  const handleEditApiKey = (apiKey: ApiKeyResponse) => {
    setSelectedApiKey(apiKey);
    setEditDialogOpen(true);
  };

  const handleRevokeClick = (apiKey: ApiKeyResponse) => {
    setSelectedApiKey(apiKey);
    setRevokeDialogOpen(true);
  };

  const handleDeleteClick = (apiKey: ApiKeyResponse) => {
    setSelectedApiKey(apiKey);
    setDeleteDialogOpen(true);
  };

  const handleRevokeApiKey = async () => {
    if (!selectedApiKey) return;
    setIsProcessing(true);
    try {
      await revokeApiKey(appId, selectedApiKey.id);
      toast.success('API key revoked successfully');
      setRevokeDialogOpen(false);
      setSelectedApiKey(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to revoke API key');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleDeleteApiKey = async () => {
    if (!selectedApiKey) return;
    setIsProcessing(true);
    try {
      await deleteApiKey(appId, selectedApiKey.id);
      toast.success('API key deleted successfully');
      setDeleteDialogOpen(false);
      setSelectedApiKey(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete API key');
    } finally {
      setIsProcessing(false);
    }
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'Never';
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5" />
                API Keys
              </CardTitle>
              <CardDescription>
                Manage API keys for programmatic access
              </CardDescription>
            </div>
            <Button onClick={() => setCreateDialogOpen(true)} size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create API Key
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : apiKeys.length === 0 ? (
            <div className="text-center py-8">
              <Key className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No API keys yet</h3>
              <p className="text-muted-foreground mb-4">
                Create API keys for programmatic access to your app.
              </p>
              <Button onClick={() => setCreateDialogOpen(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create First API Key
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Key Prefix</TableHead>
                  <TableHead>Scopes</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Expires</TableHead>
                  <TableHead>Last Used</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {apiKeys.map((apiKey) => (
                  <TableRow key={apiKey.id}>
                    <TableCell className="font-medium">{apiKey.name}</TableCell>
                    <TableCell className="font-mono text-sm">
                      {apiKey.key_prefix}...
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {apiKey.scopes.slice(0, 2).map((scope) => (
                          <Badge key={scope} variant="outline" className="text-xs">
                            {scope}
                          </Badge>
                        ))}
                        {apiKey.scopes.length > 2 && (
                          <Badge variant="outline" className="text-xs">
                            +{apiKey.scopes.length - 2}
                          </Badge>
                        )}
                        {apiKey.scopes.length === 0 && (
                          <span className="text-muted-foreground text-xs">All</span>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      {apiKey.is_active ? (
                        <Badge variant="secondary">Active</Badge>
                      ) : (
                        <Badge variant="destructive">Revoked</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {formatDate(apiKey.expires_at)}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {formatDate(apiKey.last_used_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon-sm">
                            <MoreVertical className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => handleEditApiKey(apiKey)}>
                            <Pencil className="h-4 w-4 mr-2" />
                            Edit
                          </DropdownMenuItem>
                          {apiKey.is_active && (
                            <DropdownMenuItem
                              onClick={() => handleRevokeClick(apiKey)}
                              className="text-destructive"
                            >
                              <Ban className="h-4 w-4 mr-2" />
                              Revoke
                            </DropdownMenuItem>
                          )}
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => handleDeleteClick(apiKey)}
                            className="text-destructive"
                          >
                            <Trash2 className="h-4 w-4 mr-2" />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Create API Key Dialog */}
      <ApiKeyDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        appId={appId}
        onApiKeyCreated={handleApiKeyCreated}
      />

      {/* Edit API Key Dialog */}
      {selectedApiKey && (
        <ApiKeyDialog
          open={editDialogOpen}
          onOpenChange={setEditDialogOpen}
          appId={appId}
          apiKey={selectedApiKey}
          onApiKeyUpdated={() => {
            setEditDialogOpen(false);
            setSelectedApiKey(null);
          }}
        />
      )}

      {/* Revoke Confirmation Dialog */}
      <ConfirmDialog
        open={revokeDialogOpen}
        onOpenChange={setRevokeDialogOpen}
        title="Revoke API Key"
        description={`Are you sure you want to revoke "${selectedApiKey?.name}"? This will immediately invalidate the key.`}
        confirmText="Revoke"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleRevokeApiKey}
      />

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete API Key"
        description={`Are you sure you want to delete "${selectedApiKey?.name}"? This action cannot be undone.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleDeleteApiKey}
      />

      {/* Secret Display Dialog */}
      <AppSecretDialog
        open={secretDialogOpen}
        onOpenChange={setSecretDialogOpen}
        appName="API Key"
        secret={newApiKeySecret}
        isNewApp={true}
      />
    </>
  );
}

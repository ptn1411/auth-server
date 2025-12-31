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
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { WebhookDialog } from './WebhookDialog';
import { AppSecretDialog } from './AppSecretDialog';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Plus, Webhook, MoreVertical, Pencil, Trash2, Loader2 } from 'lucide-react';
import type { WebhookResponse, WebhookWithSecretResponse } from '@/lib/auth-client';

interface WebhookListProps {
  appId: string;
  webhooks?: WebhookResponse[];
  isLoading?: boolean;
  onWebhookCreated?: () => void;
}

export function WebhookList({
  appId,
  webhooks = [],
  isLoading = false,
  onWebhookCreated,
}: WebhookListProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [secretDialogOpen, setSecretDialogOpen] = useState(false);
  const [selectedWebhook, setSelectedWebhook] = useState<WebhookResponse | null>(null);
  const [newWebhookSecret, setNewWebhookSecret] = useState<string>('');
  const [isDeleting, setIsDeleting] = useState(false);

  const { deleteWebhook } = useAppsStore();

  const handleWebhookCreated = (webhook: WebhookWithSecretResponse) => {
    setCreateDialogOpen(false);
    setNewWebhookSecret(webhook.secret);
    setSecretDialogOpen(true);
    onWebhookCreated?.();
  };

  const handleEditWebhook = (webhook: WebhookResponse) => {
    setSelectedWebhook(webhook);
    setEditDialogOpen(true);
  };

  const handleDeleteClick = (webhook: WebhookResponse) => {
    setSelectedWebhook(webhook);
    setDeleteDialogOpen(true);
  };

  const handleDeleteWebhook = async () => {
    if (!selectedWebhook) return;
    setIsDeleting(true);
    try {
      await deleteWebhook(appId, selectedWebhook.id);
      toast.success('Webhook deleted successfully');
      setDeleteDialogOpen(false);
      setSelectedWebhook(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete webhook');
    } finally {
      setIsDeleting(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Webhook className="h-5 w-5" />
                Webhooks
              </CardTitle>
              <CardDescription>
                Receive event notifications via HTTP callbacks
              </CardDescription>
            </div>
            <Button onClick={() => setCreateDialogOpen(true)} size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create Webhook
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : webhooks.length === 0 ? (
            <div className="text-center py-8">
              <Webhook className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No webhooks yet</h3>
              <p className="text-muted-foreground mb-4">
                Create webhooks to receive event notifications.
              </p>
              <Button onClick={() => setCreateDialogOpen(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create First Webhook
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>URL</TableHead>
                  <TableHead>Events</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {webhooks.map((webhook) => (
                  <TableRow key={webhook.id}>
                    <TableCell className="font-mono text-sm max-w-[200px] truncate">
                      {webhook.url}
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {webhook.events.slice(0, 2).map((event) => (
                          <Badge key={event} variant="outline" className="text-xs">
                            {event}
                          </Badge>
                        ))}
                        {webhook.events.length > 2 && (
                          <Badge variant="outline" className="text-xs">
                            +{webhook.events.length - 2}
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      {webhook.is_active ? (
                        <Badge variant="secondary">Active</Badge>
                      ) : (
                        <Badge variant="outline">Inactive</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {formatDate(webhook.created_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon-sm">
                            <MoreVertical className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => handleEditWebhook(webhook)}>
                            <Pencil className="h-4 w-4 mr-2" />
                            Edit
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onClick={() => handleDeleteClick(webhook)}
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

      {/* Create Webhook Dialog */}
      <WebhookDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        appId={appId}
        onWebhookCreated={handleWebhookCreated}
      />

      {/* Edit Webhook Dialog */}
      {selectedWebhook && (
        <WebhookDialog
          open={editDialogOpen}
          onOpenChange={setEditDialogOpen}
          appId={appId}
          webhook={selectedWebhook}
          onWebhookUpdated={() => {
            setEditDialogOpen(false);
            setSelectedWebhook(null);
          }}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete Webhook"
        description={`Are you sure you want to delete this webhook? This action cannot be undone.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isDeleting}
        onConfirm={handleDeleteWebhook}
      />

      {/* Secret Display Dialog */}
      <AppSecretDialog
        open={secretDialogOpen}
        onOpenChange={setSecretDialogOpen}
        appName="Webhook"
        secret={newWebhookSecret}
        isNewApp={true}
      />
    </>
  );
}

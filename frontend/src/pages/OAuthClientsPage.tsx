import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useAuthStore } from "@/stores/authStore";
import {
  useOAuthClientsStore,
  type OAuthClient,
  type OAuthClientWithSecret,
} from "@/stores/oauthClientsStore";
import {
  Check,
  Copy,
  ExternalLink,
  Key,
  Loader2,
  Pencil,
  Plus,
  RefreshCw,
  Shield,
  Trash2,
} from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

export function OAuthClientsPage() {
  const {
    clients,
    isLoading,
    error,
    fetchClients,
    createClient,
    updateClient,
    deleteClient,
    regenerateSecret,
  } = useOAuthClientsStore();
  const { isAuthenticated, isLoading: authLoading } = useAuthStore();

  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [secretDialogOpen, setSecretDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [regenerateDialogOpen, setRegenerateDialogOpen] = useState(false);
  const [newClient, setNewClient] = useState<OAuthClientWithSecret | null>(
    null
  );
  const [selectedClient, setSelectedClient] = useState<OAuthClient | null>(
    null
  );
  const [newSecret, setNewSecret] = useState<string | null>(null);
  const [copiedField, setCopiedField] = useState<string | null>(null);

  // Form state
  const [name, setName] = useState("");
  const [redirectUris, setRedirectUris] = useState("");
  const [isInternal, setIsInternal] = useState(false);
  const [isActive, setIsActive] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Wait for auth to be ready before fetching clients
  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      fetchClients().catch(() => {
        // Error is handled in store
      });
    }
  }, [fetchClients, authLoading, isAuthenticated]);

  const handleCreate = async () => {
    if (!name.trim()) {
      toast.error("Name is required");
      return;
    }

    const uris = redirectUris
      .split("\n")
      .map((uri) => uri.trim())
      .filter((uri) => uri.length > 0);

    if (uris.length === 0) {
      toast.error("At least one redirect URI is required");
      return;
    }

    // Validate HTTPS for external apps
    if (!isInternal) {
      const invalidUris = uris.filter((uri) => !uri.startsWith("http://"));
      if (invalidUris.length > 0) {
        toast.error("External apps require HTTPS redirect URIs");
        return;
      }
    }

    setIsSubmitting(true);
    try {
      const client = await createClient({
        name: name.trim(),
        redirect_uris: uris,
        is_internal: isInternal,
      });
      setNewClient(client);
      setCreateDialogOpen(false);
      setSecretDialogOpen(true);
      resetForm();
      toast.success("OAuth client created successfully");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to create client"
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const resetForm = () => {
    setName("");
    setRedirectUris("");
    setIsInternal(false);
    setIsActive(true);
  };

  const handleEdit = (client: OAuthClient) => {
    setSelectedClient(client);
    setName(client.name);
    setRedirectUris(client.redirect_uris.join("\n"));
    setIsActive(client.is_active);
    setEditDialogOpen(true);
  };

  const handleUpdate = async () => {
    if (!selectedClient || !name.trim()) {
      toast.error("Name is required");
      return;
    }

    const uris = redirectUris
      .split("\n")
      .map((uri) => uri.trim())
      .filter((uri) => uri.length > 0);

    if (uris.length === 0) {
      toast.error("At least one redirect URI is required");
      return;
    }

    setIsSubmitting(true);
    try {
      await updateClient(selectedClient.id, {
        name: name.trim(),
        redirect_uris: uris,
        is_active: isActive,
      });
      setEditDialogOpen(false);
      resetForm();
      setSelectedClient(null);
      toast.success("OAuth client updated successfully");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to update client"
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedClient) return;

    setIsSubmitting(true);
    try {
      await deleteClient(selectedClient.id);
      setDeleteDialogOpen(false);
      setSelectedClient(null);
      toast.success("OAuth client deleted successfully");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to delete client"
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRegenerateSecret = async () => {
    if (!selectedClient) return;

    setIsSubmitting(true);
    try {
      const secret = await regenerateSecret(selectedClient.id);
      setNewSecret(secret);
      setRegenerateDialogOpen(false);
      setSecretDialogOpen(true);
      toast.success("Client secret regenerated successfully");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to regenerate secret"
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCopy = async (text: string, field: string) => {
    await navigator.clipboard.writeText(text);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
    toast.success("Copied to clipboard");
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  };

  if (error) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold flex items-center gap-2">
              <Key className="h-8 w-8" />
              OAuth Clients
            </h1>
            <p className="text-muted-foreground">
              Manage OAuth2 client applications
            </p>
          </div>
        </div>
        <Card>
          <CardContent className="py-8 text-center">
            <p className="text-destructive">{error}</p>
            <Button
              variant="outline"
              className="mt-4"
              onClick={() => fetchClients()}>
              Retry
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Key className="h-8 w-8" />
            OAuth Clients
          </h1>
          <p className="text-muted-foreground">
            Manage OAuth2 client applications for third-party integrations
          </p>
        </div>
        <Button onClick={() => setCreateDialogOpen(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create Client
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Registered Clients</CardTitle>
          <CardDescription>
            OAuth2 clients that can request access to user data
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading || authLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : clients.length === 0 ? (
            <div className="text-center py-8">
              <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <p className="text-muted-foreground">
                No OAuth clients registered yet
              </p>
              <Button
                variant="outline"
                className="mt-4"
                onClick={() => setCreateDialogOpen(true)}>
                Create your first client
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Client ID</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Redirect URIs</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {clients.map((client) => (
                  <TableRow key={client.id}>
                    <TableCell className="font-medium">{client.name}</TableCell>
                    <TableCell>
                      <code className="text-xs bg-muted px-2 py-1 rounded">
                        {client.client_id.substring(0, 8)}...
                      </code>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        onClick={() =>
                          handleCopy(client.client_id, "client_id")
                        }>
                        <Copy className="h-4 w-4" />
                      </Button>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={client.is_internal ? "secondary" : "default"}>
                        {client.is_internal ? "Internal" : "External"}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-col gap-1">
                        {client.redirect_uris.slice(0, 2).map((uri, idx) => (
                          <span
                            key={idx}
                            className="text-xs text-muted-foreground flex items-center gap-1">
                            <ExternalLink className="h-3 w-3" />
                            {uri.length > 30
                              ? `${uri.substring(0, 30)}...`
                              : uri}
                          </span>
                        ))}
                        {client.redirect_uris.length > 2 && (
                          <span className="text-xs text-muted-foreground">
                            +{client.redirect_uris.length - 2} more
                          </span>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={client.is_active ? "default" : "destructive"}>
                        {client.is_active ? "Active" : "Inactive"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {formatDate(client.created_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleEdit(client)}
                          title="Edit">
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => {
                            setSelectedClient(client);
                            setRegenerateDialogOpen(true);
                          }}
                          title="Regenerate Secret">
                          <RefreshCw className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => {
                            setSelectedClient(client);
                            setDeleteDialogOpen(true);
                          }}
                          title="Delete"
                          className="text-destructive hover:text-destructive">
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

      {/* Create Client Dialog */}
      <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create OAuth Client</DialogTitle>
            <DialogDescription>
              Register a new OAuth2 client application
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Application Name</Label>
              <Input
                id="name"
                placeholder="My Application"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>
            <div className="form-group">
              <Label htmlFor="redirect_uris">
                Redirect URIs (one per line)
              </Label>
              <textarea
                id="redirect_uris"
                className="w-full min-h-[100px] px-3 py-2 text-sm rounded-md border border-input bg-background"
                placeholder="https://myapp.com/callback&#10;https://myapp.com/auth/callback"
                value={redirectUris}
                onChange={(e) => setRedirectUris(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                External apps require HTTPS URIs. Internal apps can use HTTP for
                development.
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                Available scopes: openid, profile, email, profile.read,
                email.read, offline_access
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="is_internal"
                checked={isInternal}
                onCheckedChange={(checked) => setIsInternal(checked === true)}
              />
              <div className="space-y-0.5">
                <Label htmlFor="is_internal">Internal Application</Label>
                <p className="text-xs text-muted-foreground">
                  Internal apps skip user consent and can use HTTP URIs
                </p>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={isSubmitting}>
              {isSubmitting && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              Create Client
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Client Secret Dialog */}
      <Dialog open={secretDialogOpen} onOpenChange={setSecretDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {newClient ? "Client Created Successfully" : "New Client Secret"}
            </DialogTitle>
            <DialogDescription>
              Save these credentials securely. The client secret will only be
              shown once.
            </DialogDescription>
          </DialogHeader>
          {(newClient || newSecret) && (
            <div className="space-y-4 py-4">
              {newClient && (
                <div className="space-y-2">
                  <Label>Client ID</Label>
                  <div className="flex items-center gap-2">
                    <Input
                      readOnly
                      value={newClient.client_id}
                      className="font-mono text-sm"
                    />
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() =>
                        handleCopy(newClient.client_id, "client_id")
                      }>
                      {copiedField === "client_id" ? (
                        <Check className="h-4 w-4 text-green-500" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                </div>
              )}
              <div className="space-y-2">
                <Label>Client Secret</Label>
                <div className="flex items-center gap-2">
                  <Input
                    readOnly
                    value={newClient?.client_secret || newSecret || ""}
                    className="font-mono text-sm"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() =>
                      handleCopy(
                        newClient?.client_secret || newSecret || "",
                        "client_secret"
                      )
                    }>
                    {copiedField === "client_secret" ? (
                      <Check className="h-4 w-4 text-green-500" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
                <p className="text-xs text-destructive">
                  ⚠️ This secret will not be shown again. Store it securely.
                </p>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button
              onClick={() => {
                setSecretDialogOpen(false);
                setNewClient(null);
                setNewSecret(null);
              }}>
              Done
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Client Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit OAuth Client</DialogTitle>
            <DialogDescription>
              Update the OAuth client settings
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">Application Name</Label>
              <Input
                id="edit-name"
                placeholder="My Application"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>
            <div className="form-group">
              <Label htmlFor="edit-redirect_uris">
                Redirect URIs (one per line)
              </Label>
              <textarea
                id="edit-redirect_uris"
                className="w-full min-h-[100px] px-3 py-2 text-sm rounded-md border border-input bg-background"
                placeholder="https://myapp.com/callback&#10;https://myapp.com/auth/callback"
                value={redirectUris}
                onChange={(e) => setRedirectUris(e.target.value)}
              />
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="edit-is_active"
                checked={isActive}
                onCheckedChange={(checked) => setIsActive(checked === true)}
              />
              <div className="space-y-0.5">
                <Label htmlFor="edit-is_active">Active</Label>
                <p className="text-xs text-muted-foreground">
                  Inactive clients cannot be used for authentication
                </p>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setEditDialogOpen(false);
                resetForm();
                setSelectedClient(null);
              }}>
              Cancel
            </Button>
            <Button onClick={handleUpdate} disabled={isSubmitting}>
              {isSubmitting && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              Save Changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete OAuth Client</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{selectedClient?.name}"? This
              action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDeleteDialogOpen(false);
                setSelectedClient(null);
              }}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={isSubmitting}>
              {isSubmitting && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Regenerate Secret Confirmation Dialog */}
      <Dialog
        open={regenerateDialogOpen}
        onOpenChange={setRegenerateDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Regenerate Client Secret</DialogTitle>
            <DialogDescription>
              Are you sure you want to regenerate the secret for "
              {selectedClient?.name}"? The old secret will be invalidated
              immediately.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setRegenerateDialogOpen(false);
                setSelectedClient(null);
              }}>
              Cancel
            </Button>
            <Button onClick={handleRegenerateSecret} disabled={isSubmitting}>
              {isSubmitting && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              Regenerate
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

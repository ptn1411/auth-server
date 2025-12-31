import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import {
  RoleList,
  PermissionList,
  AppUserList,
  WebhookList,
  ApiKeyList,
  IpRuleList,
  AppSecretDialog,
} from '@/components/apps';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import {
  ArrowLeft,
  Package,
  Users,
  Shield,
  Lock,
  Webhook,
  Key,
  Globe,
  RefreshCw,
  Loader2,
  Copy,
  Check,
} from 'lucide-react';

type AppDetailTab = 'overview' | 'users' | 'roles' | 'permissions' | 'webhooks' | 'api-keys' | 'ip-rules';

export function AppDetailPage() {
  const { appId } = useParams<{ appId: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<AppDetailTab>('overview');
  const [regenerateDialogOpen, setRegenerateDialogOpen] = useState(false);
  const [secretDialogOpen, setSecretDialogOpen] = useState(false);
  const [regeneratedSecret, setRegeneratedSecret] = useState('');
  const [isRegenerating, setIsRegenerating] = useState(false);
  const [copiedId, setCopiedId] = useState(false);

  const {
    currentApp,
    isLoading,
    error,
    fetchAppDetail,
    regenerateSecret,
    fetchAppUsers,
    clearCurrentApp,
  } = useAppsStore();

  useEffect(() => {
    if (appId) {
      fetchAppDetail(appId);
    }
    return () => {
      clearCurrentApp();
    };
  }, [appId, fetchAppDetail, clearCurrentApp]);

  const handleRegenerateSecret = async () => {
    if (!appId) return;
    setIsRegenerating(true);
    try {
      const secret = await regenerateSecret(appId);
      setRegeneratedSecret(secret);
      setRegenerateDialogOpen(false);
      setSecretDialogOpen(true);
      toast.success('App secret regenerated successfully');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to regenerate secret');
    } finally {
      setIsRegenerating(false);
    }
  };

  const handlePageChange = (page: number) => {
    if (appId) {
      fetchAppUsers(appId, { page, limit: 10 });
    }
  };

  const handleCopyId = async () => {
    if (appId) {
      await navigator.clipboard.writeText(appId);
      setCopiedId(true);
      setTimeout(() => setCopiedId(false), 2000);
      toast.success('App ID copied to clipboard');
    }
  };

  if (isLoading && !currentApp) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        <Button variant="ghost" onClick={() => navigate('/apps')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Apps
        </Button>
        <div className="text-center py-12">
          <p className="text-destructive">{error}</p>
        </div>
      </div>
    );
  }

  if (!currentApp) {
    return (
      <div className="space-y-4">
        <Button variant="ghost" onClick={() => navigate('/apps')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Apps
        </Button>
        <div className="text-center py-12">
          <p className="text-muted-foreground">App not found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate('/apps')}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div className="flex-1">
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Package className="h-8 w-8" />
            {currentApp.app.name || 'App Details'}
          </h1>
          {currentApp.app.code && (
            <p className="text-muted-foreground">
              <Badge variant="secondary" className="font-mono">
                {currentApp.app.code}
              </Badge>
            </p>
          )}
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as AppDetailTab)}>
        <TabsList className="grid w-full grid-cols-7">
          <TabsTrigger value="overview" className="flex items-center gap-1">
            <Package className="h-4 w-4" />
            <span className="hidden sm:inline">Overview</span>
          </TabsTrigger>
          <TabsTrigger value="users" className="flex items-center gap-1">
            <Users className="h-4 w-4" />
            <span className="hidden sm:inline">Users</span>
          </TabsTrigger>
          <TabsTrigger value="roles" className="flex items-center gap-1">
            <Shield className="h-4 w-4" />
            <span className="hidden sm:inline">Roles</span>
          </TabsTrigger>
          <TabsTrigger value="permissions" className="flex items-center gap-1">
            <Lock className="h-4 w-4" />
            <span className="hidden sm:inline">Permissions</span>
          </TabsTrigger>
          <TabsTrigger value="webhooks" className="flex items-center gap-1">
            <Webhook className="h-4 w-4" />
            <span className="hidden sm:inline">Webhooks</span>
          </TabsTrigger>
          <TabsTrigger value="api-keys" className="flex items-center gap-1">
            <Key className="h-4 w-4" />
            <span className="hidden sm:inline">API Keys</span>
          </TabsTrigger>
          <TabsTrigger value="ip-rules" className="flex items-center gap-1">
            <Globe className="h-4 w-4" />
            <span className="hidden sm:inline">IP Rules</span>
          </TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>App Information</CardTitle>
              <CardDescription>
                Basic information about your application
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="text-sm font-medium text-muted-foreground">App ID</label>
                  <div className="flex items-center gap-2 mt-1">
                    <code className="text-sm bg-muted px-2 py-1 rounded flex-1 truncate">
                      {appId}
                    </code>
                    <Button variant="ghost" size="icon-sm" onClick={handleCopyId}>
                      {copiedId ? (
                        <Check className="h-4 w-4 text-green-500" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                </div>
                {currentApp.app.code && (
                  <div>
                    <label className="text-sm font-medium text-muted-foreground">App Code</label>
                    <p className="mt-1 font-mono">{currentApp.app.code}</p>
                  </div>
                )}
                {currentApp.app.name && (
                  <div>
                    <label className="text-sm font-medium text-muted-foreground">App Name</label>
                    <p className="mt-1">{currentApp.app.name}</p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Security</CardTitle>
              <CardDescription>
                Manage your app's security settings
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">App Secret</p>
                  <p className="text-sm text-muted-foreground">
                    Regenerate your app secret if it has been compromised
                  </p>
                </div>
                <Button
                  variant="outline"
                  onClick={() => setRegenerateDialogOpen(true)}
                >
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Regenerate Secret
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Quick Stats */}
          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  Users
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">
                  {currentApp.users?.total ?? 0}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  Roles
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">
                  {currentApp.roles.length}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  Webhooks
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">
                  {currentApp.webhooks.length}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  API Keys
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">
                  {currentApp.apiKeys.length}
                </p>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Users Tab */}
        <TabsContent value="users">
          {appId && (
            <AppUserList
              appId={appId}
              users={currentApp.users}
              roles={currentApp.roles}
              isLoading={isLoading}
              onPageChange={handlePageChange}
            />
          )}
        </TabsContent>

        {/* Roles Tab */}
        <TabsContent value="roles">
          {appId && (
            <RoleList
              appId={appId}
              roles={currentApp.roles}
              permissions={currentApp.permissions}
              isLoading={isLoading}
              onPermissionsChanged={() => fetchAppDetail(appId)}
            />
          )}
        </TabsContent>

        {/* Permissions Tab */}
        <TabsContent value="permissions">
          {appId && (
            <PermissionList
              appId={appId}
              permissions={currentApp.permissions}
              isLoading={isLoading}
            />
          )}
        </TabsContent>

        {/* Webhooks Tab */}
        <TabsContent value="webhooks">
          {appId && (
            <WebhookList
              appId={appId}
              webhooks={currentApp.webhooks}
              isLoading={isLoading}
            />
          )}
        </TabsContent>

        {/* API Keys Tab */}
        <TabsContent value="api-keys">
          {appId && (
            <ApiKeyList
              appId={appId}
              apiKeys={currentApp.apiKeys}
              isLoading={isLoading}
            />
          )}
        </TabsContent>

        {/* IP Rules Tab */}
        <TabsContent value="ip-rules">
          {appId && (
            <IpRuleList
              appId={appId}
              ipRules={currentApp.ipRules}
              isLoading={isLoading}
            />
          )}
        </TabsContent>
      </Tabs>

      {/* Regenerate Secret Confirmation Dialog */}
      <ConfirmDialog
        open={regenerateDialogOpen}
        onOpenChange={setRegenerateDialogOpen}
        title="Regenerate App Secret"
        description="Are you sure you want to regenerate the app secret? The current secret will be invalidated immediately. Any services using the old secret will stop working."
        confirmText="Regenerate"
        variant="destructive"
        isLoading={isRegenerating}
        onConfirm={handleRegenerateSecret}
      />

      {/* New Secret Dialog */}
      <AppSecretDialog
        open={secretDialogOpen}
        onOpenChange={setSecretDialogOpen}
        appName={currentApp.app.name || 'App'}
        secret={regeneratedSecret}
        isNewApp={false}
      />
    </div>
  );
}

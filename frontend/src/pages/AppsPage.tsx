import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { AppList, CreateAppDialog, AppSecretDialog } from '@/components/apps';
import { useAppsStore, type AppWithSecret } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Plus, Package } from 'lucide-react';
import type { AppResponse } from '@/lib/auth-client';

export function AppsPage() {
  const navigate = useNavigate();
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [secretDialogOpen, setSecretDialogOpen] = useState(false);
  const [newAppSecret, setNewAppSecret] = useState<string>('');
  const [newAppName, setNewAppName] = useState<string>('');
  const [regenerateDialogOpen, setRegenerateDialogOpen] = useState(false);
  const [regeneratedSecret, setRegeneratedSecret] = useState<string>('');
  const [regenerateAppName, setRegenerateAppName] = useState<string>('');

  const { regenerateSecret } = useAppsStore();

  const handleViewDetails = (app: AppResponse) => {
    navigate(`/apps/${app.id}`);
  };

  const handleRegenerateSecret = async (app: AppResponse) => {
    try {
      const secret = await regenerateSecret(app.id);
      setRegeneratedSecret(secret);
      setRegenerateAppName(app.name);
      setRegenerateDialogOpen(true);
      toast.success('App secret regenerated successfully');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to regenerate secret');
    }
  };

  const handleAppCreated = (app: AppWithSecret) => {
    setCreateDialogOpen(false);
    setNewAppSecret(app.secret);
    setNewAppName(app.name);
    setSecretDialogOpen(true);
  };

  return (
    <div className="space-y-4 sm:space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl sm:text-3xl font-bold flex items-center gap-2">
            <Package className="h-6 w-6 sm:h-8 sm:w-8" />
            My Apps
          </h1>
          <p className="text-sm sm:text-base text-muted-foreground">
            Manage your external applications and integrations
          </p>
        </div>
        <Button onClick={() => setCreateDialogOpen(true)} className="w-full sm:w-auto">
          <Plus className="h-4 w-4 mr-2" />
          Create App
        </Button>
      </div>

      <AppList
        onViewDetails={handleViewDetails}
        onRegenerateSecret={handleRegenerateSecret}
      />

      {/* Create App Dialog */}
      <CreateAppDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        onAppCreated={handleAppCreated}
      />

      {/* New App Secret Dialog */}
      <AppSecretDialog
        open={secretDialogOpen}
        onOpenChange={setSecretDialogOpen}
        appName={newAppName}
        secret={newAppSecret}
        isNewApp={true}
      />

      {/* Regenerated Secret Dialog */}
      <AppSecretDialog
        open={regenerateDialogOpen}
        onOpenChange={setRegenerateDialogOpen}
        appName={regenerateAppName}
        secret={regeneratedSecret}
        isNewApp={false}
      />
    </div>
  );
}

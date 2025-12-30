import { useEffect, useState } from 'react';
import { useParams, useNavigate, useSearchParams } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import {
  AdminAppDetailCard,
  EditAppDialog,
} from '@/components/admin/apps';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import { ArrowLeft, Loader2 } from 'lucide-react';

export function AdminAppDetailPage() {
  const { appId } = useParams<{ appId: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  
  const [editDialogOpen, setEditDialogOpen] = useState(searchParams.get('edit') === 'true');
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const {
    currentApp,
    isLoading,
    fetchApp,
    deleteApp,
    clearCurrentApp,
  } = useAdminStore();

  useEffect(() => {
    if (appId) {
      fetchApp(appId);
    }
    
    return () => {
      clearCurrentApp();
    };
  }, [appId, fetchApp, clearCurrentApp]);

  // Handle edit query param
  useEffect(() => {
    if (searchParams.get('edit') === 'true' && currentApp) {
      setEditDialogOpen(true);
      // Remove the query param after opening dialog
      setSearchParams({});
    }
  }, [searchParams, currentApp, setSearchParams]);

  const handleDelete = async () => {
    if (!currentApp) return;
    setIsProcessing(true);
    try {
      await deleteApp(currentApp.id);
      toast.success(`Application "${currentApp.name}" has been deleted`);
      setDeleteDialogOpen(false);
      navigate('/admin/apps');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete application');
    } finally {
      setIsProcessing(false);
    }
  };

  if (isLoading && !currentApp) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!currentApp) {
    return (
      <div className="text-center py-12">
        <h2 className="text-lg font-medium mb-2">Application not found</h2>
        <p className="text-muted-foreground mb-4">
          The application you're looking for doesn't exist or has been deleted.
        </p>
        <Button onClick={() => navigate('/admin/apps')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Applications
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="sm" onClick={() => navigate('/admin/apps')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back
        </Button>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{currentApp.name}</h1>
          <p className="text-muted-foreground">App ID: {currentApp.id}</p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <AdminAppDetailCard
          app={currentApp}
          isLoading={isLoading}
          onEdit={() => setEditDialogOpen(true)}
          onDelete={() => setDeleteDialogOpen(true)}
        />
      </div>

      {/* Edit App Dialog */}
      <EditAppDialog
        open={editDialogOpen}
        onOpenChange={setEditDialogOpen}
        app={currentApp}
        onSuccess={() => {
          if (appId) {
            fetchApp(appId);
          }
        }}
      />

      {/* Delete App Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete Application"
        description={`Are you sure you want to permanently delete "${currentApp.name}"? This action cannot be undone and all app data including roles, permissions, webhooks, and API keys will be lost.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isProcessing}
        onConfirm={handleDelete}
      />
    </div>
  );
}

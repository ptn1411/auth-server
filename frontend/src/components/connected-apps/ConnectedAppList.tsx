import { useState } from 'react';
import { ConnectedAppCard } from './ConnectedAppCard';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { toast } from 'sonner';
import type { ConnectedApp } from '@/lib/auth-client';

interface ConnectedAppListProps {
  apps: ConnectedApp[];
  onRevoke: (clientId: string) => Promise<void>;
  isLoading?: boolean;
}

export function ConnectedAppList({ apps, onRevoke, isLoading }: ConnectedAppListProps) {
  const [revokeDialog, setRevokeDialog] = useState<{
    open: boolean;
    app: ConnectedApp | null;
  }>({ open: false, app: null });
  const [isRevoking, setIsRevoking] = useState(false);

  const handleRevokeClick = (app: ConnectedApp) => {
    setRevokeDialog({ open: true, app });
  };

  const handleRevokeConfirm = async () => {
    if (!revokeDialog.app) return;

    setIsRevoking(true);
    try {
      await onRevoke(revokeDialog.app.client_id);
      toast.success(`Access revoked for ${revokeDialog.app.client_name}`);
    } catch {
      toast.error('Failed to revoke access');
    } finally {
      setIsRevoking(false);
      setRevokeDialog({ open: false, app: null });
    }
  };

  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {[1, 2, 3].map((i) => (
          <div
            key={i}
            className="h-40 rounded-lg border bg-muted/50 animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (apps.length === 0) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">
          You haven't authorized any third-party applications yet.
        </p>
      </div>
    );
  }

  return (
    <>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {apps.map((app) => (
          <ConnectedAppCard
            key={app.client_id}
            app={app}
            onRevoke={handleRevokeClick}
            isRevoking={isRevoking && revokeDialog.app?.client_id === app.client_id}
          />
        ))}
      </div>

      <ConfirmDialog
        open={revokeDialog.open}
        onOpenChange={(open) => setRevokeDialog({ open, app: open ? revokeDialog.app : null })}
        title="Revoke Access"
        description={`Are you sure you want to revoke access for "${revokeDialog.app?.client_name}"? This app will no longer be able to access your data.`}
        confirmText="Revoke Access"
        variant="destructive"
        isLoading={isRevoking}
        onConfirm={handleRevokeConfirm}
      />
    </>
  );
}

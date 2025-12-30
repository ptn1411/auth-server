import { useEffect } from 'react';
import { ConnectedAppList } from '@/components/connected-apps';
import { useConnectedAppsStore } from '@/stores/connectedAppsStore';
import { Link2 } from 'lucide-react';

export function ConnectedAppsPage() {
  const { apps, isLoading, error, fetchConnectedApps, revokeConsent, clearError } = useConnectedAppsStore();

  useEffect(() => {
    fetchConnectedApps();
    return () => clearError();
  }, [fetchConnectedApps, clearError]);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold flex items-center gap-2">
          <Link2 className="h-8 w-8" />
          Connected Apps
        </h1>
        <p className="text-muted-foreground">
          Manage third-party applications that have access to your account
        </p>
      </div>

      {error && (
        <div className="text-center py-4">
          <p className="text-destructive">{error}</p>
        </div>
      )}

      <ConnectedAppList
        apps={apps}
        onRevoke={revokeConsent}
        isLoading={isLoading}
      />
    </div>
  );
}

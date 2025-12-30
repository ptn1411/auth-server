import { useEffect } from 'react';
import { AppCard } from './AppCard';
import { useAppsStore } from '@/stores/appsStore';
import { Loader2, Package } from 'lucide-react';
import type { AppResponse } from '@/lib/auth-client';

interface AppListProps {
  onViewDetails: (app: AppResponse) => void;
  onRegenerateSecret: (app: AppResponse) => void;
}

export function AppList({ onViewDetails, onRegenerateSecret }: AppListProps) {
  const { apps, isLoading, error, fetchApps } = useAppsStore();

  useEffect(() => {
    fetchApps();
  }, [fetchApps]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <p className="text-destructive">{error}</p>
      </div>
    );
  }

  if (apps.length === 0) {
    return (
      <div className="text-center py-12">
        <Package className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
        <h3 className="text-lg font-medium mb-2">No apps yet</h3>
        <p className="text-muted-foreground">
          Create your first app to get started with integrations.
        </p>
      </div>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {apps.map((app) => (
        <AppCard
          key={app.id}
          app={app}
          onViewDetails={onViewDetails}
          onRegenerateSecret={onRegenerateSecret}
        />
      ))}
    </div>
  );
}

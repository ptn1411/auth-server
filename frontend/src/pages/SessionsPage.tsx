import { SessionList } from '@/components/security/SessionList';

export function SessionsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Sessions</h1>
        <p className="text-muted-foreground">
          Manage your active sessions across devices
        </p>
      </div>
      
      <SessionList />
    </div>
  );
}

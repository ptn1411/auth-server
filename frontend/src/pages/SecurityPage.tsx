import { MfaSetup } from '@/components/security/MfaSetup';
import { PasskeyList } from '@/components/security/PasskeyList';

export function SecurityPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Security</h1>
        <p className="text-muted-foreground">
          Manage your security settings and authentication methods
        </p>
      </div>
      
      <MfaSetup />
      <PasskeyList />
    </div>
  );
}

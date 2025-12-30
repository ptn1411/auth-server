import { useEffect, useState } from 'react';
import { useAdminStore } from '@/stores/adminStore';
import {
  IpRuleTable,
  CreateIpRuleDialog,
  IpCheckForm,
} from '@/components/admin/ip-rules';

export function AdminIpRulesPage() {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);

  const {
    ipRules,
    isLoading,
    fetchIpRules,
  } = useAdminStore();

  useEffect(() => {
    fetchIpRules();
  }, [fetchIpRules]);

  const handleIpRuleCreated = () => {
    setCreateDialogOpen(false);
    // IP rules are automatically updated in the store
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">IP Rules</h1>
        <p className="text-muted-foreground">
          Manage global IP whitelist and blacklist rules for the entire system
        </p>
      </div>

      <IpCheckForm />

      <IpRuleTable
        ipRules={ipRules}
        isLoading={isLoading}
        onCreateClick={() => setCreateDialogOpen(true)}
      />

      <CreateIpRuleDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        onIpRuleCreated={handleIpRuleCreated}
      />
    </div>
  );
}

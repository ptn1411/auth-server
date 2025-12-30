import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { IpRuleDialog } from './IpRuleDialog';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Plus, Globe, Trash2, Loader2 } from 'lucide-react';
import type { IpRuleResponse } from '@/lib/auth-client';

interface IpRuleListProps {
  appId: string;
  ipRules: IpRuleResponse[];
  isLoading?: boolean;
  onIpRuleCreated?: () => void;
}

export function IpRuleList({
  appId,
  ipRules,
  isLoading = false,
  onIpRuleCreated,
}: IpRuleListProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [selectedIpRule, setSelectedIpRule] = useState<IpRuleResponse | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const { deleteIpRule } = useAppsStore();

  const handleIpRuleCreated = () => {
    setCreateDialogOpen(false);
    onIpRuleCreated?.();
  };

  const handleDeleteClick = (ipRule: IpRuleResponse) => {
    setSelectedIpRule(ipRule);
    setDeleteDialogOpen(true);
  };

  const handleDeleteIpRule = async () => {
    if (!selectedIpRule) return;
    setIsDeleting(true);
    try {
      await deleteIpRule(appId, selectedIpRule.id);
      toast.success('IP rule deleted successfully');
      setDeleteDialogOpen(false);
      setSelectedIpRule(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to delete IP rule');
    } finally {
      setIsDeleting(false);
    }
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'Never';
    return new Date(dateString).toLocaleDateString();
  };

  const getRuleTypeBadge = (ruleType: string) => {
    if (ruleType === 'whitelist') {
      return <Badge variant="secondary">Whitelist</Badge>;
    }
    return <Badge variant="destructive">Blacklist</Badge>;
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Globe className="h-5 w-5" />
                IP Rules
              </CardTitle>
              <CardDescription>
                Control access by IP address
              </CardDescription>
            </div>
            <Button onClick={() => setCreateDialogOpen(true)} size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create IP Rule
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : ipRules.length === 0 ? (
            <div className="text-center py-8">
              <Globe className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No IP rules yet</h3>
              <p className="text-muted-foreground mb-4">
                Create IP rules to whitelist or blacklist specific IP addresses.
              </p>
              <Button onClick={() => setCreateDialogOpen(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create First IP Rule
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>IP Address</TableHead>
                  <TableHead>Range</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Reason</TableHead>
                  <TableHead>Expires</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {ipRules.map((ipRule) => (
                  <TableRow key={ipRule.id}>
                    <TableCell className="font-mono">{ipRule.ip_address}</TableCell>
                    <TableCell className="font-mono text-muted-foreground">
                      {ipRule.ip_range || '-'}
                    </TableCell>
                    <TableCell>{getRuleTypeBadge(ipRule.rule_type)}</TableCell>
                    <TableCell className="max-w-[150px] truncate">
                      {ipRule.reason || '-'}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {formatDate(ipRule.expires_at)}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {formatDate(ipRule.created_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        onClick={() => handleDeleteClick(ipRule)}
                        title="Delete IP rule"
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Create IP Rule Dialog */}
      <IpRuleDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        appId={appId}
        onIpRuleCreated={handleIpRuleCreated}
      />

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete IP Rule"
        description={`Are you sure you want to delete the IP rule for ${selectedIpRule?.ip_address}? This action cannot be undone.`}
        confirmText="Delete"
        variant="destructive"
        isLoading={isDeleting}
        onConfirm={handleDeleteIpRule}
      />
    </>
  );
}

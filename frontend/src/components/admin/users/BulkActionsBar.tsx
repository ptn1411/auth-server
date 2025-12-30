import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { BulkRoleAssignmentDialog } from './BulkRoleAssignmentDialog';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import {
  Download,
  Upload,
  Shield,
  X,
  Users,
} from 'lucide-react';

interface BulkActionsBarProps {
  selectedCount: number;
  onClearSelection: () => void;
}

export function BulkActionsBar({ selectedCount, onClearSelection }: BulkActionsBarProps) {
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [roleDialogOpen, setRoleDialogOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const { exportUsers, importUsers, selectedUserIds, clearSelection } = useAdminStore();

  const handleExport = async () => {
    setIsProcessing(true);
    try {
      const users = await exportUsers();
      
      // Create and download JSON file
      const blob = new Blob([JSON.stringify(users, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `users-export-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      
      toast.success(`Exported ${users.length} users`);
      setExportDialogOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to export users');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleImportClick = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      setIsProcessing(true);
      try {
        const text = await file.text();
        const users = JSON.parse(text);
        
        if (!Array.isArray(users)) {
          throw new Error('Invalid file format: expected an array of users');
        }
        
        const imported = await importUsers(users);
        toast.success(`Imported ${imported} users`);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to import users');
      } finally {
        setIsProcessing(false);
      }
    };
    input.click();
  };

  if (selectedCount === 0) {
    return (
      <Card>
        <CardContent className="py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <span className="text-sm text-muted-foreground">
                Select users to perform bulk actions
              </span>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setExportDialogOpen(true)}
              >
                <Download className="h-4 w-4 mr-2" />
                Export All
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleImportClick}
                disabled={isProcessing}
              >
                <Upload className="h-4 w-4 mr-2" />
                Import
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
      <Card className="border-primary">
        <CardContent className="py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <Users className="h-4 w-4 text-primary" />
                <span className="text-sm font-medium">
                  {selectedCount} user{selectedCount !== 1 ? 's' : ''} selected
                </span>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={onClearSelection}
              >
                <X className="h-4 w-4 mr-1" />
                Clear
              </Button>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={isProcessing}
                onClick={() => setRoleDialogOpen(true)}
              >
                <Shield className="h-4 w-4 mr-2" />
                Assign Role
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setExportDialogOpen(true)}
              >
                <Download className="h-4 w-4 mr-2" />
                Export All
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleImportClick}
                disabled={isProcessing}
              >
                <Upload className="h-4 w-4 mr-2" />
                Import
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Export Dialog */}
      <ConfirmDialog
        open={exportDialogOpen}
        onOpenChange={setExportDialogOpen}
        title="Export Users"
        description="This will export all users to a JSON file. The export includes user details but excludes sensitive information like passwords."
        confirmText="Export"
        isLoading={isProcessing}
        onConfirm={handleExport}
      />

      {/* Bulk Role Assignment Dialog */}
      <BulkRoleAssignmentDialog
        open={roleDialogOpen}
        onOpenChange={setRoleDialogOpen}
        selectedUserIds={selectedUserIds}
        onSuccess={() => {
          clearSelection();
        }}
      />
    </>
  );
}

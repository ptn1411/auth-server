import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  User,
  Mail,
  Calendar,
  Shield,
  ShieldOff,
  Lock,
  CheckCircle,
  XCircle,
  Smartphone,
  Edit,
  UserX,
  UserCheck,
  Unlock,
  Trash2,
  Loader2,
} from 'lucide-react';
import type { AdminUserDetail } from '@/lib/auth-client';

interface UserDetailCardProps {
  user: AdminUserDetail;
  isLoading?: boolean;
  onEdit: () => void;
  onActivate: () => void;
  onDeactivate: () => void;
  onUnlock: () => void;
  onDelete: () => void;
}

export function UserDetailCard({
  user,
  isLoading = false,
  onEdit,
  onActivate,
  onDeactivate,
  onUnlock,
  onDelete,
}: UserDetailCardProps) {
  const isUserLocked = user.locked_until && new Date(user.locked_until) > new Date();

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <User className="h-5 w-5" />
              User Details
            </CardTitle>
            <CardDescription>
              View and manage user information
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={onEdit}>
              <Edit className="h-4 w-4 mr-2" />
              Edit
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Basic Info */}
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Mail className="h-4 w-4" />
              Email
            </div>
            <p className="font-medium">{user.email}</p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Calendar className="h-4 w-4" />
              Created
            </div>
            <p className="font-medium">{formatDate(user.created_at)}</p>
          </div>
        </div>

        {/* Status Badges */}
        <div className="space-y-2">
          <h4 className="text-sm font-medium text-muted-foreground">Status</h4>
          <div className="flex flex-wrap gap-2">
            {/* Active Status */}
            {user.is_active ? (
              <Badge variant="default" className="flex items-center gap-1">
                <CheckCircle className="h-3 w-3" />
                Active
              </Badge>
            ) : (
              <Badge variant="secondary" className="flex items-center gap-1">
                <XCircle className="h-3 w-3" />
                Inactive
              </Badge>
            )}

            {/* Locked Status */}
            {isUserLocked && (
              <Badge variant="destructive" className="flex items-center gap-1">
                <Lock className="h-3 w-3" />
                Locked until {formatDate(user.locked_until!)}
              </Badge>
            )}

            {/* Admin Status */}
            {user.is_system_admin ? (
              <Badge variant="outline" className="flex items-center gap-1">
                <Shield className="h-3 w-3" />
                System Admin
              </Badge>
            ) : (
              <Badge variant="outline" className="flex items-center gap-1">
                <ShieldOff className="h-3 w-3" />
                Regular User
              </Badge>
            )}

            {/* Email Verified */}
            {user.email_verified ? (
              <Badge variant="outline" className="flex items-center gap-1">
                <CheckCircle className="h-3 w-3" />
                Email Verified
              </Badge>
            ) : (
              <Badge variant="secondary" className="flex items-center gap-1">
                <XCircle className="h-3 w-3" />
                Email Not Verified
              </Badge>
            )}

            {/* MFA Status */}
            {user.mfa_enabled ? (
              <Badge variant="outline" className="flex items-center gap-1">
                <Smartphone className="h-3 w-3" />
                MFA Enabled
              </Badge>
            ) : (
              <Badge variant="secondary" className="flex items-center gap-1">
                <Smartphone className="h-3 w-3" />
                MFA Disabled
              </Badge>
            )}
          </div>
        </div>

        {/* Security Info */}
        <div className="space-y-2">
          <h4 className="text-sm font-medium text-muted-foreground">Security</h4>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-1">
              <p className="text-sm text-muted-foreground">Failed Login Attempts</p>
              <p className="font-medium">
                {user.failed_login_attempts}
                {user.failed_login_attempts > 0 && (
                  <span className="text-destructive ml-1">
                    ({user.failed_login_attempts >= 5 ? 'Account may be locked' : 'Warning'})
                  </span>
                )}
              </p>
            </div>
            <div className="space-y-1">
              <p className="text-sm text-muted-foreground">Last Updated</p>
              <p className="font-medium">{formatDate(user.updated_at)}</p>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex flex-wrap gap-2 pt-4 border-t">
          {user.is_active ? (
            <Button variant="outline" onClick={onDeactivate}>
              <UserX className="h-4 w-4 mr-2" />
              Deactivate
            </Button>
          ) : (
            <Button variant="outline" onClick={onActivate}>
              <UserCheck className="h-4 w-4 mr-2" />
              Activate
            </Button>
          )}
          
          {isUserLocked && (
            <Button variant="outline" onClick={onUnlock}>
              <Unlock className="h-4 w-4 mr-2" />
              Unlock Account
            </Button>
          )}
          
          <Button variant="destructive" onClick={onDelete}>
            <Trash2 className="h-4 w-4 mr-2" />
            Delete User
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

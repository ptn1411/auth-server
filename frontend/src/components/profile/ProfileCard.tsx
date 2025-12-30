import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { useAuthStore } from '@/stores/authStore';
import { User, Mail, Shield, Calendar } from 'lucide-react';

export function ProfileCard() {
  const { user } = useAuthStore();

  if (!user) return null;

  const getInitials = (email: string) => {
    return email.substring(0, 2).toUpperCase();
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <User className="h-5 w-5" />
          Profile Information
        </CardTitle>
        <CardDescription>Your account details</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-6 sm:flex-row sm:items-start">
          <Avatar className="h-20 w-20">
            <AvatarFallback className="text-2xl">
              {getInitials(user.email)}
            </AvatarFallback>
          </Avatar>
          
          <div className="flex-1 space-y-4">
            <div className="flex items-center gap-2">
              <Mail className="h-4 w-4 text-muted-foreground" />
              <span className="font-medium">{user.email}</span>
              <Badge variant={user.email_verified ? 'default' : 'destructive'}>
                {user.email_verified ? 'Verified' : 'Not Verified'}
              </Badge>
            </div>
            
            <div className="flex items-center gap-2">
              <Shield className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">MFA Status:</span>
              <Badge variant={user.mfa_enabled ? 'default' : 'secondary'}>
                {user.mfa_enabled ? 'Enabled' : 'Disabled'}
              </Badge>
            </div>
            
            <div className="flex items-center gap-2">
              <Calendar className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">
                Member since {new Date(user.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

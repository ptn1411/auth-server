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
      <CardHeader className="pb-3 sm:pb-6">
        <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
          <User className="h-4 w-4 sm:h-5 sm:w-5" />
          Profile Information
        </CardTitle>
        <CardDescription className="text-xs sm:text-sm">Your account details</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-4 sm:gap-6 sm:flex-row sm:items-start">
          <Avatar className="h-16 w-16 sm:h-20 sm:w-20 mx-auto sm:mx-0">
            <AvatarFallback className="text-xl sm:text-2xl">
              {getInitials(user.email)}
            </AvatarFallback>
          </Avatar>
          
          <div className="flex-1 space-y-3 sm:space-y-4">
            <div className="flex flex-col sm:flex-row sm:items-center gap-1 sm:gap-2">
              <div className="flex items-center gap-2">
                <Mail className="h-4 w-4 text-muted-foreground" />
                <span className="font-medium text-sm sm:text-base break-all">{user.email}</span>
              </div>
              <Badge variant={user.email_verified ? 'default' : 'destructive'} className="w-fit text-xs">
                {user.email_verified ? 'Verified' : 'Not Verified'}
              </Badge>
            </div>
            
            <div className="flex items-center gap-2 flex-wrap">
              <Shield className="h-4 w-4 text-muted-foreground" />
              <span className="text-xs sm:text-sm text-muted-foreground">MFA Status:</span>
              <Badge variant={user.mfa_enabled ? 'default' : 'secondary'} className="text-xs">
                {user.mfa_enabled ? 'Enabled' : 'Disabled'}
              </Badge>
            </div>
            
            <div className="flex items-center gap-2">
              <Calendar className="h-4 w-4 text-muted-foreground" />
              <span className="text-xs sm:text-sm text-muted-foreground">
                Member since {new Date(user.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

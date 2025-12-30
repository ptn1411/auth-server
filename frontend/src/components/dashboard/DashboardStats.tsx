import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useAuthStore } from '@/stores/authStore';
import { Shield, Mail, Key, Clock } from 'lucide-react';

export function DashboardStats() {
  const { user } = useAuthStore();

  if (!user) return null;

  const stats = [
    {
      title: 'Email Status',
      value: user.email_verified ? 'Verified' : 'Not Verified',
      icon: Mail,
      variant: user.email_verified ? 'default' : 'destructive',
    },
    {
      title: 'MFA Status',
      value: user.mfa_enabled ? 'Enabled' : 'Disabled',
      icon: Shield,
      variant: user.mfa_enabled ? 'default' : 'secondary',
    },
    {
      title: 'Account Status',
      value: user.is_active ? 'Active' : 'Inactive',
      icon: Key,
      variant: user.is_active ? 'default' : 'destructive',
    },
    {
      title: 'Member Since',
      value: new Date(user.created_at).toLocaleDateString(),
      icon: Clock,
      variant: 'outline',
    },
  ] as const;

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {stats.map((stat) => (
        <Card key={stat.title}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">{stat.title}</CardTitle>
            <stat.icon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <Badge variant={stat.variant}>{stat.value}</Badge>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

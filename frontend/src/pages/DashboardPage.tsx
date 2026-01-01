import { Link } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useAuthStore } from '@/stores/authStore';
import { DashboardStats } from '@/components/dashboard/DashboardStats';
import { RecentActivity } from '@/components/dashboard/RecentActivity';
import { User, Shield, Key, Settings } from 'lucide-react';

export function DashboardPage() {
  const { user } = useAuthStore();

  const quickActions = [
    {
      title: 'Profile',
      description: 'View and update your profile',
      icon: User,
      href: '/profile',
    },
    {
      title: 'Security',
      description: 'Manage MFA and passkeys',
      icon: Shield,
      href: '/security',
    },
    {
      title: 'Sessions',
      description: 'View active sessions',
      icon: Key,
      href: '/sessions',
    },
    {
      title: 'Audit Logs',
      description: 'View account activity',
      icon: Settings,
      href: '/audit-logs',
    },
  ];

  return (
    <div className="space-y-4 sm:space-y-6">
      <div>
        <h1 className="text-2xl sm:text-3xl font-bold">Dashboard</h1>
        <p className="text-sm sm:text-base text-muted-foreground">
          Welcome back{user?.email ? `, ${user.email}` : ''}
        </p>
      </div>

      {/* User Profile Summary */}
      <Card>
        <CardHeader className="pb-3 sm:pb-6">
          <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
            <User className="h-4 w-4 sm:h-5 sm:w-5" />
            Profile Summary
          </CardTitle>
          <CardDescription className="text-xs sm:text-sm">Your account information at a glance</CardDescription>
        </CardHeader>
        <CardContent>
          {user && (
            <div className="flex flex-col gap-4">
              <div className="space-y-2">
                <p className="text-base sm:text-lg font-medium break-all">{user.email}</p>
                <div className="flex flex-wrap gap-2">
                  <Badge variant={user.email_verified ? 'default' : 'destructive'} className="text-xs">
                    {user.email_verified ? 'Email Verified' : 'Email Not Verified'}
                  </Badge>
                  <Badge variant={user.mfa_enabled ? 'default' : 'secondary'} className="text-xs">
                    {user.mfa_enabled ? 'MFA Enabled' : 'MFA Disabled'}
                  </Badge>
                </div>
              </div>
              <Button asChild variant="outline" className="w-full sm:w-auto">
                <Link to="/profile">Edit Profile</Link>
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Stats */}
      <DashboardStats />

      {/* Quick Actions */}
      <div>
        <h2 className="text-lg sm:text-xl font-semibold mb-3 sm:mb-4">Quick Actions</h2>
        <div className="grid gap-3 sm:gap-4 grid-cols-2 lg:grid-cols-4">
          {quickActions.map((action) => (
            <Card key={action.title} className="hover:bg-accent/50 transition-colors">
              <Link to={action.href}>
                <CardHeader className="p-3 sm:p-6">
                  <CardTitle className="flex items-center gap-2 text-sm sm:text-base">
                    <action.icon className="h-4 w-4" />
                    {action.title}
                  </CardTitle>
                  <CardDescription className="text-xs sm:text-sm hidden sm:block">{action.description}</CardDescription>
                </CardHeader>
              </Link>
            </Card>
          ))}
        </div>
      </div>

      {/* Recent Activity */}
      <RecentActivity />
    </div>
  );
}

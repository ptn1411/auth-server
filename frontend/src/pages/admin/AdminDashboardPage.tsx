import { useEffect, useMemo } from 'react';
import { Link } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAdminStore } from '@/stores/adminStore';
import { useAdminScopesStore } from '@/stores/adminScopesStore';
import { useAuthStore } from '@/stores/authStore';
import {
  Users,
  Package,
  FileText,
  Activity,
  UserCheck,
  UserX,
  ShieldAlert,
  KeyRound,
  Globe,
} from 'lucide-react';

export function AdminDashboardPage() {
  const { user } = useAuthStore();
  const { fetchUsers, fetchApps, fetchIpRules, fetchAuditLogs, users, apps, ipRules, auditLogs, isLoading } = useAdminStore();
  const { fetchScopes, total: totalScopes } = useAdminScopesStore();

  useEffect(() => {
    Promise.all([
      fetchUsers({ page: 1, limit: 100 }),
      fetchApps({ page: 1, limit: 1 }),
      fetchIpRules(),
      fetchAuditLogs({ page: 1, limit: 1 }),
      fetchScopes(1, 1),
    ]).catch(console.error);
  }, [fetchUsers, fetchApps, fetchIpRules, fetchAuditLogs, fetchScopes]);

  const stats = useMemo(() => {
    const activeCount = users?.data.filter(u => u.is_active).length ?? 0;
    const totalUsersCount = users?.total ?? 0;
    
    return {
      totalUsers: totalUsersCount,
      activeUsers: activeCount,
      inactiveUsers: totalUsersCount - activeCount,
      totalApps: apps?.total ?? 0,
      totalIpRules: ipRules.length,
      totalScopes: totalScopes,
      recentAuditLogs: auditLogs?.total ?? 0,
    };
  }, [users, apps, ipRules, auditLogs, totalScopes]);

  const statCards = [
    {
      title: 'Total Users',
      value: stats.totalUsers,
      icon: Users,
      description: 'Registered users',
      href: '/admin/users',
      color: 'text-blue-500',
    },
    {
      title: 'Applications',
      value: stats.totalApps,
      icon: Package,
      description: 'Registered apps',
      href: '/admin/apps',
      color: 'text-purple-500',
    },
    {
      title: 'OAuth Scopes',
      value: stats.totalScopes,
      icon: KeyRound,
      description: 'Authorization scopes',
      href: '/admin/scopes',
      color: 'text-orange-500',
    },
    {
      title: 'IP Rules',
      value: stats.totalIpRules,
      icon: Globe,
      description: 'Access control rules',
      href: '/admin/ip-rules',
      color: 'text-green-500',
    },
  ];

  const quickLinks = [
    {
      title: 'User Management',
      description: 'View, search, and manage users',
      icon: Users,
      href: '/admin/users',
    },
    {
      title: 'App Management',
      description: 'Manage registered applications',
      icon: Package,
      href: '/admin/apps',
    },
    {
      title: 'OAuth Scopes',
      description: 'Manage authorization scopes',
      icon: KeyRound,
      href: '/admin/scopes',
    },
    {
      title: 'IP Rules',
      description: 'Manage IP access rules',
      icon: Globe,
      href: '/admin/ip-rules',
    },
    {
      title: 'Audit Logs',
      description: 'View security events',
      icon: FileText,
      href: '/admin/audit-logs',
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Admin Dashboard</h1>
        <p className="text-muted-foreground">
          Welcome back{user?.email ? `, ${user.email}` : ''}
        </p>
      </div>

      {/* Stats */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {statCards.map((stat) => (
          <Link key={stat.title} to={stat.href}>
            <Card className="hover:bg-accent/50 transition-colors cursor-pointer h-full">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{stat.title}</CardTitle>
                <stat.icon className={`h-4 w-4 ${stat.color}`} />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {isLoading ? '...' : stat.value.toLocaleString()}
                </div>
                <p className="text-xs text-muted-foreground">{stat.description}</p>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>

      {/* System Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            System Overview
          </CardTitle>
          <CardDescription>User status and activity summary</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="flex items-center gap-3 p-4 rounded-lg bg-green-500/10 border border-green-500/20">
              <UserCheck className="h-8 w-8 text-green-500" />
              <div>
                <p className="text-sm font-medium text-muted-foreground">Active Users</p>
                <p className="text-2xl font-bold">{isLoading ? '...' : stats.activeUsers}</p>
              </div>
            </div>
            <div className="flex items-center gap-3 p-4 rounded-lg bg-red-500/10 border border-red-500/20">
              <UserX className="h-8 w-8 text-red-500" />
              <div>
                <p className="text-sm font-medium text-muted-foreground">Inactive Users</p>
                <p className="text-2xl font-bold">{isLoading ? '...' : stats.inactiveUsers}</p>
              </div>
            </div>
            <div className="flex items-center gap-3 p-4 rounded-lg bg-blue-500/10 border border-blue-500/20">
              <ShieldAlert className="h-8 w-8 text-blue-500" />
              <div>
                <p className="text-sm font-medium text-muted-foreground">Audit Events</p>
                <p className="text-2xl font-bold">{isLoading ? '...' : stats.recentAuditLogs}</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Quick Links */}
      <div>
        <h2 className="text-lg font-semibold mb-4">Quick Access</h2>
        <div className="grid gap-3 md:grid-cols-3 lg:grid-cols-5">
          {quickLinks.map((link) => (
            <Link key={link.title} to={link.href}>
              <Card className="hover:bg-accent/50 transition-colors cursor-pointer h-full">
                <CardHeader className="p-4">
                  <CardTitle className="flex items-center gap-2 text-sm">
                    <link.icon className="h-4 w-4" />
                    {link.title}
                  </CardTitle>
                  <CardDescription className="text-xs">{link.description}</CardDescription>
                </CardHeader>
              </Card>
            </Link>
          ))}
        </div>
      </div>
    </div>
  );
}

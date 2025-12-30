import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAdminStore } from '@/stores/adminStore';
import { useAuthStore } from '@/stores/authStore';
import {
  Users,
  Package,
  FileText,
  Shield,
  Activity,
  UserCheck,
  UserX,
  ShieldAlert,
} from 'lucide-react';

interface SystemStats {
  totalUsers: number;
  activeUsers: number;
  inactiveUsers: number;
  totalApps: number;
  totalIpRules: number;
  recentAuditLogs: number;
}

export function AdminDashboardPage() {
  const { user } = useAuthStore();
  const { fetchUsers, fetchApps, fetchIpRules, fetchAuditLogs, users, apps, ipRules, auditLogs, isLoading } = useAdminStore();
  const [stats, setStats] = useState<SystemStats>({
    totalUsers: 0,
    activeUsers: 0,
    inactiveUsers: 0,
    totalApps: 0,
    totalIpRules: 0,
    recentAuditLogs: 0,
  });

  useEffect(() => {
    // Fetch data for stats
    Promise.all([
      fetchUsers({ page: 1, limit: 1 }),
      fetchApps({ page: 1, limit: 1 }),
      fetchIpRules(),
      fetchAuditLogs({ page: 1, limit: 1 }),
    ]).catch(console.error);
  }, [fetchUsers, fetchApps, fetchIpRules, fetchAuditLogs]);

  useEffect(() => {
    // Calculate stats from fetched data
    const activeCount = users?.data.filter(u => u.is_active).length ?? 0;
    const totalUsersCount = users?.total ?? 0;
    
    setStats({
      totalUsers: totalUsersCount,
      activeUsers: activeCount,
      inactiveUsers: totalUsersCount - activeCount,
      totalApps: apps?.total ?? 0,
      totalIpRules: ipRules.length,
      recentAuditLogs: auditLogs?.total ?? 0,
    });
  }, [users, apps, ipRules, auditLogs]);

  const statCards = [
    {
      title: 'Total Users',
      value: stats.totalUsers,
      icon: Users,
      description: 'Registered users in the system',
      href: '/admin/users',
    },
    {
      title: 'Active Users',
      value: stats.activeUsers,
      icon: UserCheck,
      description: 'Users with active accounts',
      href: '/admin/users',
    },
    {
      title: 'Total Apps',
      value: stats.totalApps,
      icon: Package,
      description: 'Registered applications',
      href: '/admin/apps',
    },
    {
      title: 'IP Rules',
      value: stats.totalIpRules,
      icon: Shield,
      description: 'Active IP whitelist/blacklist rules',
      href: '/admin/ip-rules',
    },
  ];

  const quickLinks = [
    {
      title: 'User Management',
      description: 'View, search, and manage all users',
      icon: Users,
      href: '/admin/users',
    },
    {
      title: 'App Management',
      description: 'Manage all registered applications',
      icon: Package,
      href: '/admin/apps',
    },
    {
      title: 'Audit Logs',
      description: 'View system-wide security events',
      icon: FileText,
      href: '/admin/audit-logs',
    },
    {
      title: 'IP Rules',
      description: 'Manage global IP access rules',
      icon: Shield,
      href: '/admin/ip-rules',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Admin Dashboard</h1>
        <p className="text-muted-foreground">
          Welcome back{user?.email ? `, ${user.email}` : ''}. Here's an overview of your system.
        </p>
      </div>

      {/* System Stats */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {statCards.map((stat) => (
          <Link key={stat.title} to={stat.href}>
            <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{stat.title}</CardTitle>
                <stat.icon className="h-4 w-4 text-muted-foreground" />
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

      {/* Quick Links */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Quick Links</h2>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {quickLinks.map((link) => (
            <Card key={link.title} className="hover:bg-accent/50 transition-colors">
              <Link to={link.href}>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <link.icon className="h-4 w-4" />
                    {link.title}
                  </CardTitle>
                  <CardDescription>{link.description}</CardDescription>
                </CardHeader>
              </Link>
            </Card>
          ))}
        </div>
      </div>

      {/* System Health Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            System Overview
          </CardTitle>
          <CardDescription>Current system status and recent activity</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="flex items-center gap-3 p-3 rounded-lg bg-muted/50">
              <UserCheck className="h-8 w-8 text-green-500" />
              <div>
                <p className="text-sm font-medium">Active Users</p>
                <p className="text-2xl font-bold">{isLoading ? '...' : stats.activeUsers}</p>
              </div>
            </div>
            <div className="flex items-center gap-3 p-3 rounded-lg bg-muted/50">
              <UserX className="h-8 w-8 text-red-500" />
              <div>
                <p className="text-sm font-medium">Inactive Users</p>
                <p className="text-2xl font-bold">{isLoading ? '...' : stats.inactiveUsers}</p>
              </div>
            </div>
            <div className="flex items-center gap-3 p-3 rounded-lg bg-muted/50">
              <ShieldAlert className="h-8 w-8 text-blue-500" />
              <div>
                <p className="text-sm font-medium">Audit Events</p>
                <p className="text-2xl font-bold">{isLoading ? '...' : stats.recentAuditLogs}</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

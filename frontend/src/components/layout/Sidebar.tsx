import { NavLink } from 'react-router-dom';
import { cn } from '@/lib/utils';
import {
  LayoutDashboard,
  User,
  Shield,
  Monitor,
  FileText,
  X,
  Package,
  Link2,
  Settings,
  Users,
  Globe,
  Key,
  KeyRound,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useAuthStore } from '@/stores/authStore';

interface SidebarProps {
  open?: boolean;
  onClose?: () => void;
}

const navItems = [
  {
    title: 'Dashboard',
    href: '/dashboard',
    icon: LayoutDashboard,
  },
  {
    title: 'Profile',
    href: '/profile',
    icon: User,
  },
  {
    title: 'Security',
    href: '/security',
    icon: Shield,
  },
  {
    title: 'Sessions',
    href: '/sessions',
    icon: Monitor,
  },
  {
    title: 'Audit Logs',
    href: '/audit-logs',
    icon: FileText,
  },
  {
    title: 'My Apps',
    href: '/apps',
    icon: Package,
  },
  {
    title: 'Connected Apps',
    href: '/connected-apps',
    icon: Link2,
  },
  {
    title: 'OAuth Clients',
    href: '/oauth-clients',
    icon: Key,
  },
];

const adminNavItems = [
  {
    title: 'Admin Dashboard',
    href: '/admin',
    icon: Settings,
  },
  {
    title: 'Users',
    href: '/admin/users',
    icon: Users,
  },
  {
    title: 'Apps',
    href: '/admin/apps',
    icon: Package,
  },
  {
    title: 'OAuth Scopes',
    href: '/admin/scopes',
    icon: KeyRound,
  },
  {
    title: 'Audit Logs',
    href: '/admin/audit-logs',
    icon: FileText,
  },
  {
    title: 'IP Rules',
    href: '/admin/ip-rules',
    icon: Globe,
  },
];

export function Sidebar({ open, onClose }: SidebarProps) {
  const { user } = useAuthStore();
  const isAdmin = user?.is_system_admin ?? false;

  return (
    <>
      {/* Mobile overlay */}
      {open && (
        <div
          className="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm md:hidden"
          onClick={onClose}
        />
      )}

      {/* Sidebar */}
      <aside
        className={cn(
          'fixed top-14 z-40 h-[calc(100vh-3.5rem)] w-64 border-r bg-background transition-transform duration-300 ease-in-out md:sticky md:translate-x-0',
          open ? 'translate-x-0' : '-translate-x-full'
        )}
      >
        {/* Mobile close button */}
        <div className="flex items-center justify-end p-2 md:hidden">
          <Button variant="ghost" size="icon" onClick={onClose}>
            <X className="h-5 w-5" />
            <span className="sr-only">Close menu</span>
          </Button>
        </div>

        {/* Navigation */}
        <nav className="flex flex-col gap-1 p-4">
          {navItems.map((item) => (
            <NavLink
              key={item.href}
              to={item.href}
              onClick={onClose}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                )
              }
            >
              <item.icon className="h-4 w-4" />
              {item.title}
            </NavLink>
          ))}

          {/* Admin Section */}
          {isAdmin && (
            <>
              <div className="my-4 border-t" />
              <div className="px-3 py-2 text-xs font-semibold uppercase text-muted-foreground">
                Admin
              </div>
              {adminNavItems.map((item) => (
                <NavLink
                  key={item.href}
                  to={item.href}
                  onClick={onClose}
                  className={({ isActive }) =>
                    cn(
                      'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                      isActive
                        ? 'bg-primary text-primary-foreground'
                        : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                    )
                  }
                >
                  <item.icon className="h-4 w-4" />
                  {item.title}
                </NavLink>
              ))}
            </>
          )}
        </nav>
      </aside>
    </>
  );
}

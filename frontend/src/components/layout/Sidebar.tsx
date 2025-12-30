import { NavLink } from 'react-router-dom';
import { cn } from '@/lib/utils';
import {
  LayoutDashboard,
  User,
  Shield,
  Monitor,
  FileText,
  X,
} from 'lucide-react';
import { Button } from '@/components/ui/button';

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
];

export function Sidebar({ open, onClose }: SidebarProps) {
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
        </nav>
      </aside>
    </>
  );
}

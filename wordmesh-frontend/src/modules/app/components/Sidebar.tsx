/**
 * 侧边栏组件
 */

'use client';

import { usePathname, useRouter } from 'next/navigation';
import Link from 'next/link';
import { BookOpen, Home, Search, Network, Settings, LogOut } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useAuth } from '@/modules/auth';

const navigation = [
  { name: '仪表板', href: '/dashboard', icon: Home },
  { name: '我的词网', href: '/words', icon: BookOpen },
  { name: '搜索', href: '/search', icon: Search },
  { name: '网络视图', href: '/network', icon: Network },
  { name: '设置', href: '/settings', icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();
  const router = useRouter();
  const { logout, user } = useAuth();

  const handleLogout = () => {
    logout();
  };

  return (
    <aside className="w-64 border-r bg-card flex flex-col">
      {/* Logo */}
      <div className="flex items-center gap-3 p-6 border-b">
        <div className="bg-primary text-primary-foreground flex size-10 items-center justify-center rounded-lg">
          <BookOpen className="size-5" />
        </div>
        <div>
          <h1 className="text-lg font-bold">WordMesh</h1>
          <p className="text-xs text-muted-foreground">单词知识网络</p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-1">
        {navigation.map((item) => {
          const isActive = pathname === item.href || pathname?.startsWith(item.href + '/');
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200',
                isActive
                  ? 'bg-primary text-primary-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )}
            >
              <item.icon className="size-4" />
              {item.name}
            </Link>
          );
        })}
      </nav>

      {/* User Info & Logout */}
      <div className="p-4 border-t space-y-2">
        {user && (
          <div className="px-3 py-2 text-sm">
            <p className="font-medium">{user.username}</p>
            <p className="text-xs text-muted-foreground">用户 ID: {user.id}</p>
          </div>
        )}
        <Button
          variant="ghost"
          className="w-full justify-start text-muted-foreground hover:text-destructive"
          onClick={handleLogout}
        >
          <LogOut className="size-4 mr-2" />
          退出登录
        </Button>
      </div>
    </aside>
  );
}


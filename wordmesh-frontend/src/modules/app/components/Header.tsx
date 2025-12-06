/**
 * 顶部导航栏组件
 */

'use client';

import { Bell, Menu } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useAuth } from '@/modules/auth';

export function Header() {
  const { user } = useAuth();

  return (
    <header className="h-16 border-b bg-card flex items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" className="lg:hidden">
          <Menu className="size-5" />
        </Button>
        <h2 className="text-lg font-semibold">WordMesh</h2>
      </div>

      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="size-5" />
          <span className="absolute top-1 right-1 size-2 bg-destructive rounded-full" />
        </Button>
        {user && (
          <div className="text-sm">
            <span className="text-muted-foreground">欢迎, </span>
            <span className="font-medium">{user.username}</span>
          </div>
        )}
      </div>
    </header>
  );
}


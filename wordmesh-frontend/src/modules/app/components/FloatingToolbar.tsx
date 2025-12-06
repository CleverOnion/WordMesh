/**
 * 浮动工具栏组件
 * 替换侧边栏，提供主要功能入口
 */

'use client';

import { useState } from 'react';
import { BookOpen, Search, Settings, LogOut, Menu, X, User } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useAuth } from '@/modules/auth';

interface FloatingToolbarProps {
  onOpenWords?: () => void;
  onOpenSearch?: () => void;
  onOpenSettings?: () => void;
  className?: string;
}

export function FloatingToolbar({
  onOpenWords,
  onOpenSearch,
  onOpenSettings,
  className,
}: FloatingToolbarProps) {
  const { logout, user } = useAuth();
  const [isExpanded, setIsExpanded] = useState(false);

  const handleLogout = () => {
    logout();
  };

  return (
    <div
      className={cn(
        'fixed top-2 left-2 sm:top-4 sm:left-4 z-30 flex flex-col gap-2 transition-smooth',
        className
      )}
    >
      {/* 主菜单按钮 */}
      <Button
        variant="default"
        size="icon"
        className={cn(
          'size-10 sm:size-12 rounded-full shadow-lg hover:shadow-xl transition-smooth gpu-accelerated',
          'bg-primary text-primary-foreground hover:bg-primary/90',
          'button-press touch-manipulation'
        )}
        onClick={() => setIsExpanded(!isExpanded)}
        aria-label="打开菜单"
      >
        {isExpanded ? (
          <X className="size-5 transition-transform duration-300" />
        ) : (
          <Menu className="size-5 transition-transform duration-300" />
        )}
      </Button>

      {/* 展开的功能按钮 */}
      <div
        className={cn(
          'flex flex-col gap-2 transition-all duration-300 ease-out',
          isExpanded
            ? 'opacity-100 translate-y-0 pointer-events-auto'
            : 'opacity-0 -translate-y-4 pointer-events-none'
        )}
      >
        {/* 单词管理 */}
        {onOpenWords && (
          <Button
            variant="secondary"
            size="icon"
            className={cn(
              'size-10 sm:size-12 rounded-full shadow-lg hover:shadow-xl transition-smooth gpu-accelerated',
              'bg-card hover:bg-accent border border-border',
              'button-press touch-manipulation'
            )}
            onClick={() => {
              onOpenWords();
              setIsExpanded(false);
            }}
            aria-label="单词管理"
            title="单词管理"
          >
            <BookOpen className="size-5" />
          </Button>
        )}

        {/* 搜索 */}
        {onOpenSearch && (
          <Button
            variant="secondary"
            size="icon"
            className={cn(
              'size-10 sm:size-12 rounded-full shadow-lg hover:shadow-xl transition-smooth gpu-accelerated',
              'bg-card hover:bg-accent border border-border',
              'button-press touch-manipulation'
            )}
            onClick={() => {
              onOpenSearch();
              setIsExpanded(false);
            }}
            aria-label="搜索"
            title="搜索"
          >
            <Search className="size-4 sm:size-5" />
          </Button>
        )}

        {/* 设置 */}
        {onOpenSettings && (
          <Button
            variant="secondary"
            size="icon"
            className={cn(
              'size-10 sm:size-12 rounded-full shadow-lg hover:shadow-xl transition-smooth gpu-accelerated',
              'bg-card hover:bg-accent border border-border',
              'button-press touch-manipulation'
            )}
            onClick={() => {
              onOpenSettings();
              setIsExpanded(false);
            }}
            aria-label="设置"
            title="设置"
          >
            <Settings className="size-4 sm:size-5" />
          </Button>
        )}

        {/* 用户信息 */}
        {user && (
          <div className="flex flex-col gap-2">
            <div className="px-3 py-2 text-xs bg-card/80 backdrop-blur-sm rounded-lg border border-border shadow-sm">
              <p className="font-medium">{user.username}</p>
              <p className="text-muted-foreground">ID: {user.id}</p>
            </div>
            <Button
              variant="secondary"
              size="icon"
              className={cn(
                'size-10 sm:size-12 rounded-full shadow-lg hover:shadow-xl transition-smooth gpu-accelerated',
                'bg-card hover:bg-destructive/10 hover:text-destructive border border-border',
                'button-press touch-manipulation'
              )}
              onClick={handleLogout}
              aria-label="退出登录"
              title="退出登录"
            >
              <LogOut className="size-4 sm:size-5" />
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}


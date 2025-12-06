/**
 * 浮动抽屉组件
 * 支持从不同方向滑入，带有遮罩层和平滑动画
 */

'use client';

import { useEffect, ReactNode } from 'react';
import { X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

export type DrawerDirection = 'left' | 'right' | 'top' | 'bottom';

interface FloatingDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  direction?: DrawerDirection;
  title?: string;
  children: ReactNode;
  className?: string;
  width?: string;
  height?: string;
  showCloseButton?: boolean;
}

export function FloatingDrawer({
  isOpen,
  onClose,
  direction = 'right',
  title,
  children,
  className,
  width,
  height,
  showCloseButton = true,
}: FloatingDrawerProps) {
  // 处理 ESC 键关闭
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      // 防止背景滚动
      document.body.style.overflow = 'hidden';
    }

    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = '';
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const getDrawerStyles = () => {
    const baseStyles = 'fixed z-50 bg-card shadow-2xl transition-smooth gpu-accelerated';
    
    switch (direction) {
      case 'right':
        return cn(
          baseStyles,
          'top-0 right-0 h-full',
          width || 'w-full sm:max-w-md',
          'drawer-slide-in-right',
          className
        );
      case 'left':
        return cn(
          baseStyles,
          'top-0 left-0 h-full',
          width || 'w-full sm:max-w-md',
          'drawer-slide-in-left',
          className
        );
      case 'top':
        return cn(
          baseStyles,
          'top-0 left-0 right-0',
          height || 'h-auto max-h-[80vh]',
          'drawer-slide-in-top',
          className
        );
      case 'bottom':
        return cn(
          baseStyles,
          'bottom-0 left-0 right-0',
          height || 'h-auto max-h-[80vh]',
          'drawer-slide-in-bottom',
          className
        );
      default:
        return baseStyles;
    }
  };

  return (
    <>
      {/* 遮罩层 */}
      <div
        className={cn(
          'fixed inset-0 bg-black/50 backdrop-blur-sm z-40 backdrop-fade-in',
          isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
        )}
        onClick={onClose}
        aria-hidden="true"
      />

      {/* 抽屉内容 */}
      <div
        className={getDrawerStyles()}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? 'drawer-title' : undefined}
      >
        {/* 头部 */}
        {(title || showCloseButton) && (
          <div className="flex items-center justify-between p-6 border-b">
            {title && (
              <h2 id="drawer-title" className="text-xl font-semibold">
                {title}
              </h2>
            )}
            {showCloseButton && (
              <Button
                variant="ghost"
                size="icon"
                onClick={onClose}
                className="ml-auto"
                aria-label="关闭抽屉"
              >
                <X className="size-4" />
              </Button>
            )}
          </div>
        )}

        {/* 内容区域 */}
        <div
          className={cn(
            'overflow-y-auto',
            direction === 'top' || direction === 'bottom' ? 'p-6' : 'p-6'
          )}
          style={{
            maxHeight:
              direction === 'left' || direction === 'right'
                ? 'calc(100vh - 80px)'
                : undefined,
          }}
        >
          {children}
        </div>
      </div>
    </>
  );
}


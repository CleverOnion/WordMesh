/**
 * 应用主布局组件
 * 全屏布局，无侧边栏
 */

'use client';

import { ReactNode } from 'react';

interface AppLayoutProps {
  children: ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="h-screen w-screen overflow-hidden">
      {children}
    </div>
  );
}


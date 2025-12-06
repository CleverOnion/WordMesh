/**
 * 主内容区域组件
 */

'use client';

import { ReactNode } from 'react';

interface MainContentProps {
  children: ReactNode;
  className?: string;
}

export function MainContent({ children, className }: MainContentProps) {
  return (
    <div className={`flex-1 overflow-y-auto bg-background ${className}`}>
      {children}
    </div>
  );
}


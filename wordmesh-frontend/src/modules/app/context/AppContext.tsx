/**
 * 应用全局上下文
 */

'use client';

import { createContext, useContext, ReactNode } from 'react';
import { useAuth } from '@/modules/auth';
import type { UserProfile } from '@/modules/auth';

interface AppContextValue {
  user: UserProfile | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

const AppContext = createContext<AppContextValue | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
  const { user, isAuthenticated, isLoading } = useAuth();

  return (
    <AppContext.Provider value={{ user, isAuthenticated, isLoading }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error('useApp must be used within an AppProvider');
  }
  return context;
}


/**
 * 路由配置
 */

export interface RouteConfig {
  path: string;
  name: string;
  requiresAuth: boolean;
}

export const routes: RouteConfig[] = [
  { path: '/login', name: '登录', requiresAuth: false },
  { path: '/register', name: '注册', requiresAuth: false },
  { path: '/dashboard', name: '仪表板', requiresAuth: true },
  { path: '/words', name: '我的词网', requiresAuth: true },
  { path: '/search', name: '搜索', requiresAuth: true },
  { path: '/network', name: '网络视图', requiresAuth: true },
  { path: '/settings', name: '设置', requiresAuth: true },
];

// 公开路由（不需要认证）
export const publicRoutes = ['/login', '/register'];

// 受保护路由（需要认证）
export function isProtectedRoute(path: string): boolean {
  return !publicRoutes.includes(path);
}


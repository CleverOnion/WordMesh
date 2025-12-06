/**
 * 认证模块类型定义
 */

import { type Timestamp } from '@/shared/types/common.types';

// 注册请求
export interface RegisterRequest {
  username: string;
  password: string;
}

// 登录请求
export interface LoginRequest {
  username: string;
  password: string;
}

// 刷新令牌请求
export interface RefreshRequest {
  refresh_token: string;
}

// 认证令牌响应
export interface AuthTokens {
  access_token: string;
  refresh_token: string | null;
}

// 用户资料
export interface UserProfile {
  id: number;
  username: string;
  created_at: Timestamp;
}

// 认证状态
export interface AuthState {
  isAuthenticated: boolean;
  user: UserProfile | null;
  isLoading: boolean;
}


/**
 * 认证 API
 */

import { apiClient } from '@/shared';
import type {
  RegisterRequest,
  LoginRequest,
  RefreshRequest,
  AuthTokens,
  UserProfile,
} from '../types/auth.types';

// 注册
export async function register(
  data: RegisterRequest
): Promise<{ code: 2000; message: string; data: UserProfile; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<UserProfile>('/auth/register', data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: UserProfile; traceId?: string; timestamp?: number };
}

// 登录
export async function login(
  data: LoginRequest
): Promise<{ code: 2000; message: string; data: AuthTokens; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<AuthTokens>('/auth/login', data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: AuthTokens; traceId?: string; timestamp?: number };
}

// 刷新令牌
export async function refreshToken(
  data: RefreshRequest
): Promise<{ code: 2000; message: string; data: AuthTokens; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<AuthTokens>('/auth/refresh', data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: AuthTokens; traceId?: string; timestamp?: number };
}

// 获取当前用户资料
export async function getProfile(): Promise<{ code: 2000; message: string; data: UserProfile; traceId?: string; timestamp?: number }> {
  const response = await apiClient.get<UserProfile>('/auth/profile');
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: UserProfile; traceId?: string; timestamp?: number };
}


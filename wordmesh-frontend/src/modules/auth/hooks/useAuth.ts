/**
 * 认证 Hook
 * 提供登录、注册、登出、获取用户信息等功能
 */

'use client';

import { useState, useEffect, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import * as authApi from '../api/authApi';
import { TokenStorage } from '../utils/tokenStorage';
import type { AuthState, UserProfile, LoginRequest, RegisterRequest } from '../types/auth.types';
import { getErrorMessage, getFieldErrors } from '@/shared/utils/errorHandler';
import { showErrorToast, showSuccessToast } from '@/shared/utils/toast';

export function useAuth() {
  const router = useRouter();
  const [state, setState] = useState<AuthState>({
    isAuthenticated: false,
    user: null,
    isLoading: true,
  });

  // 检查认证状态
  const checkAuth = useCallback(async () => {
    const token = TokenStorage.getAccessToken();
    if (!token) {
      setState({
        isAuthenticated: false,
        user: null,
        isLoading: false,
      });
      return;
    }

    try {
      const response = await authApi.getProfile();
      setState({
        isAuthenticated: true,
        user: response.data,
        isLoading: false,
      });
    } catch (error) {
      // Token 无效，清除并重置状态
      TokenStorage.clearTokens();
      setState({
        isAuthenticated: false,
        user: null,
        isLoading: false,
      });
    }
  }, []);

  // 初始化时检查认证状态
  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  // 登录
  const login = useCallback(
    async (credentials: LoginRequest) => {
      try {
        const response = await authApi.login(credentials);
        const { access_token, refresh_token } = response.data;

        // 存储 token
        TokenStorage.setAccessToken(access_token);
        if (refresh_token) {
          TokenStorage.setRefreshToken(refresh_token);
        }

        // 获取用户信息
        const profileResponse = await authApi.getProfile();
        setState({
          isAuthenticated: true,
          user: profileResponse.data,
          isLoading: false,
        });

        showSuccessToast('登录成功');
        return { success: true as const };
      } catch (error) {
        showErrorToast(error);
        return {
          success: false as const,
          error: getErrorMessage(error),
          fieldErrors: getFieldErrors(error),
        };
      }
    },
    []
  );

  // 注册
  const register = useCallback(
    async (data: RegisterRequest) => {
      try {
        const response = await authApi.register(data);
        // 注册成功后自动登录
        const loginResult = await login({
          username: data.username,
          password: data.password,
        });

        if (!loginResult.success) {
          return loginResult;
        }

        showSuccessToast('注册成功');
        return { success: true as const, user: response.data };
      } catch (error) {
        showErrorToast(error);
        return {
          success: false as const,
          error: getErrorMessage(error),
          fieldErrors: getFieldErrors(error),
        };
      }
    },
    [login]
  );

  // 登出
  const logout = useCallback(() => {
    TokenStorage.clearTokens();
    setState({
      isAuthenticated: false,
      user: null,
      isLoading: false,
    });
    router.push('/login');
  }, [router]);

  return {
    ...state,
    login,
    register,
    logout,
    checkAuth,
  };
}


/**
 * Token 刷新 Hook
 * 自动处理 access_token 过期时的刷新逻辑
 */

'use client';

import { useCallback } from 'react';
import * as authApi from '../api/authApi';
import { TokenStorage } from '../utils/tokenStorage';
import { ErrorCode } from '@/shared/types/api.types';

export function useTokenRefresh() {
  const refresh = useCallback(async (): Promise<boolean> => {
    const refreshToken = TokenStorage.getRefreshToken();
    if (!refreshToken) {
      return false;
    }

    try {
      const response = await authApi.refreshToken({
        refresh_token: refreshToken,
      });

      const { access_token, refresh_token } = response.data;
      TokenStorage.setAccessToken(access_token);
      if (refresh_token) {
        TokenStorage.setRefreshToken(refresh_token);
      } else {
        // 如果服务端没有返回新的 refresh_token，保留旧的
        // 或者根据业务需求清除
      }

      return true;
    } catch (error) {
      // 刷新失败，清除所有 token
      if (
        error &&
        typeof error === 'object' &&
        'code' in error &&
        (error.code === ErrorCode.TOKEN_EXPIRED ||
          error.code === ErrorCode.TOKEN_INVALID ||
          error.code === ErrorCode.REFRESH_DISABLED)
      ) {
        TokenStorage.clearTokens();
      }
      return false;
    }
  }, []);

  return { refresh };
}


/**
 * 义项 Hook
 * 提供单个义项的 CRUD 操作
 */

'use client';

import { useState, useCallback } from 'react';
import * as senseApi from '../api/senseApi';
import type { AddSenseRequest, UpdateSenseRequest, UserSense } from '../types/sense.types';
import { getErrorMessage, getFieldErrors } from '@/shared/utils/errorHandler';

export function useSense() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 添加义项
  const addSense = useCallback(
    async (userWordId: number, data: AddSenseRequest) => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await senseApi.addSense(userWordId, data);
        return {
          success: true as const,
          data: response.data,
        };
      } catch (err) {
        const errorMessage = getErrorMessage(err);
        setError(errorMessage);
        return {
          success: false as const,
          error: errorMessage,
          fieldErrors: getFieldErrors(err),
        };
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  // 更新义项
  const updateSense = useCallback(
    async (senseId: number, data: UpdateSenseRequest) => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await senseApi.updateSense(senseId, data);
        return {
          success: true as const,
          data: response.data,
        };
      } catch (err) {
        const errorMessage = getErrorMessage(err);
        setError(errorMessage);
        return {
          success: false as const,
          error: errorMessage,
          fieldErrors: getFieldErrors(err),
        };
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  // 删除义项
  const deleteSense = useCallback(async (senseId: number) => {
    setIsLoading(true);
    setError(null);
    try {
      await senseApi.deleteSense(senseId);
      return { success: true as const };
    } catch (err) {
      const errorMessage = getErrorMessage(err);
      setError(errorMessage);
      return {
        success: false as const,
        error: errorMessage,
      };
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    addSense,
    updateSense,
    deleteSense,
    isLoading,
    error,
  };
}


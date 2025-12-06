/**
 * 单词 Hook
 * 提供单个单词的 CRUD 操作
 */

'use client';

import { useState, useCallback } from 'react';
import * as wordApi from '../api/wordApi';
import type { AddWordRequest, UserWordAggregate } from '../types/word.types';
import { getErrorMessage, getFieldErrors } from '@/shared/utils/errorHandler';

export function useWord() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 添加单词到我的词网
  const addWord = useCallback(async (data: AddWordRequest) => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await wordApi.addToMyNetwork(data);
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
  }, []);

  // 从我的词网移除单词
  const removeWord = useCallback(async (userWordId: number) => {
    setIsLoading(true);
    setError(null);
    try {
      await wordApi.removeFromMyNetwork(userWordId);
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
    addWord,
    removeWord,
    isLoading,
    error,
  };
}


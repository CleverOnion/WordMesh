/**
 * 关联 Hook
 * 提供关联的创建和删除操作
 */

'use client';

import { useState, useCallback } from 'react';
import * as associationApi from '../api/associationApi';
import type {
  CreateWordLinkRequest,
  CreateSenseWordLinkRequest,
  DeleteWordLinkRequest,
  DeleteSenseWordLinkRequest,
  WordLinkRecord,
  SenseWordLinkRecord,
} from '../types/association.types';
import { getErrorMessage, getFieldErrors } from '@/shared/utils/errorHandler';

export function useAssociation() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 创建词-词关联
  const createWordLink = useCallback(async (data: CreateWordLinkRequest) => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await associationApi.createWordLink(data);
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

  // 创建义-词关联
  const createSenseWordLink = useCallback(async (data: CreateSenseWordLinkRequest) => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await associationApi.createSenseWordLink(data);
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

  // 删除词-词关联
  const deleteWordLink = useCallback(async (data: DeleteWordLinkRequest) => {
    setIsLoading(true);
    setError(null);
    try {
      await associationApi.deleteWordLink(data);
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

  // 删除义-词关联
  const deleteSenseWordLink = useCallback(async (data: DeleteSenseWordLinkRequest) => {
    setIsLoading(true);
    setError(null);
    try {
      await associationApi.deleteSenseWordLink(data);
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
    createWordLink,
    createSenseWordLink,
    deleteWordLink,
    deleteSenseWordLink,
    isLoading,
    error,
  };
}


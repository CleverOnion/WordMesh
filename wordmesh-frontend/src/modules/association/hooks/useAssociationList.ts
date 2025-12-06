/**
 * 关联列表 Hook
 * 提供关联列表的查询和筛选功能
 */

'use client';

import { useState, useCallback, useEffect } from 'react';
import * as associationApi from '../api/associationApi';
import type {
  ListLinksRequest,
  WordLinkRecord,
  SenseWordLinkRecord,
} from '../types/association.types';
import { getErrorMessage } from '@/shared/utils/errorHandler';
import { PAGINATION_DEFAULTS } from '@/shared/constants/app.constants';

export function useAssociationList(
  endpointType: 'word' | 'sense',
  endpointId: number,
  initialKind?: string
) {
  const [links, setLinks] = useState<(WordLinkRecord | SenseWordLinkRecord)[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [kind, setKind] = useState<string | undefined>(initialKind);

  // 查询关联列表
  const fetchLinks = useCallback(
    async (filterKind?: string) => {
      setIsLoading(true);
      setError(null);

      try {
        const response = await associationApi.listLinks({
          endpoint_type: endpointType,
          endpoint_id: endpointId,
          kind: filterKind || kind,
          limit: PAGINATION_DEFAULTS.PAGE_SIZE,
          offset: 0,
        });

        setLinks(response.data);
        if (filterKind !== undefined) {
          setKind(filterKind);
        }

        return { success: true as const, data: response.data };
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
    },
    [endpointType, endpointId, kind]
  );

  // 刷新列表
  const refresh = useCallback(() => {
    return fetchLinks();
  }, [fetchLinks]);

  // 按类型筛选
  const filterByKind = useCallback(
    (filterKind: string | undefined) => {
      return fetchLinks(filterKind);
    },
    [fetchLinks]
  );

  // 初始加载
  useEffect(() => {
    if (endpointId) {
      fetchLinks();
    }
  }, [endpointId, endpointType]);

  return {
    links,
    isLoading,
    error,
    kind,
    fetchLinks,
    refresh,
    filterByKind,
    setKind,
  };
}


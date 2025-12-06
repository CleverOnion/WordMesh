/**
 * 单词列表 Hook
 * 提供单词列表的搜索和分页功能
 */

'use client';

import { useState, useCallback, useEffect } from 'react';
import * as wordApi from '../api/wordApi';
import type { SearchRequest, SearchResponse, UserWordAggregate } from '../types/word.types';
import { getErrorMessage } from '@/shared/utils/errorHandler';
import { PAGINATION_DEFAULTS } from '@/shared/constants/app.constants';

export function useWordList(initialQuery?: string) {
  const [words, setWords] = useState<UserWordAggregate[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pagination, setPagination] = useState({
    total: 0,
    page: PAGINATION_DEFAULTS.PAGE,
    pageSize: PAGINATION_DEFAULTS.PAGE_SIZE,
    totalPages: 0,
  });
  const [query, setQuery] = useState(initialQuery || '');
  const [scope, setScope] = useState<SearchRequest['scope']>('both');

  // 搜索单词
  const search = useCallback(
    async (searchQuery?: string, searchScope?: SearchRequest['scope']) => {
      setIsLoading(true);
      setError(null);

      const currentQuery = searchQuery !== undefined ? searchQuery : query;
      const currentScope = searchScope !== undefined ? searchScope : scope;

      try {
        const response = await wordApi.searchMyNetwork({
          q: currentQuery,
          scope: currentScope,
          limit: pagination.pageSize,
          offset: (pagination.page - 1) * pagination.pageSize,
        });

        setWords(response.data.items);
        setPagination({
          total: response.data.total,
          page: response.data.page,
          pageSize: response.data.pageSize,
          totalPages: response.data.totalPages,
        });
        setQuery(currentQuery);
        setScope(currentScope);

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
    [query, scope, pagination.page, pagination.pageSize]
  );

  // 刷新列表
  const refresh = useCallback(() => {
    return search();
  }, [search]);

  // 改变页码
  const changePage = useCallback(
    (page: number) => {
      setPagination((prev) => ({ ...prev, page }));
    },
    []
  );

  // 改变每页数量
  const changePageSize = useCallback((pageSize: number) => {
    setPagination((prev) => ({ ...prev, pageSize, page: 1 }));
  }, []);

  // 初始加载和分页变化时重新加载
  useEffect(() => {
    let cancelled = false;

    const loadData = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const response = await wordApi.searchMyNetwork({
          q: query,
          scope: scope,
          limit: pagination.pageSize,
          offset: (pagination.page - 1) * pagination.pageSize,
        });

        if (!cancelled) {
          setWords(response.data.items);
          setPagination({
            total: response.data.total,
            page: response.data.page,
            pageSize: response.data.pageSize,
            totalPages: response.data.totalPages,
          });
        }
      } catch (err) {
        if (!cancelled) {
          const errorMessage = getErrorMessage(err);
          setError(errorMessage);
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    loadData();

    return () => {
      cancelled = true;
    };
  }, [query, scope, pagination.page, pagination.pageSize]);

  return {
    words,
    isLoading,
    error,
    pagination,
    query,
    scope,
    search,
    refresh,
    changePage,
    changePageSize,
    setQuery,
    setScope,
  };
}


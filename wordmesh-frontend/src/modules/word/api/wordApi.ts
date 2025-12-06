/**
 * 单词 API
 */

import { apiClient } from '@/shared';
import type {
  AddWordRequest,
  UserWordAggregate,
  SearchRequest,
  SearchResponse,
} from '../types/word.types';

// 加入我的词网
export async function addToMyNetwork(
  data: AddWordRequest
): Promise<{ code: 2000; message: string; data: UserWordAggregate; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<UserWordAggregate>('/words/my', data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: UserWordAggregate; traceId?: string; timestamp?: number };
}

// 从我的词网移除
export async function removeFromMyNetwork(
  userWordId: number
): Promise<{ code: 2000; message: string; data: null; traceId?: string; timestamp?: number }> {
  const response = await apiClient.delete<null>(`/words/my/${userWordId}`);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: null; traceId?: string; timestamp?: number };
}

// 搜索我的词网
export async function searchMyNetwork(
  params: SearchRequest
): Promise<{ code: 2000; message: string; data: SearchResponse; traceId?: string; timestamp?: number }> {
  const queryParams: Record<string, string | number> = {};
  if (params.q) queryParams.q = params.q;
  if (params.scope) queryParams.scope = params.scope;
  if (params.limit) queryParams.limit = params.limit;
  if (params.offset) queryParams.offset = params.offset;

  const response = await apiClient.get<SearchResponse>('/words/my/search', {
    params: queryParams,
  });
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: SearchResponse; traceId?: string; timestamp?: number };
}


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

  // 后端返回的是数组格式，需要转换为分页格式
  const response = await apiClient.get<UserWordAggregate[]>('/words/my/search', {
    params: queryParams,
  });
  if (response.code !== 2000) {
    throw response;
  }

  // 将数组格式转换为分页格式
  const items = response.data;
  const limit = params.limit || 20;
  const offset = params.offset || 0;
  // 如果返回的数据少于 limit，说明这是最后一页
  // 否则，我们无法确定总数，使用 offset + items.length 作为估算值
  const estimatedTotal = items.length < limit ? offset + items.length : offset + items.length + 1;
  const page = Math.floor(offset / limit) + 1;
  const totalPages = Math.ceil(estimatedTotal / limit);

  const searchResponse: SearchResponse = {
    items,
    total: estimatedTotal,
    page,
    pageSize: limit,
    totalPages,
  };

  return {
    code: 2000,
    message: response.message,
    data: searchResponse,
    traceId: response.traceId,
    timestamp: response.timestamp,
  };
}


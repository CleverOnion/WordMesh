/**
 * 关联网络 API
 */

import { apiClient } from '@/shared';
import type {
  CreateWordLinkRequest,
  CreateSenseWordLinkRequest,
  DeleteWordLinkRequest,
  DeleteSenseWordLinkRequest,
  ListLinksRequest,
  WordLinkRecord,
  SenseWordLinkRecord,
} from '../types/association.types';

// 创建词-词关联
export async function createWordLink(
  data: CreateWordLinkRequest
): Promise<{ code: 2000; message: string; data: WordLinkRecord; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<WordLinkRecord>('/words/associations/word', data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: WordLinkRecord; traceId?: string; timestamp?: number };
}

// 创建义-词关联
export async function createSenseWordLink(
  data: CreateSenseWordLinkRequest
): Promise<{ code: 2000; message: string; data: SenseWordLinkRecord; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<SenseWordLinkRecord>('/words/associations/sense-word', data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: SenseWordLinkRecord; traceId?: string; timestamp?: number };
}

// 删除词-词关联
export async function deleteWordLink(
  data: DeleteWordLinkRequest
): Promise<{ code: 2000; message: string; data: null; traceId?: string; timestamp?: number }> {
  const response = await apiClient.delete<null>('/words/associations/word', { body: data });
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: null; traceId?: string; timestamp?: number };
}

// 删除义-词关联
export async function deleteSenseWordLink(
  data: DeleteSenseWordLinkRequest
): Promise<{ code: 2000; message: string; data: null; traceId?: string; timestamp?: number }> {
  const response = await apiClient.delete<null>('/words/associations/sense-word', { body: data });
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: null; traceId?: string; timestamp?: number };
}

// 查询关联列表
export async function listLinks(
  params: ListLinksRequest
): Promise<{ code: 2000; message: string; data: (WordLinkRecord | SenseWordLinkRecord)[]; traceId?: string; timestamp?: number }> {
  const queryParams: Record<string, string | number> = {
    endpoint_type: params.endpoint_type,
    endpoint_id: params.endpoint_id,
  };
  if (params.kind) queryParams.kind = params.kind;
  if (params.limit) queryParams.limit = params.limit;
  if (params.offset) queryParams.offset = params.offset;

  const response = await apiClient.get<(WordLinkRecord | SenseWordLinkRecord)[]>('/words/associations', {
    params: queryParams,
  });
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: (WordLinkRecord | SenseWordLinkRecord)[]; traceId?: string; timestamp?: number };
}


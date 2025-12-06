/**
 * 义项 API
 */

import { apiClient } from '@/shared';
import type { AddSenseRequest, UpdateSenseRequest, UserSense } from '../types/sense.types';

// 添加义项
export async function addSense(
  userWordId: number,
  data: AddSenseRequest
): Promise<{ code: 2000; message: string; data: UserSense; traceId?: string; timestamp?: number }> {
  const response = await apiClient.post<UserSense>(`/words/my/${userWordId}/senses`, data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: UserSense; traceId?: string; timestamp?: number };
}

// 更新义项
export async function updateSense(
  senseId: number,
  data: UpdateSenseRequest
): Promise<{ code: 2000; message: string; data: UserSense; traceId?: string; timestamp?: number }> {
  const response = await apiClient.patch<UserSense>(`/words/my/senses/${senseId}`, data);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: UserSense; traceId?: string; timestamp?: number };
}

// 删除义项
export async function deleteSense(
  senseId: number
): Promise<{ code: 2000; message: string; data: null; traceId?: string; timestamp?: number }> {
  const response = await apiClient.delete<null>(`/words/my/senses/${senseId}`);
  if (response.code !== 2000) {
    throw response;
  }
  return response as { code: 2000; message: string; data: null; traceId?: string; timestamp?: number };
}


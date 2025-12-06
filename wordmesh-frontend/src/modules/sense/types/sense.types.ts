/**
 * 义项模块类型定义
 */

import { type Timestamp } from '@/shared/types/common.types';

// 个人义项（与 word 模块中的 UserSense 保持一致）
export interface UserSense {
  id: number | null;
  text: string;
  is_primary: boolean;
  sort_order: number;
  note: string | null;
  created_at: Timestamp;
}

// 添加义项请求
export interface AddSenseRequest {
  text: string;
  is_primary?: boolean;
  sort_order?: number;
  note?: string;
}

// 更新义项请求
export interface UpdateSenseRequest {
  text?: string;
  is_primary?: boolean;
  sort_order?: number;
  note?: string | null;
}


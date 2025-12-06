/**
 * 单词模块类型定义
 */

import { type Timestamp } from '@/shared/types/common.types';

// 全局词项
export interface Word {
  id: number;
  text: string;
  canonical_key: string;
  created_at: Timestamp;
}

// 个人义项
export interface UserSense {
  id: number | null;
  text: string;
  is_primary: boolean;
  sort_order: number;
  note: string | null;
  created_at: Timestamp;
}

// 个人词项
export interface UserWord {
  id: number | null;
  user_id: number;
  word_id: number;
  tags: string[];
  note: string | null;
  senses: UserSense[];
  created_at: Timestamp;
}

// 单词聚合（包含全局词项和个人词项）
export interface UserWordAggregate {
  word: Word;
  user_word: UserWord;
}

// 添加单词请求
export interface AddWordRequest {
  text: string;
  tags?: string[];
  note?: string;
  first_sense?: SenseRequest;
}

// 义项请求
export interface SenseRequest {
  text: string;
  is_primary?: boolean;
  sort_order?: number;
  note?: string;
}

// 搜索请求
export interface SearchRequest {
  q?: string;
  scope?: 'word' | 'sense' | 'both';
  limit?: number;
  offset?: number;
}

// 搜索范围
export type SearchScope = 'word' | 'sense' | 'both';

// 搜索响应
export interface SearchResponse {
  items: UserWordAggregate[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}


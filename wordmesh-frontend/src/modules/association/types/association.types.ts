/**
 * 关联网络模块类型定义
 */

import { type Timestamp } from '@/shared/types/common.types';

// 词-词关联类型（非语义关联）
export type WordLinkKind = 'similar_form' | 'root_affix';

// 义-词关联类型（语义关联）
export type SenseWordLinkKind = 'synonym' | 'antonym' | 'related';

// 词-词关联记录
export interface WordLinkRecord {
  link_id: string;
  user_id: number;
  kind: WordLinkKind;
  note: string | null;
  created_at: Timestamp;
  word_a_id: number;
  word_b_id: number;
}

// 义-词关联记录
export interface SenseWordLinkRecord {
  link_id: string;
  user_id: number;
  kind: SenseWordLinkKind;
  note: string | null;
  created_at: Timestamp;
  sense_id: number;
  source_word_id: number;
  target_word_id: number;
}

// 关联类型（联合类型）
export type AssociationRecord = WordLinkRecord | SenseWordLinkRecord;

// 创建词-词关联请求
export interface CreateWordLinkRequest {
  word_a_id: number;
  word_b_id: number;
  kind: WordLinkKind;
  note?: string;
}

// 创建义-词关联请求
export interface CreateSenseWordLinkRequest {
  sense_id: number;
  target_word_id: number;
  kind: SenseWordLinkKind;
  note?: string;
}

// 删除词-词关联请求
export interface DeleteWordLinkRequest {
  word_a_id: number;
  word_b_id: number;
  kind: WordLinkKind;
}

// 删除义-词关联请求
export interface DeleteSenseWordLinkRequest {
  sense_id: number;
  target_word_id: number;
  kind: SenseWordLinkKind;
}

// 查询关联请求
export interface ListLinksRequest {
  endpoint_type: 'word' | 'sense';
  endpoint_id: number;
  kind?: string;
  limit?: number;
  offset?: number;
}

// 关联类型显示名称映射
export const WORD_LINK_KIND_LABELS: Record<WordLinkKind, string> = {
  similar_form: '形近词',
  root_affix: '词根/词缀',
};

export const SENSE_WORD_LINK_KIND_LABELS: Record<SenseWordLinkKind, string> = {
  synonym: '同义词',
  antonym: '反义词',
  related: '相关词',
};


/**
 * 笔记 Hook
 * 提供笔记的更新功能
 * 笔记是单词或义项的一部分，通过更新单词/义项来管理笔记
 */

'use client';

import { useCallback } from 'react';
import { useSense } from '@/modules/sense';
import type { NoteNode, UpdateNoteRequest } from '../types/note.types';

export function useNote() {
  const { updateSense, isLoading, error } = useSense();

  // 更新义项笔记
  const updateSenseNote = useCallback(
    async (senseId: number, note: string | null) => {
      return updateSense(senseId, { note });
    },
    [updateSense]
  );

  // 通用更新笔记接口
  const updateNote = useCallback(
    async (node: NoteNode, request: UpdateNoteRequest) => {
      if (node.type === 'sense') {
        return updateSenseNote(node.id, request.note);
      } else {
        // 单词笔记更新需要通过更新整个单词来实现
        // 当前 API 可能不支持单独更新笔记字段
        // 返回错误提示
        return {
          success: false as const,
          error: '单词笔记更新功能需要通过更新单词接口实现',
        };
      }
    },
    [updateSenseNote]
  );

  return {
    updateNote,
    updateSenseNote,
    isLoading,
    error,
  };
}

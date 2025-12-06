/**
 * 义项列表 Hook
 * 管理单词下的所有义项
 */

'use client';

import { useState, useCallback } from 'react';
import type { UserSense } from '../types/sense.types';

export function useSenseList(initialSenses: UserSense[] = []) {
  const [senses, setSenses] = useState<UserSense[]>(initialSenses);

  // 添加义项到列表
  const addSense = useCallback((sense: UserSense) => {
    setSenses((prev) => {
      // 如果设置为主义项，取消其他义项的主义项标记
      if (sense.is_primary) {
        const updated = prev.map((s) => ({ ...s, is_primary: false }));
        return [...updated, sense].sort((a, b) => a.sort_order - b.sort_order);
      }
      return [...prev, sense].sort((a, b) => a.sort_order - b.sort_order);
    });
  }, []);

  // 更新列表中的义项
  const updateSense = useCallback((senseId: number, updates: Partial<UserSense>) => {
    setSenses((prev) => {
      const updated = prev.map((s) => {
        if (s.id === senseId) {
          const newSense = { ...s, ...updates };
          // 如果设置为主义项，取消其他义项的主义项标记
          if (updates.is_primary) {
            return newSense;
          }
          return newSense;
        }
        // 如果其他义项被设置为主义项，取消当前义项的主义项标记
        if (updates.is_primary && s.is_primary) {
          return { ...s, is_primary: false };
        }
        return s;
      });

      // 如果设置了主义项，确保只有一个主义项
      if (updates.is_primary) {
        return updated.map((s) => ({
          ...s,
          is_primary: s.id === senseId ? true : false,
        }));
      }

      return updated.sort((a, b) => a.sort_order - b.sort_order);
    });
  }, []);

  // 从列表中删除义项
  const removeSense = useCallback((senseId: number) => {
    setSenses((prev) => prev.filter((s) => s.id !== senseId));
  }, []);

  // 设置义项列表
  const setSensesList = useCallback((newSenses: UserSense[]) => {
    setSenses(newSenses.sort((a, b) => a.sort_order - b.sort_order));
  }, []);

  return {
    senses,
    addSense,
    updateSense,
    removeSense,
    setSenses: setSensesList,
  };
}


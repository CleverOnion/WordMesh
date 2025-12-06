/**
 * 单词列表组件
 */

'use client';

import { WordCard } from './WordCard';
import type { UserWordAggregate } from '../types/word.types';
import { Loader2 } from 'lucide-react';

interface WordListProps {
  words: UserWordAggregate[];
  isLoading?: boolean;
  onRemove?: (userWordId: number) => void;
  onAddSense?: (userWordId: number) => void;
  emptyMessage?: string;
}

export function WordList({
  words,
  isLoading = false,
  onRemove,
  onAddSense,
  emptyMessage = '暂无单词',
}: WordListProps) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="size-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载中...</span>
      </div>
    );
  }

  if (words.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-center">
        <p className="text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {words.map((word) => (
        <WordCard
          key={word.user_word.id || word.word.id}
          word={word}
          onRemove={onRemove}
          onAddSense={onAddSense}
        />
      ))}
    </div>
  );
}


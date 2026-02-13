/**
 * 单词选择器组件
 * 支持搜索和选择单词
 */

'use client';

import { useState, useMemo } from 'react';
import { Search, X } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import type { UserWordAggregate } from '../types/word.types';

interface WordSelectorProps {
  words: UserWordAggregate[];
  value?: number;
  onValueChange: (wordId: number) => void;
  placeholder?: string;
  excludeWordIds?: number[];
  className?: string;
  disabled?: boolean;
}

export function WordSelector({
  words,
  value,
  onValueChange,
  placeholder = '选择单词...',
  excludeWordIds = [],
  className,
  disabled = false,
}: WordSelectorProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [isOpen, setIsOpen] = useState(false);

  // 过滤单词列表：排除指定ID，并根据搜索查询过滤
  const filteredWords = useMemo(() => {
    return words.filter((word) => {
      // 排除指定的单词ID
      if (excludeWordIds.includes(word.word.id)) {
        return false;
      }

      // 搜索过滤
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        return (
          word.word.text.toLowerCase().includes(query) ||
          word.user_word.senses.some((sense) =>
            sense.text.toLowerCase().includes(query)
          )
        );
      }

      return true;
    });
  }, [words, searchQuery, excludeWordIds]);

  // 获取当前选中的单词
  const selectedWord = useMemo(() => {
    return words.find((w) => w.word.id === value);
  }, [words, value]);

  const handleSelect = (wordIdStr: string) => {
    const wordId = parseInt(wordIdStr, 10);
    if (!isNaN(wordId)) {
      onValueChange(wordId);
      setIsOpen(false);
      setSearchQuery('');
    }
  };

  return (
    <div className={className}>
      <Select
        value={value?.toString()}
        onValueChange={handleSelect}
        open={isOpen}
        onOpenChange={setIsOpen}
        disabled={disabled}
      >
        <SelectTrigger className="w-full">
          <SelectValue placeholder={placeholder}>
            {selectedWord ? (
              <span className="flex items-center gap-2">
                <span className="font-medium">{selectedWord.word.text}</span>
                <span className="text-xs text-muted-foreground">
                  (ID: {selectedWord.word.id})
                </span>
              </span>
            ) : (
              placeholder
            )}
          </SelectValue>
        </SelectTrigger>
        <SelectContent className="w-[var(--radix-select-trigger-width)]">
          {/* 搜索框 */}
          <div className="p-2 border-b">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
              <Input
                placeholder="搜索单词..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9 pr-9 h-8"
                onClick={(e) => e.stopPropagation()}
                onKeyDown={(e) => e.stopPropagation()}
              />
              {searchQuery && (
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setSearchQuery('')}
                  className="absolute right-1 top-1/2 -translate-y-1/2 h-6 w-6"
                >
                  <X className="size-3" />
                </Button>
              )}
            </div>
          </div>

          {/* 单词列表 */}
          <div className="max-h-[300px] overflow-y-auto">
            {filteredWords.length === 0 ? (
              <div className="px-2 py-6 text-center text-sm text-muted-foreground">
                {searchQuery ? '未找到匹配的单词' : '暂无单词'}
              </div>
            ) : (
              filteredWords.map((word) => (
                <SelectItem
                  key={word.word.id}
                  value={word.word.id.toString()}
                  className="cursor-pointer"
                >
                  <div className="flex items-center justify-between w-full">
                    <span className="font-medium">{word.word.text}</span>
                    <span className="text-xs text-muted-foreground ml-2">
                      ID: {word.word.id}
                    </span>
                  </div>
                </SelectItem>
              ))
            )}
          </div>
        </SelectContent>
      </Select>
    </div>
  );
}




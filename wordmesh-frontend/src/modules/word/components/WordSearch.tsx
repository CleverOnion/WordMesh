/**
 * 单词搜索组件
 */

'use client';

import { useState, useEffect } from 'react';
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
import type { SearchScope } from '../types/word.types';

interface WordSearchProps {
  query: string;
  scope: SearchScope;
  onSearch: (query: string, scope: SearchScope) => void;
  placeholder?: string;
  className?: string;
}

export function WordSearch({
  query: initialQuery,
  scope: initialScope,
  onSearch,
  placeholder = '搜索单词或义项...',
  className,
}: WordSearchProps) {
  const [query, setQuery] = useState(initialQuery);
  const [scope, setScope] = useState<SearchScope>(initialScope);

  // 防抖搜索
  useEffect(() => {
    const timer = setTimeout(() => {
      onSearch(query, scope);
    }, 300);

    return () => clearTimeout(timer);
  }, [query, scope, onSearch]);

  const handleClear = () => {
    setQuery('');
    onSearch('', scope);
  };

  return (
    <div className={`flex gap-2 ${className}`}>
      <div className="relative flex-1">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
        <Input
          placeholder={placeholder}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="pl-9 pr-9 transition-all duration-200 focus:scale-[1.01] focus:shadow-md"
        />
        {query && (
          <Button
            variant="ghost"
            size="icon"
            onClick={handleClear}
            className="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7"
          >
            <X className="size-4" />
          </Button>
        )}
      </div>
      <Select value={scope} onValueChange={(value) => setScope(value as SearchScope)}>
        <SelectTrigger className="w-[140px] transition-all duration-200 focus:scale-[1.01]">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="both">全部</SelectItem>
          <SelectItem value="word">单词</SelectItem>
          <SelectItem value="sense">义项</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}


/**
 * 单词卡片组件
 */

'use client';

import { useState } from 'react';
import { BookOpen, Tag, Trash2, Plus, ChevronDown, ChevronUp } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { UserWordAggregate } from '../types/word.types';
import { formatDateTime } from '@/shared/utils/format';

interface WordCardProps {
  word: UserWordAggregate;
  onRemove?: (userWordId: number) => void;
  onAddSense?: (userWordId: number) => void;
  className?: string;
}

export function WordCard({ word, onRemove, onAddSense, className }: WordCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const { user_word, word: globalWord } = word;

  const primarySense = user_word.senses.find((s) => s.is_primary);
  const otherSenses = user_word.senses.filter((s) => !s.is_primary);

  return (
    <Card
      className={`transition-all duration-300 hover:shadow-lg hover:border-primary/20 ${className}`}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="text-xl font-bold mb-1">{globalWord.text}</CardTitle>
            {primarySense && (
              <CardDescription className="text-base mt-1">
                {primarySense.text}
              </CardDescription>
            )}
          </div>
          <div className="flex gap-2">
            {onAddSense && (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onAddSense(user_word.id!)}
                className="h-8 w-8"
                title="添加义项"
              >
                <Plus className="size-4" />
              </Button>
            )}
            {onRemove && (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onRemove(user_word.id!)}
                className="h-8 w-8 text-destructive hover:text-destructive"
                title="移除单词"
              >
                <Trash2 className="size-4" />
              </Button>
            )}
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-3">
        {/* 标签 */}
        {user_word.tags.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {user_word.tags.map((tag, index) => (
              <Badge key={index} variant="secondary" className="text-xs">
                <Tag className="size-3 mr-1" />
                {tag}
              </Badge>
            ))}
          </div>
        )}

        {/* 笔记 */}
        {user_word.note && (
          <div className="text-sm text-muted-foreground bg-muted/50 p-2 rounded-md">
            {user_word.note}
          </div>
        )}

        {/* 义项列表 */}
        {user_word.senses.length > 0 && (
          <div className="space-y-2">
            {otherSenses.length > 0 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setIsExpanded(!isExpanded)}
                className="w-full justify-between"
              >
                <span className="text-sm">
                  {otherSenses.length} 个其他义项
                </span>
                {isExpanded ? (
                  <ChevronUp className="size-4" />
                ) : (
                  <ChevronDown className="size-4" />
                )}
              </Button>
            )}

            {isExpanded && otherSenses.length > 0 && (
              <div className="space-y-2 pl-4 border-l-2 border-muted">
                {otherSenses.map((sense) => (
                  <div key={sense.id} className="text-sm">
                    <span className="font-medium">{sense.text}</span>
                    {sense.note && (
                      <span className="text-muted-foreground ml-2">
                        - {sense.note}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* 元信息 */}
        <div className="text-xs text-muted-foreground pt-2 border-t">
          <div className="flex items-center gap-4">
            <span>创建于 {formatDateTime(user_word.created_at)}</span>
            {user_word.senses.length > 0 && (
              <span>{user_word.senses.length} 个义项</span>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}


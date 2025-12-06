/**
 * 关联列表组件
 */

'use client';

import { Trash2, Link2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Loader2 } from 'lucide-react';
import type {
  WordLinkRecord,
  SenseWordLinkRecord,
  WordLinkKind,
  SenseWordLinkKind,
} from '../types/association.types';
import {
  WORD_LINK_KIND_LABELS,
  SENSE_WORD_LINK_KIND_LABELS,
} from '../types/association.types';
import { formatDateTime } from '@/shared/utils/format';

interface AssociationListProps {
  links: (WordLinkRecord | SenseWordLinkRecord)[];
  isLoading?: boolean;
  onDelete?: (link: WordLinkRecord | SenseWordLinkRecord) => void;
  emptyMessage?: string;
  className?: string;
}

// 判断是否为词-词关联
function isWordLink(link: WordLinkRecord | SenseWordLinkRecord): link is WordLinkRecord {
  return 'word_a_id' in link;
}

export function AssociationList({
  links,
  isLoading = false,
  onDelete,
  emptyMessage = '暂无关联',
  className,
}: AssociationListProps) {
  if (isLoading) {
    return (
      <div className={`flex items-center justify-center py-12 ${className}`}>
        <Loader2 className="size-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载中...</span>
      </div>
    );
  }

  if (links.length === 0) {
    return (
      <div className={`flex flex-col items-center justify-center py-12 text-center ${className}`}>
        <p className="text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className={`space-y-3 ${className}`}>
      {links.map((link) => {
        const isWord = isWordLink(link);
        const kindLabel = isWord
          ? WORD_LINK_KIND_LABELS[link.kind as WordLinkKind]
          : SENSE_WORD_LINK_KIND_LABELS[link.kind as SenseWordLinkKind];

        return (
          <Card
            key={link.link_id}
            className="transition-all duration-300 hover:shadow-md hover:border-primary/20"
          >
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <Link2 className="size-4 text-muted-foreground" />
                    <CardTitle className="text-base">{kindLabel}</CardTitle>
                    <Badge variant="secondary" className="text-xs">
                      {isWord ? '词-词' : '义-词'}
                    </Badge>
                  </div>
                  <CardDescription className="text-sm">
                    {isWord ? (
                      <>
                        单词 A: {link.word_a_id} ↔ 单词 B: {link.word_b_id}
                      </>
                    ) : (
                      <>
                        义项 {link.sense_id} → 单词 {link.target_word_id}
                      </>
                    )}
                  </CardDescription>
                </div>
                {onDelete && (
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => onDelete(link)}
                    className="h-8 w-8 text-destructive hover:text-destructive"
                    title="删除关联"
                  >
                    <Trash2 className="size-4" />
                  </Button>
                )}
              </div>
            </CardHeader>
            {link.note && (
              <CardContent className="pt-0">
                <p className="text-sm text-muted-foreground bg-muted/50 p-2 rounded-md">
                  {link.note}
                </p>
                <p className="text-xs text-muted-foreground mt-2">
                  {formatDateTime(link.created_at)}
                </p>
              </CardContent>
            )}
            {!link.note && (
              <CardContent className="pt-0">
                <p className="text-xs text-muted-foreground">
                  {formatDateTime(link.created_at)}
                </p>
              </CardContent>
            )}
          </Card>
        );
      })}
    </div>
  );
}


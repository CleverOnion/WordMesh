/**
 * 义项详情组件
 */

'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Star } from 'lucide-react';
import type { UserSense } from '../types/sense.types';
import { formatDateTime } from '@/shared/utils/format';

interface SenseDetailProps {
  sense: UserSense;
  wordText?: string;
  className?: string;
}

export function SenseDetail({ sense, wordText, className }: SenseDetailProps) {
  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-xl">
            {wordText && <span className="text-muted-foreground">{wordText} - </span>}
            义项详情
          </CardTitle>
          {sense.is_primary && (
            <Badge variant="default">
              <Star className="size-3 mr-1" />
              主义项
            </Badge>
          )}
        </div>
        <CardDescription>排序顺序: {sense.sort_order}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <h3 className="text-sm font-medium text-muted-foreground mb-2">义项定义</h3>
          <p className="text-base">{sense.text}</p>
        </div>

        {sense.note && (
          <div>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">备注</h3>
            <p className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-md">
              {sense.note}
            </p>
          </div>
        )}

        <div className="text-xs text-muted-foreground pt-4 border-t">
          创建于 {formatDateTime(sense.created_at)}
        </div>
      </CardContent>
    </Card>
  );
}


/**
 * 义项卡片组件
 */

'use client';

import { Edit2, Trash2, Star, StarOff, Link2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { UserSense } from '../types/sense.types';
import { formatDateTime } from '@/shared/utils/format';

interface SenseCardProps {
  sense: UserSense;
  onEdit?: (sense: UserSense) => void;
  onDelete?: (senseId: number) => void;
  onTogglePrimary?: (senseId: number, isPrimary: boolean) => void;
  onAddAssociation?: () => void;
  className?: string;
}

export function SenseCard({
  sense,
  onEdit,
  onDelete,
  onTogglePrimary,
  onAddAssociation,
  className,
}: SenseCardProps) {
  return (
    <Card
      className={`transition-all duration-300 hover:shadow-md hover:border-primary/20 ${
        sense.is_primary ? 'border-primary/50 bg-primary/5' : ''
      } ${className}`}
    >
      <CardContent className="p-4 relative">
        <div className="flex items-start justify-between gap-3 relative">
          <div className="flex-1 space-y-2">
            <div className="flex items-center gap-2">
              {sense.is_primary && (
                <Badge variant="default" className="text-xs">
                  <Star className="size-3 mr-1" />
                  主义项
                </Badge>
              )}
              <span className="text-sm text-muted-foreground">
                排序: {sense.sort_order}
              </span>
            </div>
            <p className="text-base font-medium">{sense.text}</p>
            {sense.note && (
              <p className="text-sm text-muted-foreground bg-muted/50 p-2 rounded-md">
                {sense.note}
              </p>
            )}
            <p className="text-xs text-muted-foreground">
              {formatDateTime(sense.created_at)}
            </p>
          </div>
          <div className="flex flex-col gap-2 shrink-0" style={{ position: 'relative', zIndex: 100 }}>
            {onTogglePrimary && sense.id && (
              <Button
                variant="ghost"
                size="icon"
                type="button"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  console.log('Toggle primary clicked for sense:', sense.id, 'current is_primary:', sense.is_primary);
                  if (sense.id) {
                    onTogglePrimary(sense.id, !sense.is_primary);
                  }
                }}
                style={{ position: 'relative', zIndex: 1000, pointerEvents: 'auto' }}
                className="h-8 w-8"
                title={sense.is_primary ? '取消主义项' : '设为主义项'}
              >
                {sense.is_primary ? (
                  <Star className="size-4 fill-yellow-400 text-yellow-400" />
                ) : (
                  <StarOff className="size-4" />
                )}
              </Button>
            )}
            {onAddAssociation && sense.id && (
              <Button
                variant="ghost"
                size="icon"
                type="button"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  console.log('Add association clicked for sense:', sense.id);
                  onAddAssociation();
                }}
                style={{ position: 'relative', zIndex: 1000, pointerEvents: 'auto' }}
                className="h-8 w-8"
                title="添加义-词关联"
              >
                <Link2 className="size-4" />
              </Button>
            )}
            {onEdit && (
              <Button
                variant="ghost"
                size="icon"
                type="button"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  console.log('Edit clicked for sense:', sense);
                  onEdit(sense);
                }}
                style={{ position: 'relative', zIndex: 1000, pointerEvents: 'auto' }}
                className="h-8 w-8"
                title="编辑义项"
              >
                <Edit2 className="size-4" />
              </Button>
            )}
            {onDelete && sense.id && (
              <Button
                variant="ghost"
                size="icon"
                type="button"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  console.log('Delete clicked for sense:', sense.id);
                  if (sense.id) {
                    onDelete(sense.id);
                  }
                }}
                style={{ position: 'relative', zIndex: 1000, pointerEvents: 'auto' }}
                className="h-8 w-8 text-destructive hover:text-destructive"
                title="删除义项"
              >
                <Trash2 className="size-4" />
              </Button>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}


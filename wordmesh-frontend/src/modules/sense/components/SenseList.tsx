/**
 * 义项列表组件
 */

'use client';

import { SenseCard } from './SenseCard';
import type { UserSense } from '../types/sense.types';

interface SenseListProps {
  senses: UserSense[];
  onEdit?: (sense: UserSense) => void;
  onDelete?: (senseId: number) => void;
  onTogglePrimary?: (senseId: number, isPrimary: boolean) => void;
  onAddAssociation?: (senseId: number) => void;
  emptyMessage?: string;
  className?: string;
}

export function SenseList({
  senses,
  onEdit,
  onDelete,
  onTogglePrimary,
  onAddAssociation,
  emptyMessage = '暂无义项',
  className,
}: SenseListProps) {
  if (senses.length === 0) {
    return (
      <div className={`flex flex-col items-center justify-center py-8 text-center ${className}`}>
        <p className="text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  // 按排序顺序显示，主义项优先
  const sortedSenses = [...senses].sort((a, b) => {
    if (a.is_primary && !b.is_primary) return -1;
    if (!a.is_primary && b.is_primary) return 1;
    return a.sort_order - b.sort_order;
  });

  return (
    <div className={`space-y-3 ${className}`}>
      {sortedSenses.map((sense) => (
        <SenseCard
          key={sense.id}
          sense={sense}
          onEdit={onEdit}
          onDelete={onDelete}
          onTogglePrimary={onTogglePrimary}
          onAddAssociation={sense.id ? () => onAddAssociation?.(sense.id!) : undefined}
        />
      ))}
    </div>
  );
}


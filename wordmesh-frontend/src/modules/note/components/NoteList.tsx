/**
 * 笔记列表组件
 * 用于展示多个节点的笔记
 */

'use client';

import { NoteCard } from './NoteCard';
import type { NoteNode } from '../types/note.types';

interface NoteItem {
  node: NoteNode;
  note?: string | null;
  title?: string;
}

interface NoteListProps {
  notes: NoteItem[];
  onUpdate?: (node: NoteNode) => void;
  emptyMessage?: string;
  className?: string;
}

export function NoteList({
  notes,
  onUpdate,
  emptyMessage = '暂无笔记',
  className,
}: NoteListProps) {
  if (notes.length === 0) {
    return (
      <div className={`flex flex-col items-center justify-center py-12 text-center ${className}`}>
        <p className="text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {notes.map((item, index) => (
        <NoteCard
          key={`${item.node.type}-${item.node.id}-${index}`}
          node={item.node}
          note={item.note}
          title={item.title}
          onUpdate={() => onUpdate?.(item.node)}
        />
      ))}
    </div>
  );
}


/**
 * 笔记卡片组件
 */

'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Edit2, FileText } from 'lucide-react';
import { NoteEditor } from './NoteEditor';
import type { NoteNode } from '../types/note.types';

interface NoteCardProps {
  node: NoteNode;
  note?: string | null;
  title?: string;
  onUpdate?: () => void;
  className?: string;
}

export function NoteCard({
  node,
  note,
  title,
  onUpdate,
  className,
}: NoteCardProps) {
  const [isEditing, setIsEditing] = useState(false);

  if (isEditing) {
    return (
      <Card className={className}>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <FileText className="size-5" />
            {title || '笔记'}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <NoteEditor
            node={node}
            initialNote={note}
            onSave={() => {
              setIsEditing(false);
              onUpdate?.();
            }}
            onCancel={() => setIsEditing(false)}
          />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={`transition-all duration-300 hover:shadow-md ${className}`}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            <FileText className="size-5" />
            {title || '笔记'}
          </CardTitle>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsEditing(true)}
            className="h-8 w-8"
            title="编辑笔记"
          >
            <Edit2 className="size-4" />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {note ? (
          <div className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-md whitespace-pre-wrap">
            {note}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground text-center py-4">
            暂无笔记
            <Button
              variant="link"
              size="sm"
              onClick={() => setIsEditing(true)}
              className="ml-2 h-auto p-0"
            >
              添加笔记
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}


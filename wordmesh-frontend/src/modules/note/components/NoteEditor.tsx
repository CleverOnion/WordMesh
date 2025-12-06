/**
 * 笔记编辑器组件
 */

'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Loader2, Save, X } from 'lucide-react';
import { useNote } from '../hooks/useNote';
import type { NoteNode } from '../types/note.types';

// 表单验证 schema
const noteFormSchema = z.object({
  note: z.string().max(512, '笔记长度不能超过 512 个字符').optional(),
});

type NoteFormValues = z.infer<typeof noteFormSchema>;

interface NoteEditorProps {
  node: NoteNode;
  initialNote?: string | null;
  onSave?: () => void;
  onCancel?: () => void;
  className?: string;
}

export function NoteEditor({
  node,
  initialNote,
  onSave,
  onCancel,
  className,
}: NoteEditorProps) {
  const { updateNote, isLoading } = useNote();
  const [isEditing, setIsEditing] = useState(!initialNote);

  const form = useForm<NoteFormValues>({
    resolver: zodResolver(noteFormSchema),
    defaultValues: {
      note: initialNote || '',
    },
  });

  const handleSubmit = async (values: NoteFormValues) => {
    const result = await updateNote(node, {
      note: values.note || null,
    });

    if (result.success) {
      setIsEditing(false);
      onSave?.();
    }
  };

  if (!isEditing && initialNote) {
    return (
      <div className={`space-y-2 ${className}`}>
        <div className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-md whitespace-pre-wrap">
          {initialNote}
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setIsEditing(true)}
            className="transition-all duration-200 hover:scale-[1.02]"
          >
            编辑笔记
          </Button>
        </div>
      </div>
    );
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className={`space-y-4 ${className}`}>
        <FormField
          control={form.control}
          name="note"
          render={({ field }) => (
            <FormItem>
              <FormLabel>
                {node.type === 'word' ? '单词笔记' : '义项笔记'}
              </FormLabel>
              <FormControl>
                <Textarea
                  placeholder="输入笔记内容..."
                  className="transition-all duration-200 focus:scale-[1.01] focus:shadow-md resize-none min-h-[100px]"
                  rows={5}
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <div className="flex gap-2">
          <Button
            type="submit"
            size="sm"
            disabled={isLoading}
            className="transition-all duration-300 hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
          >
            {isLoading ? (
              <span className="flex items-center gap-2">
                <Loader2 className="size-4 animate-spin" />
                保存中...
              </span>
            ) : (
              <span className="flex items-center gap-2">
                <Save className="size-4" />
                保存
              </span>
            )}
          </Button>
          {onCancel && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => {
                form.reset();
                setIsEditing(false);
                onCancel();
              }}
              disabled={isLoading}
              className="transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
            >
              <X className="size-4 mr-1" />
              取消
            </Button>
          )}
        </div>
      </form>
    </Form>
  );
}


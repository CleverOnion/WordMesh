/**
 * 关联表单组件
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
  FormDescription,
} from '@/components/ui/form';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Loader2 } from 'lucide-react';
import { AssociationTypeSelector } from './AssociationTypeSelector';
import { WordSelector } from '@/modules/word';
import type {
  CreateWordLinkRequest,
  CreateSenseWordLinkRequest,
  WordLinkKind,
  SenseWordLinkKind,
} from '../types/association.types';
import type { UserWordAggregate } from '@/modules/word';
import type { UserSense } from '@/modules/sense';

// 词-词关联表单 schema
const wordLinkFormSchema = z.object({
  word_a_id: z.number().int().positive('请选择第一个单词'),
  word_b_id: z.number().int().positive('请选择第二个单词'),
  kind: z.enum(['similar_form', 'root_affix']),
  note: z.string().max(512, '备注长度不能超过 512 个字符').optional(),
});

// 义-词关联表单 schema
const senseWordLinkFormSchema = z.object({
  sense_id: z.number().int().positive('请选择义项'),
  target_word_id: z.number().int().positive('请选择目标单词'),
  kind: z.enum(['synonym', 'antonym', 'related']),
  note: z.string().max(512, '备注长度不能超过 512 个字符').optional(),
});

type WordLinkFormValues = z.infer<typeof wordLinkFormSchema>;
type SenseWordLinkFormValues = z.infer<typeof senseWordLinkFormSchema>;

interface AssociationFormProps {
  type: 'word' | 'sense';
  onSubmit: (
    data: CreateWordLinkRequest | CreateSenseWordLinkRequest
  ) => Promise<{ success: boolean }>;
  onCancel?: () => void;
  isLoading?: boolean;
  // 词-词关联的预设值
  wordAId?: number;
  wordBId?: number;
  // 义-词关联的预设值
  senseId?: number;
  targetWordId?: number;
  // 单词列表（用于 WordSelector）
  words?: UserWordAggregate[];
  // 当前选中的单词（用于义项选择器）
  selectedWord?: UserWordAggregate | null;
}

export function AssociationForm({
  type,
  onSubmit,
  onCancel,
  isLoading = false,
  wordAId,
  wordBId,
  senseId,
  targetWordId,
  words = [],
  selectedWord,
}: AssociationFormProps) {
  const wordForm = useForm<WordLinkFormValues>({
    resolver: zodResolver(wordLinkFormSchema),
    defaultValues: {
      word_a_id: wordAId || 0,
      word_b_id: wordBId || 0,
      kind: 'similar_form',
      note: '',
    },
  });

  const senseForm = useForm<SenseWordLinkFormValues>({
    resolver: zodResolver(senseWordLinkFormSchema),
    defaultValues: {
      sense_id: senseId || 0,
      target_word_id: targetWordId || 0,
      kind: 'synonym',
      note: '',
    },
  });

  const handleWordSubmit = async (values: WordLinkFormValues) => {
    const request: CreateWordLinkRequest = {
      word_a_id: values.word_a_id,
      word_b_id: values.word_b_id,
      kind: values.kind as WordLinkKind,
      note: values.note || undefined,
    };
    const result = await onSubmit(request);
    if (result.success) {
      wordForm.reset();
    }
  };

  const handleSenseSubmit = async (values: SenseWordLinkFormValues) => {
    const request: CreateSenseWordLinkRequest = {
      sense_id: values.sense_id,
      target_word_id: values.target_word_id,
      kind: values.kind as SenseWordLinkKind,
      note: values.note || undefined,
    };
    const result = await onSubmit(request);
    if (result.success) {
      senseForm.reset();
    }
  };

  if (type === 'word') {
    return (
      <Form {...wordForm}>
        <form onSubmit={wordForm.handleSubmit(handleWordSubmit)} className="space-y-5">
          <FormField
            control={wordForm.control}
            name="word_a_id"
            render={({ field }) => (
              <FormItem>
                <FormLabel>第一个单词</FormLabel>
                <FormControl>
                  <WordSelector
                    words={words}
                    value={field.value || undefined}
                    onValueChange={field.onChange}
                    placeholder="选择第一个单词"
                    excludeWordIds={wordBId ? [wordBId] : []}
                    disabled={isLoading}
                  />
                </FormControl>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />

          <FormField
            control={wordForm.control}
            name="word_b_id"
            render={({ field }) => (
              <FormItem>
                <FormLabel>第二个单词</FormLabel>
                <FormControl>
                  <WordSelector
                    words={words}
                    value={field.value || undefined}
                    onValueChange={field.onChange}
                    placeholder="选择第二个单词"
                    excludeWordIds={wordAId ? [wordAId] : []}
                    disabled={isLoading}
                  />
                </FormControl>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />

          <FormField
            control={wordForm.control}
            name="kind"
            render={({ field }) => (
              <FormItem>
                <FormLabel>关联类型</FormLabel>
                <FormControl>
                  <AssociationTypeSelector
                    type="word"
                    value={field.value}
                    onValueChange={field.onChange}
                  />
                </FormControl>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />

          <FormField
            control={wordForm.control}
            name="note"
            render={({ field }) => (
              <FormItem>
                <FormLabel>备注（可选）</FormLabel>
                <FormControl>
                  <Textarea
                    placeholder="添加备注..."
                    className="transition-all duration-200 focus:scale-[1.01] focus:shadow-md resize-none"
                    rows={3}
                    {...field}
                  />
                </FormControl>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />

          <div className="flex gap-3">
            <Button
              type="submit"
              className="flex-1 transition-all duration-300 hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
              disabled={isLoading}
            >
              {isLoading ? (
                <span className="flex items-center gap-2">
                  <Loader2 className="size-4 animate-spin" />
                  创建中...
                </span>
              ) : (
                '创建关联'
              )}
            </Button>
            {onCancel && (
              <Button
                type="button"
                variant="outline"
                onClick={onCancel}
                disabled={isLoading}
                className="transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
              >
                取消
              </Button>
            )}
          </div>
        </form>
      </Form>
    );
  }

  return (
    <Form {...senseForm}>
      <form onSubmit={senseForm.handleSubmit(handleSenseSubmit)} className="space-y-5">
        <FormField
          control={senseForm.control}
          name="sense_id"
          render={({ field }) => (
            <FormItem>
              <FormLabel>义项</FormLabel>
              <FormControl>
                {selectedWord && selectedWord.user_word.senses.length > 0 ? (
                  <Select
                    value={field.value?.toString()}
                    onValueChange={(value) => field.onChange(parseInt(value, 10))}
                    disabled={isLoading}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择义项" />
                    </SelectTrigger>
                    <SelectContent>
                      {selectedWord.user_word.senses.map((sense) => (
                        <SelectItem
                          key={sense.id || `sense-${sense.text}`}
                          value={sense.id?.toString() || ''}
                        >
                          <div className="flex flex-col">
                            <span className="font-medium">{sense.text}</span>
                            {sense.is_primary && (
                              <span className="text-xs text-muted-foreground">主义项</span>
                            )}
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                ) : (
                  <Select
                    value={field.value?.toString()}
                    onValueChange={(value) => field.onChange(parseInt(value, 10))}
                    disabled={isLoading}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="请输入义项 ID" />
                    </SelectTrigger>
                    <SelectContent>
                      <div className="px-2 py-6 text-center text-sm text-muted-foreground">
                        请先选择单词以查看义项列表
                      </div>
                    </SelectContent>
                  </Select>
                )}
              </FormControl>
              <FormDescription>
                {selectedWord
                  ? `从 "${selectedWord.word.text}" 的义项中选择`
                  : '请先选择单词以查看义项列表'}
              </FormDescription>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <FormField
          control={senseForm.control}
          name="target_word_id"
          render={({ field }) => (
            <FormItem>
              <FormLabel>目标单词</FormLabel>
              <FormControl>
                <WordSelector
                  words={words}
                  value={field.value || undefined}
                  onValueChange={field.onChange}
                  placeholder="选择目标单词"
                  excludeWordIds={selectedWord ? [selectedWord.word.id] : []}
                  disabled={isLoading}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <FormField
          control={senseForm.control}
          name="kind"
          render={({ field }) => (
            <FormItem>
              <FormLabel>关联类型</FormLabel>
              <FormControl>
                <AssociationTypeSelector
                  type="sense"
                  value={field.value}
                  onValueChange={field.onChange}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <FormField
          control={senseForm.control}
          name="note"
          render={({ field }) => (
            <FormItem>
              <FormLabel>备注（可选）</FormLabel>
              <FormControl>
                <Textarea
                  placeholder="添加备注..."
                  className="transition-all duration-200 focus:scale-[1.01] focus:shadow-md resize-none"
                  rows={3}
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <div className="flex gap-3">
          <Button
            type="submit"
            className="flex-1 transition-all duration-300 hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
            disabled={isLoading}
          >
            {isLoading ? (
              <span className="flex items-center gap-2">
                <Loader2 className="size-4 animate-spin" />
                创建中...
              </span>
            ) : (
              '创建关联'
            )}
          </Button>
          {onCancel && (
            <Button
              type="button"
              variant="outline"
              onClick={onCancel}
              disabled={isLoading}
              className="transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
            >
              取消
            </Button>
          )}
        </div>
      </form>
    </Form>
  );
}


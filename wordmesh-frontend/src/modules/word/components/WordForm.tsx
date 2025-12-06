/**
 * 添加单词表单组件
 */

'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  FormDescription,
} from '@/components/ui/form';
import { Loader2, Plus, X } from 'lucide-react';
import type { AddWordRequest } from '../types/word.types';
import { validateWordText } from '@/shared/utils/validation';

// 表单验证 schema
const wordFormSchema = z.object({
  text: z
    .string()
    .min(1, '单词不能为空')
    .max(128, '单词长度不能超过 128 个字符')
    .refine(validateWordText, '单词格式不正确'),
  tags: z.array(z.string()).optional(),
  note: z.string().max(512, '备注长度不能超过 512 个字符').optional(),
  firstSense: z
    .object({
      text: z.string().min(1, '义项不能为空').max(512, '义项长度不能超过 512 个字符'),
      is_primary: z.boolean().optional(),
      note: z.string().max(512, '备注长度不能超过 512 个字符').optional(),
    })
    .optional(),
});

type WordFormValues = z.infer<typeof wordFormSchema>;

interface WordFormProps {
  onSubmit: (data: AddWordRequest) => Promise<{ success: boolean }>;
  onCancel?: () => void;
  isLoading?: boolean;
}

export function WordForm({ onSubmit, onCancel, isLoading = false }: WordFormProps) {
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');

  const form = useForm<WordFormValues>({
    resolver: zodResolver(wordFormSchema),
    defaultValues: {
      text: '',
      tags: [],
      note: '',
      firstSense: undefined,
    },
  });

  const handleSubmit = async (values: WordFormValues) => {
    const request: AddWordRequest = {
      text: values.text,
      tags: tags.length > 0 ? tags : undefined,
      note: values.note || undefined,
      firstSense: values.firstSense
        ? {
            text: values.firstSense.text,
            is_primary: values.firstSense.is_primary || false,
            note: values.firstSense.note || undefined,
          }
        : undefined,
    };

    const result = await onSubmit(request);
    if (result.success) {
      form.reset();
      setTags([]);
      setTagInput('');
    }
  };

  const addTag = () => {
    const trimmed = tagInput.trim();
    if (trimmed && !tags.includes(trimmed) && tags.length < 20) {
      setTags([...tags, trimmed]);
      setTagInput('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setTags(tags.filter((tag) => tag !== tagToRemove));
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-5">
        <FormField
          control={form.control}
          name="text"
          render={({ field }) => (
            <FormItem>
              <FormLabel>单词</FormLabel>
              <FormControl>
                <Input
                  placeholder="请输入单词"
                  className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        {/* 标签输入 */}
        <div className="space-y-2">
          <FormLabel>标签（可选）</FormLabel>
          <div className="flex gap-2">
            <Input
              placeholder="输入标签后按回车"
              value={tagInput}
              onChange={(e) => setTagInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  addTag();
                }
              }}
              className="flex-1"
            />
            <Button type="button" variant="outline" onClick={addTag} disabled={tags.length >= 20}>
              <Plus className="size-4" />
            </Button>
          </div>
          {tags.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {tags.map((tag) => (
                <div
                  key={tag}
                  className="flex items-center gap-1 bg-secondary text-secondary-foreground px-2 py-1 rounded-md text-sm"
                >
                  <span>{tag}</span>
                  <button
                    type="button"
                    onClick={() => removeTag(tag)}
                    className="hover:text-destructive transition-colors"
                  >
                    <X className="size-3" />
                  </button>
                </div>
              ))}
            </div>
          )}
          <FormDescription>最多添加 20 个标签</FormDescription>
        </div>

        <FormField
          control={form.control}
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
                  value={field.value || ''}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        {/* 首条义项（可选） */}
        <div className="space-y-4 p-4 border rounded-md">
          <FormLabel>首条义项（可选）</FormLabel>
          <FormField
            control={form.control}
            name="firstSense.text"
            render={({ field }) => (
              <FormItem>
                <FormLabel>义项定义</FormLabel>
                <FormControl>
                  <Input
                    placeholder="请输入义项定义"
                    className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                    {...field}
                    value={field.value || ''}
                  />
                </FormControl>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="firstSense.is_primary"
            render={({ field }) => (
              <FormItem className="flex items-center space-x-2 space-y-0 rounded-md border p-4">
                <FormControl>
                  <Checkbox
                    checked={field.value || false}
                    onCheckedChange={field.onChange}
                  />
                </FormControl>
                <div className="space-y-1 leading-none">
                  <FormLabel className="!mt-0 cursor-pointer">
                    设为主义项
                  </FormLabel>
                </div>
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="firstSense.note"
            render={({ field }) => (
              <FormItem>
                <FormLabel>义项备注（可选）</FormLabel>
                <FormControl>
                  <Textarea
                    placeholder="添加义项备注..."
                    className="transition-all duration-200 focus:scale-[1.01] focus:shadow-md resize-none"
                    rows={2}
                    {...field}
                    value={field.value || ''}
                  />
                </FormControl>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />
        </div>

        <div className="flex gap-3">
          <Button
            type="submit"
            className="flex-1 transition-all duration-300 hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
            disabled={isLoading}
          >
            {isLoading ? (
              <span className="flex items-center gap-2">
                <Loader2 className="size-4 animate-spin" />
                添加中...
              </span>
            ) : (
              '添加单词'
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


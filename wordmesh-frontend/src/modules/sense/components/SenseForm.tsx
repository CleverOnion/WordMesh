/**
 * 义项表单组件（用于添加和编辑）
 */

'use client';

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
import { Loader2 } from 'lucide-react';
import type { AddSenseRequest, UpdateSenseRequest, UserSense } from '../types/sense.types';
import { validateSenseDefinition } from '@/shared/utils/validation';

// 表单验证 schema
const senseFormSchema = z.object({
  text: z
    .string()
    .min(1, '义项定义不能为空')
    .max(512, '义项定义长度不能超过 512 个字符')
    .refine(validateSenseDefinition, '义项定义格式不正确'),
  is_primary: z.boolean().optional(),
  sort_order: z.number().int().min(0).optional(),
  note: z.string().max(512, '备注长度不能超过 512 个字符').optional(),
});

type SenseFormValues = z.infer<typeof senseFormSchema>;

interface SenseFormProps {
  sense?: UserSense;
  onSubmit: (data: AddSenseRequest | UpdateSenseRequest) => Promise<{ success: boolean }>;
  onCancel?: () => void;
  isLoading?: boolean;
}

export function SenseForm({ sense, onSubmit, onCancel, isLoading = false }: SenseFormProps) {
  const isEditing = !!sense;

  const form = useForm<SenseFormValues>({
    resolver: zodResolver(senseFormSchema),
    defaultValues: {
      text: sense?.text || '',
      is_primary: sense?.is_primary || false,
      sort_order: sense?.sort_order ?? 0,
      note: sense?.note || '',
    },
  });

  const handleSubmit = async (values: SenseFormValues) => {
    const request: AddSenseRequest | UpdateSenseRequest = {
      text: values.text,
      is_primary: values.is_primary,
      sort_order: values.sort_order,
      note: values.note || undefined,
    };

    const result = await onSubmit(request);
    if (result.success && !isEditing) {
      form.reset();
    }
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-5">
        <FormField
          control={form.control}
          name="text"
          render={({ field }) => (
            <FormItem>
              <FormLabel>义项定义</FormLabel>
              <FormControl>
                <Input
                  placeholder="请输入义项定义"
                  className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <div className="grid grid-cols-2 gap-4">
          <FormField
            control={form.control}
            name="sort_order"
            render={({ field }) => (
              <FormItem>
                <FormLabel>排序顺序</FormLabel>
                <FormControl>
                  <Input
                    type="number"
                    min={0}
                    placeholder="0"
                    className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                    {...field}
                    onChange={(e) => field.onChange(parseInt(e.target.value) || 0)}
                    value={field.value ?? 0}
                  />
                </FormControl>
                <FormDescription>数字越小越靠前</FormDescription>
                <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="is_primary"
            render={({ field }) => (
              <FormItem className="flex items-center space-x-2 space-y-0 rounded-md border p-4">
                <FormControl>
                  <Checkbox
                    checked={field.value || false}
                    onCheckedChange={field.onChange}
                  />
                </FormControl>
                <div className="space-y-1 leading-none">
                  <FormLabel className="!mt-0 cursor-pointer">设为主义项</FormLabel>
                  <FormDescription>每个单词只能有一个主义项</FormDescription>
                </div>
              </FormItem>
            )}
          />
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
                {isEditing ? '更新中...' : '添加中...'}
              </span>
            ) : (
              isEditing ? '更新义项' : '添加义项'
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


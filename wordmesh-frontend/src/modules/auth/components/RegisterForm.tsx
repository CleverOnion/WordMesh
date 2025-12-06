/**
 * 注册表单组件
 */

'use client';

import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { useAuth } from '../hooks/useAuth';
import { validateUsername, validatePassword } from '@/shared/utils/validation';
import { Loader2 } from 'lucide-react';

// 表单验证 schema
const registerSchema = z
  .object({
    username: z
      .string()
      .min(3, '用户名长度必须在 3 到 32 之间')
      .max(32, '用户名长度必须在 3 到 32 之间')
      .refine(validateUsername, '用户名格式不正确'),
    password: z
      .string()
      .min(8, '密码长度至少 8 位')
      .refine(validatePassword, '密码格式不正确'),
    confirmPassword: z.string().min(8, '请确认密码'),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: '两次输入的密码不一致',
    path: ['confirmPassword'],
  });

type RegisterFormValues = z.infer<typeof registerSchema>;

export function RegisterForm() {
  const router = useRouter();
  const { register: registerUser, isLoading } = useAuth();

  const form = useForm<RegisterFormValues>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      username: '',
      password: '',
      confirmPassword: '',
    },
  });

  const onSubmit = async (values: RegisterFormValues) => {
    const result = await registerUser({
      username: values.username,
      password: values.password,
    });

    if (result.success) {
      router.push('/network');
    } else {
      // 设置字段级错误（仅用于表单验证错误）
      if (result.fieldErrors) {
        result.fieldErrors.forEach((error) => {
          form.setError(error.field as keyof RegisterFormValues, {
            message: error.message,
          });
        });
      }
      // API 错误已通过 toast 显示，不需要在表单中显示
    }
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-5">
        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem className="space-y-2">
              <FormLabel className="transition-colors duration-200">
                用户名
              </FormLabel>
              <FormControl>
                <Input
                  placeholder="请输入用户名（3-32个字符）"
                  autoComplete="username"
                  className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem className="space-y-2">
              <FormLabel className="transition-colors duration-200">
                密码
              </FormLabel>
              <FormControl>
                <Input
                  type="password"
                  placeholder="请输入密码（至少8位）"
                  autoComplete="new-password"
                  className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="confirmPassword"
          render={({ field }) => (
            <FormItem className="space-y-2">
              <FormLabel className="transition-colors duration-200">
                确认密码
              </FormLabel>
              <FormControl>
                <Input
                  type="password"
                  placeholder="请再次输入密码"
                  autoComplete="new-password"
                  className="transition-all duration-200 focus:scale-[1.02] focus:shadow-md"
                  {...field}
                />
              </FormControl>
              <FormMessage className="animate-in slide-in-from-top-1 duration-200" />
            </FormItem>
          )}
        />

        <Button
          type="submit"
          className="w-full transition-all duration-300 hover:scale-[1.02] hover:shadow-lg active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={isLoading}
        >
          {isLoading ? (
            <span className="flex items-center gap-2">
              <Loader2 className="size-4 animate-spin" />
              注册中...
            </span>
          ) : (
            '注册'
          )}
        </Button>
      </form>
    </Form>
  );
}

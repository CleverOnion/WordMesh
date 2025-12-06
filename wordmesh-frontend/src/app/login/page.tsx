/**
 * 登录页面
 */

'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { BookOpen, Infinity, Link2, GraduationCap } from 'lucide-react';
import { LoginForm } from '@/modules/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export default function LoginPage() {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-zinc-50 via-white to-zinc-100 dark:from-black dark:via-zinc-950 dark:to-zinc-900 p-4">
      <div className="w-full max-w-6xl grid lg:grid-cols-2 gap-8 items-center">
        {/* 左侧：品牌和介绍 */}
        <div
          className={`hidden lg:flex flex-col items-center justify-center space-y-6 p-8 transition-all duration-1000 ${
            mounted
              ? 'opacity-100 translate-x-0'
              : 'opacity-0 -translate-x-8'
          }`}
        >
          <div
            className={`flex items-center gap-3 mb-8 transition-all duration-700 delay-100 ${
              mounted ? 'opacity-100 scale-100' : 'opacity-0 scale-95'
            }`}
          >
            <div className="bg-primary text-primary-foreground flex size-12 items-center justify-center rounded-xl shadow-lg hover:scale-110 transition-transform duration-300 hover:shadow-xl">
              <BookOpen className="size-6" />
            </div>
            <h1 className="text-4xl font-bold bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent">
              WordMesh
            </h1>
          </div>
          <div
            className={`space-y-4 text-center max-w-md transition-all duration-700 delay-200 ${
              mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
            }`}
          >
            <h2 className="text-3xl font-semibold">
              构建您的单词知识网络
            </h2>
            <p className="text-lg text-muted-foreground leading-relaxed">
              通过创建个性化的单词关联网络，实现高效、深刻的单词记忆。
              让每个单词都成为您知识宇宙中的一颗星。
            </p>
          </div>
          <div
            className={`mt-8 grid grid-cols-3 gap-4 text-center transition-all duration-700 delay-300 ${
              mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
            }`}
          >
            <div className="space-y-2 group">
              <div className="flex justify-center">
                <div className="bg-primary/10 p-2 rounded-lg group-hover:bg-primary/20 group-hover:scale-110 transition-all duration-300">
                  <Infinity className="size-5 text-primary" />
                </div>
              </div>
              <div className="text-sm text-muted-foreground group-hover:text-foreground transition-colors duration-300">
                无限扩展
              </div>
            </div>
            <div className="space-y-2 group">
              <div className="flex justify-center">
                <div className="bg-primary/10 p-2 rounded-lg group-hover:bg-primary/20 group-hover:scale-110 transition-all duration-300">
                  <Link2 className="size-5 text-primary" />
                </div>
              </div>
              <div className="text-sm text-muted-foreground group-hover:text-foreground transition-colors duration-300">
                智能关联
              </div>
            </div>
            <div className="space-y-2 group">
              <div className="flex justify-center">
                <div className="bg-primary/10 p-2 rounded-lg group-hover:bg-primary/20 group-hover:scale-110 transition-all duration-300">
                  <GraduationCap className="size-5 text-primary" />
                </div>
              </div>
              <div className="text-sm text-muted-foreground group-hover:text-foreground transition-colors duration-300">
                个性化学习
              </div>
            </div>
          </div>
        </div>

        {/* 右侧：登录表单 */}
        <div className="w-full flex items-center justify-center">
          <Card
            className={`w-full max-w-md shadow-xl border-2 transition-all duration-700 delay-200 ${
              mounted
                ? 'opacity-100 translate-y-0 scale-100'
                : 'opacity-0 translate-y-8 scale-95'
            } hover:shadow-2xl hover:border-primary/20`}
          >
            <CardHeader className="space-y-1 text-center">
              <div
                className={`flex justify-center mb-4 transition-all duration-500 delay-300 ${
                  mounted
                    ? 'opacity-100 scale-100 rotate-0'
                    : 'opacity-0 scale-0 rotate-180'
                }`}
              >
                <div className="bg-primary/10 p-3 rounded-full hover:bg-primary/20 hover:scale-110 transition-all duration-300">
                  <BookOpen className="size-6 text-primary" />
                </div>
              </div>
              <CardTitle
                className={`text-2xl font-bold transition-all duration-500 delay-400 ${
                  mounted
                    ? 'opacity-100 translate-y-0'
                    : 'opacity-0 translate-y-2'
                }`}
              >
                欢迎回来
              </CardTitle>
              <CardDescription
                className={`transition-all duration-500 delay-500 ${
                  mounted
                    ? 'opacity-100 translate-y-0'
                    : 'opacity-0 translate-y-2'
                }`}
              >
                登录您的账户，继续构建单词网络
              </CardDescription>
            </CardHeader>
            <CardContent
              className={`transition-all duration-500 delay-600 ${
                mounted ? 'opacity-100' : 'opacity-0'
              }`}
            >
              <LoginForm />
              <div className="mt-6 text-center text-sm">
                <span className="text-muted-foreground">还没有账号？</span>{' '}
                <Link
                  href="/register"
                  className="font-medium text-primary hover:underline transition-all duration-200 hover:text-primary/80 relative group"
                >
                  立即注册
                  <span className="absolute bottom-0 left-0 w-0 h-0.5 bg-primary group-hover:w-full transition-all duration-300" />
                </Link>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

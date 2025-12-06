/**
 * 注册页面
 */

'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { BookOpen, Sparkles } from 'lucide-react';
import { RegisterForm } from '@/modules/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export default function RegisterPage() {
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
            <div className="bg-primary text-primary-foreground flex size-12 items-center justify-center rounded-xl shadow-lg hover:scale-110 transition-transform duration-300 hover:shadow-xl relative">
              <BookOpen className="size-6" />
              <Sparkles className="size-3 absolute -top-1 -right-1 text-yellow-400 animate-pulse" />
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
              开始您的学习之旅
            </h2>
            <p className="text-lg text-muted-foreground leading-relaxed">
              加入 WordMesh，从零开始构建您的单词知识网络。
              每个单词都是您知识宇宙中的一颗星，让它们相互连接，形成属于您的学习星系。
            </p>
          </div>
          <div
            className={`mt-8 space-y-3 text-left max-w-sm transition-all duration-700 delay-300 ${
              mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
            }`}
          >
            {[
              {
                title: '个性化学习',
                desc: '根据您的理解创建单词和义项',
              },
              {
                title: '智能关联',
                desc: '建立同义、反义、形近等多种关联',
              },
              {
                title: '可视化探索',
                desc: '以图形化方式浏览您的知识网络',
              },
            ].map((item, index) => (
              <div
                key={index}
                className="flex items-start gap-3 group hover:translate-x-1 transition-all duration-300"
              >
                <div className="mt-1 size-2 rounded-full bg-primary group-hover:scale-150 transition-transform duration-300" />
                <div>
                  <div className="font-medium group-hover:text-primary transition-colors duration-300">
                    {item.title}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {item.desc}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* 右侧：注册表单 */}
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
                  <Sparkles className="size-6 text-primary" />
                </div>
              </div>
              <CardTitle
                className={`text-2xl font-bold transition-all duration-500 delay-400 ${
                  mounted
                    ? 'opacity-100 translate-y-0'
                    : 'opacity-0 translate-y-2'
                }`}
              >
                创建账户
              </CardTitle>
              <CardDescription
                className={`transition-all duration-500 delay-500 ${
                  mounted
                    ? 'opacity-100 translate-y-0'
                    : 'opacity-0 translate-y-2'
                }`}
              >
                注册您的账户，开始构建单词网络
              </CardDescription>
            </CardHeader>
            <CardContent
              className={`transition-all duration-500 delay-600 ${
                mounted ? 'opacity-100' : 'opacity-0'
              }`}
            >
              <RegisterForm />
              <div className="mt-6 text-center text-sm">
                <span className="text-muted-foreground">已有账号？</span>{' '}
                <Link
                  href="/login"
                  className="font-medium text-primary hover:underline transition-all duration-200 hover:text-primary/80 relative group"
                >
                  立即登录
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

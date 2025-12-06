/**
 * 仪表板页面
 */

'use client';

import { ProfileCard } from '@/modules/auth';
import { WordList, useWordList, useWord } from '@/modules/word';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { BookOpen, Network, Search, Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import Link from 'next/link';
import { useState } from 'react';
import { WordForm } from '@/modules/word';

export default function DashboardPage() {
  const { words = [], isLoading, refresh } = useWordList();
  const { addWord, isLoading: isAddingWord } = useWord();
  const [showAddWord, setShowAddWord] = useState(false);

  return (
    <div className="space-y-6">
      {/* 欢迎区域 */}
      <div>
        <h1 className="text-3xl font-bold">欢迎使用 WordMesh</h1>
        <p className="text-muted-foreground mt-2">
          开始构建您的单词知识网络
        </p>
      </div>

      {/* 快速操作卡片 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card className="transition-all duration-300 hover:shadow-md">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">我的词网</CardTitle>
            <BookOpen className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{words.length}</div>
            <p className="text-xs text-muted-foreground">已添加的单词</p>
          </CardContent>
        </Card>

        <Card className="transition-all duration-300 hover:shadow-md">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">网络视图</CardTitle>
            <Network className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <Link href="/network">
              <Button variant="link" className="p-0 h-auto">
                查看网络图
              </Button>
            </Link>
          </CardContent>
        </Card>

        <Card className="transition-all duration-300 hover:shadow-md">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">搜索</CardTitle>
            <Search className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <Link href="/search">
              <Button variant="link" className="p-0 h-auto">
                搜索单词
              </Button>
            </Link>
          </CardContent>
        </Card>

        <Card className="transition-all duration-300 hover:shadow-md">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">添加单词</CardTitle>
            <Plus className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <Button
              variant="link"
              className="p-0 h-auto"
              onClick={() => setShowAddWord(true)}
            >
              快速添加
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* 用户信息 */}
      <div className="grid gap-6 md:grid-cols-2">
        <ProfileCard />
      </div>

      {/* 最近添加的单词 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>最近添加的单词</CardTitle>
              <CardDescription>您最近添加到词网的单词</CardDescription>
            </div>
            <Link href="/words">
              <Button variant="outline" size="sm">
                查看全部
              </Button>
            </Link>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-muted-foreground text-center py-8">加载中...</p>
          ) : words.length > 0 ? (
            <WordList words={words.slice(0, 5)} />
          ) : (
            <div className="text-center py-8">
              <p className="text-muted-foreground mb-4">还没有添加任何单词</p>
              <Button onClick={() => setShowAddWord(true)}>
                <Plus className="size-4 mr-2" />
                添加第一个单词
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* 添加单词表单（模态） */}
      {showAddWord && (
        <Card>
          <CardHeader>
            <CardTitle>添加新单词</CardTitle>
          </CardHeader>
          <CardContent>
            <WordForm
              onSubmit={async (data) => {
                const result = await addWord(data);
                if (result.success) {
                  await refresh();
                  setShowAddWord(false);
                }
                return result;
              }}
              onCancel={() => setShowAddWord(false)}
              isLoading={isAddingWord}
            />
          </CardContent>
        </Card>
      )}
    </div>
  );
}


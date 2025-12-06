/**
 * 搜索页面
 */

'use client';

import { WordList, WordSearch, useWordList } from '@/modules/word';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Search } from 'lucide-react';

export default function SearchPage() {
  const { words = [], isLoading, search } = useWordList();

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold flex items-center gap-2">
          <Search className="size-8" />
          搜索单词
        </h1>
        <p className="text-muted-foreground mt-2">
          在您的词网中搜索单词和义项
        </p>
      </div>

      {/* 搜索栏 */}
      <Card>
        <CardHeader>
          <CardTitle>搜索</CardTitle>
          <CardDescription>输入关键词搜索单词或义项</CardDescription>
        </CardHeader>
        <CardContent>
          <WordSearch onSearch={search} />
        </CardContent>
      </Card>

      {/* 搜索结果 */}
      <Card>
        <CardHeader>
          <CardTitle>搜索结果</CardTitle>
          <CardDescription>
            {words.length} 个结果
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-muted-foreground text-center py-8">搜索中...</p>
          ) : words.length > 0 ? (
            <WordList words={words} />
          ) : (
            <div className="text-center py-12">
              <p className="text-muted-foreground">
                没有找到匹配的单词，请尝试其他关键词
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}


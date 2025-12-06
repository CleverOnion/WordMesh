/**
 * 搜索抽屉
 * 包装搜索功能
 */

'use client';

import { FloatingDrawer } from './FloatingDrawer';
import { WordList, WordSearch, useWordList } from '@/modules/word';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface SearchDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SearchDrawer({ isOpen, onClose }: SearchDrawerProps) {
  const { words = [], isLoading, search } = useWordList();

  return (
    <FloatingDrawer
      isOpen={isOpen}
      onClose={onClose}
      direction="right"
      title="搜索单词"
      width="w-full max-w-2xl"
    >
      <div className="space-y-6">
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
            <CardDescription>{words.length} 个结果</CardDescription>
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
    </FloatingDrawer>
  );
}


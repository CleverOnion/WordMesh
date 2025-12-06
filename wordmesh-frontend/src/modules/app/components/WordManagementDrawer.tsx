/**
 * 单词管理抽屉
 * 包装单词管理功能
 */

'use client';

import { useState } from 'react';
import { FloatingDrawer } from './FloatingDrawer';
import { WordList, WordForm, WordSearch, useWordList, useWord } from '@/modules/word';
import { SenseList, SenseForm, useSense } from '@/modules/sense';
import { NoteCard } from '@/modules/note';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Plus, BookOpen } from 'lucide-react';
import type { UserWordAggregate } from '@/modules/word';

interface WordManagementDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function WordManagementDrawer({ isOpen, onClose }: WordManagementDrawerProps) {
  const { words = [], isLoading, refresh, search } = useWordList();
  const { addWord, isLoading: isAddingWord } = useWord();
  const { addSense } = useSense();
  const [selectedWord, setSelectedWord] = useState<UserWordAggregate | null>(null);
  const [showAddWord, setShowAddWord] = useState(false);
  const [showAddSense, setShowAddSense] = useState(false);

  const handleWordSelect = (word: UserWordAggregate) => {
    setSelectedWord(word);
    setShowAddSense(false);
  };

  const handleAddSense = async (data: any) => {
    if (!selectedWord?.user_word.id) return { success: false };
    const result = await addSense(selectedWord.user_word.id, data);
    if (result.success) {
      await refresh();
      setShowAddSense(false);
    }
    return result;
  };

  return (
    <FloatingDrawer
      isOpen={isOpen}
      onClose={onClose}
      direction="right"
      title="单词管理"
      width="w-full max-w-4xl"
    >
      <div className="space-y-6">
        {/* 搜索栏 */}
        <Card>
          <CardContent className="pt-6">
            <WordSearch onSearch={search} />
          </CardContent>
        </Card>

        <div className="grid gap-6 lg:grid-cols-3">
          {/* 单词列表 */}
          <div className="lg:col-span-1">
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle>单词列表</CardTitle>
                    <CardDescription>{words?.length || 0} 个单词</CardDescription>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAddWord(true)}
                  >
                    <Plus className="size-4 mr-2" />
                    添加
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                {isLoading ? (
                  <p className="text-muted-foreground text-center py-8">加载中...</p>
                ) : (
                  <WordList
                    words={words}
                    onSelect={handleWordSelect}
                    selectedWordId={selectedWord?.user_word.id || null}
                  />
                )}
              </CardContent>
            </Card>
          </div>

          {/* 单词详情 */}
          <div className="lg:col-span-2 space-y-4">
            {selectedWord ? (
              <>
                {/* 单词信息 */}
                <Card>
                  <CardHeader>
                    <CardTitle>{selectedWord.word.text}</CardTitle>
                    <CardDescription>
                      单词 ID: {selectedWord.word.id} | 用户词项 ID: {selectedWord.user_word.id}
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {selectedWord.user_word.tags.length > 0 && (
                      <div>
                        <p className="text-sm font-medium mb-2">标签</p>
                        <div className="flex flex-wrap gap-2">
                          {selectedWord.user_word.tags.map((tag, index) => (
                            <span
                              key={index}
                              className="px-2 py-1 bg-primary/10 text-primary rounded-md text-sm"
                            >
                              {tag}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* 笔记 */}
                    <NoteCard
                      node={{ type: 'word', id: selectedWord.user_word.id || 0 }}
                      note={selectedWord.user_word.note}
                      title="单词笔记"
                      onUpdate={refresh}
                    />
                  </CardContent>
                </Card>

                {/* 义项列表 */}
                <Card>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle>义项</CardTitle>
                        <CardDescription>
                          {selectedWord.user_word.senses.length} 个义项
                        </CardDescription>
                      </div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setShowAddSense(true)}
                      >
                        <Plus className="size-4 mr-2" />
                        添加义项
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <SenseList
                      senses={selectedWord.user_word.senses}
                      onEdit={(sense) => {
                        // 可以打开编辑对话框
                      }}
                      onDelete={async (senseId) => {
                        await refresh();
                      }}
                    />
                  </CardContent>
                </Card>

                {/* 添加义项表单 */}
                {showAddSense && (
                  <Card>
                    <CardHeader>
                      <CardTitle>添加新义项</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <SenseForm
                        onSubmit={handleAddSense}
                        onCancel={() => setShowAddSense(false)}
                      />
                    </CardContent>
                  </Card>
                )}
              </>
            ) : (
              <Card>
                <CardContent className="py-12 text-center">
                  <p className="text-muted-foreground">
                    请从左侧选择一个单词查看详情
                  </p>
                </CardContent>
              </Card>
            )}
          </div>
        </div>

        {/* 添加单词表单 */}
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
    </FloatingDrawer>
  );
}

